from __future__ import annotations

import asyncio

import pytest

from app.intent.catalog_matcher import DeterministicCatalogMatcher
from app.intent.hybrid_interpreter import HybridIntentInterpreter
from app.schemas.intents import AiPlannerConfig, ExecutionProfile, IntentRequest


@pytest.mark.parametrize(
    ("message", "command_fragment"),
    [
        ("check consumption list", "az consumption usage list"),
        ("check consumption usage", "az consumption usage list"),
        ("show azure usage", "az consumption usage list"),
        ("show lambda quota", "aws service-quotas get-service-quota"),
        ("docker images", "docker images"),
        ("git status", "git status --short --branch"),
    ],
)
def test_matches_catalog_examples_locally(message: str, command_fragment: str) -> None:
    result = DeterministicCatalogMatcher().match(message)

    assert result is not None
    if result.requires_clarification:
        assert (
            message.startswith("check consumption")
            or message == "show azure usage"
            or message == "show lambda quota"
        )
        return
    assert result.matched is True
    assert result.source == "local"
    assert result.intent is not None
    assert result.intent.action == "catalog_command"
    assert command_fragment in result.intent.parameters["command"]


def test_parameterized_catalog_alias_extracts_value() -> None:
    result = DeterministicCatalogMatcher().match("use subscription Production")

    assert result is not None and result.matched
    assert result.intent is not None
    assert result.intent.parameters["command"] == 'az account set --subscription "Production"'


def test_catalog_requests_missing_workflow_parameters_without_ai() -> None:
    result = DeterministicCatalogMatcher().match("list github workflows")

    assert result is not None
    assert result.matched is False
    assert result.requires_clarification is True
    assert "owner-repo" in result.message
    assert "workflow" in result.message


def test_hybrid_interpreter_does_not_call_ai_for_catalog_match() -> None:
    class Ai:
        async def plan(self, *_args):
            raise AssertionError("AI fallback must not run for a deterministic catalog match")

    request = IntentRequest(
        message="docker images",
        execution_profile=ExecutionProfile(
            host_os="windows",
            runtime="local",
            runtime_name="Windows PowerShell",
            target_os="windows",
            shell="powershell",
            working_directory="D:\\work",
            architecture="x86_64",
            privilege="standard",
        ),
        ai_config=AiPlannerConfig(
            enabled=True,
            endpoint="http://127.0.0.1:11434/v1/chat/completions",
            model="test-model",
        ),
    )
    result = asyncio.run(HybridIntentInterpreter(ai_planner=Ai()).interpret(request))

    assert result.matched is True
    assert result.source == "local"
    assert result.intent is not None
    assert result.intent.action == "catalog_command"
