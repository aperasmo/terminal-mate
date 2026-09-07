import asyncio
import json

import httpx
import pytest

from app.intent.action_catalog import validate_clarification, validate_intent
from app.intent.ai_planner import AiPlanner
from app.intent.hybrid_interpreter import HybridIntentInterpreter
from app.schemas.intents import (
    AiPlannerConfig,
    CommandIntent,
    ExecutionProfile,
    IntentRequest,
    IntentClarification,
    IntentResponse,
)


def request(*, enabled: bool = True, api_key: str | None = "test-secret") -> IntentRequest:
    return IntentRequest(
        message="Locate Python files changed by the migration work",
        execution_profile=ExecutionProfile(
            host_os="windows",
            runtime="local",
            runtime_name="Windows",
            target_os="windows",
            shell="powershell",
            working_directory=r"D:\work",
            architecture="x86_64",
            privilege="standard-user",
        ),
        ai_config=AiPlannerConfig(
            enabled=enabled,
            endpoint="https://example.test/v1/chat/completions",
            model="test-model",
            api_key=api_key,
        ),
    )


def response_payload(content: object) -> dict:
    return {"choices": [{"message": {"content": json.dumps(content)}}]}


def test_ai_planner_accepts_a_supported_structured_intent() -> None:
    async def handler(call: httpx.Request) -> httpx.Response:
        assert call.headers["Authorization"] == "Bearer test-secret"
        payload = json.loads(call.content)
        assert payload["model"] == "test-model"
        assert "shell commands" in payload["messages"][0]["content"]
        return httpx.Response(
            200,
            json=response_payload(
                {
                    "schema_version": 1,
                    "action": "find_files",
                    "parameters": {"pattern": "*.py", "recursive": True},
                }
            ),
        )

    async def run() -> CommandIntent | None:
        async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as client:
            return await AiPlanner(client).plan(request(), request().ai_config)

    intent = asyncio.run(run())
    assert intent is not None
    assert intent.action == "find_files"
    assert intent.parameters == {"pattern": "*.py", "recursive": True}


@pytest.mark.parametrize(
    "content",
    [
        {"schema_version": 1, "action": "run_arbitrary_shell", "parameters": {}},
        {
            "schema_version": 1,
            "action": "find_files",
            "parameters": {"pattern": "*.py"},
            "command": "Get-ChildItem *.py",
        },
        {"schema_version": 1, "action": "inspect_port", "parameters": {"port": 70000}},
    ],
)
def test_ai_planner_rejects_actions_outside_the_validated_contract(content: dict) -> None:
    async def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json=response_payload(content))

    async def run() -> None:
        async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as client:
            await AiPlanner(client).plan(request(), request().ai_config)

    with pytest.raises(ValueError):
        asyncio.run(run())


def test_ai_planner_allows_an_explicit_unsupported_response() -> None:
    async def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json=response_payload({"unsupported": True}))

    async def run() -> CommandIntent | None:
        async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as client:
            return await AiPlanner(client).plan(request(), request().ai_config)

    assert asyncio.run(run()) is None


def test_ai_planner_accepts_a_complete_aws_budget_intent() -> None:
    content = {
        "schema_version": 1,
        "action": "aws_budget_create",
        "parameters": {
            "account_id": "123456789012",
            "budget_name": "Waypoint-Monthly-Budget",
            "amount": "5",
        },
    }

    async def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json=response_payload(content))

    async def run() -> CommandIntent | IntentClarification | None:
        async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as client:
            return await AiPlanner(client).plan(request(), request().ai_config)

    intent = asyncio.run(run())
    assert isinstance(intent, CommandIntent)
    assert intent.action == "aws_budget_create"
    assert intent.parameters["account_id"] == "123456789012"


def test_ai_planner_requests_missing_aws_budget_parameters_without_guessing() -> None:
    content = {
        "schema_version": 1,
        "action": "aws_budget_create",
        "parameters": {},
        "missing_parameters": ["account_id", "budget_name", "amount"],
        "question": "What AWS account ID, budget name, and amount should I use?",
    }

    async def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json=response_payload(content))

    async def run() -> CommandIntent | IntentClarification | None:
        async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as client:
            return await AiPlanner(client).plan(request(), request().ai_config)

    clarification = asyncio.run(run())
    assert isinstance(clarification, IntentClarification)
    assert set(clarification.missing_parameters) == {
        "account_id",
        "budget_name",
        "amount",
    }


def test_ai_planner_requests_every_required_aws_budget_alert_value() -> None:
    content = {
        "schema_version": 1,
        "action": "aws_budget_alert_create",
        "parameters": {"amount": "5"},
        "missing_parameters": [
            "account_id",
            "budget_name",
            "threshold",
            "notification_email",
        ],
        "question": "Which account, budget name, threshold, and email should I use?",
    }

    async def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json=response_payload(content))

    async def run() -> CommandIntent | IntentClarification | None:
        async with httpx.AsyncClient(transport=httpx.MockTransport(handler)) as client:
            return await AiPlanner(client).plan(request(), request().ai_config)

    clarification = asyncio.run(run())
    assert isinstance(clarification, IntentClarification)
    assert set(clarification.missing_parameters) == {
        "account_id",
        "budget_name",
        "threshold",
        "notification_email",
    }


def test_hybrid_interpreter_returns_guided_cloud_clarification() -> None:
    class Local:
        def match(self, _: IntentRequest) -> IntentResponse:
            return IntentResponse(matched=False, source="local", intent=None, message=None)

    class Ai:
        async def plan(self, *_args):
            return IntentClarification(
                schema_version=1,
                action="aws_budget_create",
                parameters={"amount": "5"},
                missing_parameters=["account_id", "budget_name"],
                question="Which AWS account ID and budget name should I use?",
            )

    result = asyncio.run(HybridIntentInterpreter(Local(), Ai()).interpret(request()))
    assert result.matched is False
    assert result.requires_clarification is True
    assert "account id" in result.message.lower()
    assert "Try:" in result.message


def test_cloud_validation_rejects_invalid_or_guessed_identifiers_and_amounts() -> None:
    with pytest.raises(ValueError, match="12 digits"):
        validate_intent(
            CommandIntent(
                schema_version=1,
                action="aws_budget_create",
                parameters={
                    "account_id": "123",
                    "budget_name": "Monthly",
                    "amount": "5",
                },
            )
        )

    with pytest.raises(ValueError, match="positive number"):
        validate_intent(
            CommandIntent(
                schema_version=1,
                action="gcp_budget_create",
                parameters={
                    "billing_account": "000000-000000-000000",
                    "display_name": "Monthly",
                    "amount": "0",
                },
            )
        )

    with pytest.raises(ValueError, match="exact missing"):
        validate_clarification(
            IntentClarification(
                schema_version=1,
                action="aws_budget_create",
                parameters={},
                missing_parameters=["amount"],
                question="What amount should I use?",
            )
        )


def test_local_match_always_wins_before_ai() -> None:
    class Local:
        def match(self, _: IntentRequest) -> IntentResponse:
            return IntentResponse(
                matched=True,
                source="local",
                intent=CommandIntent(
                    schema_version=1,
                    action="show_current_directory",
                    parameters={},
                ),
                message=None,
            )

    class Ai:
        async def plan(self, *_args):
            raise AssertionError("AI must not be called for a local match")

    result = asyncio.run(HybridIntentInterpreter(Local(), Ai()).interpret(request()))
    assert result.matched is True
    assert result.source == "local"


def test_ai_provider_failure_returns_a_generic_message_without_the_key() -> None:
    class Local:
        def match(self, _: IntentRequest) -> IntentResponse:
            return IntentResponse(matched=False, source="local", intent=None, message=None)

    class Ai:
        async def plan(self, *_args):
            raise RuntimeError("provider rejected test-secret")

    result = asyncio.run(HybridIntentInterpreter(Local(), Ai()).interpret(request()))
    assert result.matched is False
    assert result.source == "ai"
    assert result.message is not None
    assert "test-secret" not in result.message


def test_catalog_rejects_non_string_list_parameters() -> None:
    with pytest.raises(ValueError):
        validate_intent(
            CommandIntent(
                schema_version=1,
                action="create_files",
                parameters={"paths": ["one.txt", 2]},
            )
        )


@pytest.mark.parametrize(
    "endpoint",
    [
        "http://example.test/v1/chat/completions",
        "http://localhost.example.test/v1/chat/completions",
        "file:///tmp/provider",
    ],
)
def test_ai_planner_rejects_insecure_non_loopback_endpoints(endpoint: str) -> None:
    with pytest.raises(ValueError):
        AiPlannerConfig(enabled=True, endpoint=endpoint, model="test-model")


@pytest.mark.parametrize(
    "endpoint",
    [
        "http://localhost:11434/v1/chat/completions",
        "http://127.0.0.1:11434/v1/chat/completions",
        "http://[::1]:11434/v1/chat/completions",
        "https://example.test/v1/chat/completions",
    ],
)
def test_ai_planner_allows_https_and_exact_loopback_endpoints(endpoint: str) -> None:
    config = AiPlannerConfig(enabled=True, endpoint=endpoint, model="test-model")
    assert config.endpoint == endpoint
