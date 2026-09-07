from __future__ import annotations

from app.intent.ai_planner import AiPlanner
from app.intent.action_catalog import clarification_message
from app.intent.local_matcher import LocalIntentMatcher
from app.schemas.intents import IntentClarification, IntentRequest, IntentResponse


class HybridIntentInterpreter:
    def __init__(
        self,
        local_matcher: LocalIntentMatcher | None = None,
        ai_planner: AiPlanner | None = None,
    ) -> None:
        self._local = local_matcher or LocalIntentMatcher()
        self._ai = ai_planner or AiPlanner()

    async def interpret(self, request: IntentRequest) -> IntentResponse:
        local = self._local.match(request)
        if local.matched or local.requires_clarification:
            return local

        config = request.ai_config
        if config is None or not config.enabled:
            return local

        try:
            intent = await self._ai.plan(request, config)
        except Exception:
            # Provider failures never weaken the safety boundary and never
            # expose endpoint responses, credentials, or request details.
            return IntentResponse(
                matched=False,
                source="ai",
                intent=None,
                message="AI planning was unavailable; review the request as a literal command.",
            )

        if intent is None:
            return IntentResponse(
                matched=False,
                source="ai",
                intent=None,
                message="No supported TerminalMate action matched this request.",
            )
        if isinstance(intent, IntentClarification):
            return IntentResponse(
                matched=False,
                source="ai",
                intent=None,
                message=clarification_message(intent),
                requires_clarification=True,
            )
        return IntentResponse(matched=True, source="ai", intent=intent, message=None)
