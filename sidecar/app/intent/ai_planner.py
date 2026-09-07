from __future__ import annotations

import json
from typing import Any

import httpx
from pydantic import ValidationError

from app.intent.action_catalog import prompt_catalog, validate_clarification, validate_intent
from app.schemas.intents import AiPlannerConfig, CommandIntent, IntentClarification, IntentRequest


SYSTEM_PROMPT = """You are TerminalMate's intent planner.
Translate the user's request into exactly one supported structured action.
Never return shell commands, scripts, explanations, markdown, or extra keys.
Use only the supplied action catalog. Do not invent actions or parameters.
Never guess account IDs, subscription IDs, billing accounts, projects, regions,
zones, resource names, amounts, dates, credentials, SSH hosts, or key paths.
Return JSON with this exact shape:
{"schema_version":1,"action":"action_name","parameters":{}}
If an action matches but required values are missing, return:
{"schema_version":1,"action":"action_name","parameters":{},"missing_parameters":["required_name"],"question":"Ask one concise question for the missing values."}
If no catalog action accurately represents the request, return:
{"unsupported":true}
"""


class AiPlanner:
    def __init__(self, client: httpx.AsyncClient | None = None) -> None:
        self._client = client

    async def plan(
        self, request: IntentRequest, config: AiPlannerConfig
    ) -> CommandIntent | IntentClarification | None:
        payload = {
            "model": config.model,
            "temperature": 0,
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {
                    "role": "user",
                    "content": json.dumps(
                        {
                            "request": request.message,
                            "execution_profile": request.execution_profile.model_dump(),
                            "actions": prompt_catalog(),
                        },
                        separators=(",", ":"),
                    ),
                },
            ],
        }
        headers = {"Content-Type": "application/json"}
        if config.api_key:
            headers["Authorization"] = f"Bearer {config.api_key}"

        if self._client is not None:
            response = await self._client.post(config.endpoint, json=payload, headers=headers)
        else:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(config.endpoint, json=payload, headers=headers)
        response.raise_for_status()
        content = self._extract_content(response.json())
        parsed = self._parse_json(content)
        if parsed == {"unsupported": True}:
            return None
        if set(parsed) == {
            "schema_version",
            "action",
            "parameters",
            "missing_parameters",
            "question",
        }:
            try:
                clarification = IntentClarification.model_validate(parsed)
            except ValidationError as error:
                raise ValueError("AI response did not match the clarification schema.") from error
            return validate_clarification(clarification)
        if set(parsed) != {"schema_version", "action", "parameters"}:
            raise ValueError("AI response did not contain the required intent fields.")
        try:
            intent = CommandIntent.model_validate(parsed)
        except ValidationError as error:
            raise ValueError("AI response did not match the command-intent schema.") from error
        return validate_intent(intent)

    @staticmethod
    def _extract_content(payload: dict[str, Any]) -> str:
        try:
            content = payload["choices"][0]["message"]["content"]
        except (KeyError, IndexError, TypeError) as error:
            raise ValueError("AI provider response did not contain message content.") from error
        if not isinstance(content, str) or not content.strip():
            raise ValueError("AI provider returned empty message content.")
        return content.strip()

    @staticmethod
    def _parse_json(content: str) -> dict[str, Any]:
        candidate = content
        if candidate.startswith("```"):
            lines = candidate.splitlines()
            candidate = "\n".join(lines[1:-1]).strip()
            if candidate.lower().startswith("json"):
                candidate = candidate[4:].lstrip()
        try:
            payload = json.loads(candidate)
        except json.JSONDecodeError as error:
            raise ValueError("AI provider did not return valid JSON.") from error
        if not isinstance(payload, dict):
            raise ValueError("AI provider response must be a JSON object.")
        return payload
