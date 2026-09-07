from __future__ import annotations

import pytest

from app.intent.local_matcher import LocalIntentMatcher
from app.schemas.intents import ExecutionProfile, IntentRequest


def request(message: str) -> IntentRequest:
    return IntentRequest(
        message=message,
        execution_profile=ExecutionProfile(
            host_os="windows",
            runtime="local",
            runtime_name="Windows PowerShell",
            target_os="windows",
            shell="powershell",
            working_directory=r"D:\APE\Portfolio-Project\waypoint",
            architecture="x86_64",
            privilege="standard",
        ),
    )


@pytest.mark.parametrize(
    ("message", "fragment"),
    [
        ("check for any leaks", 'gitleaks git --redact=100 --log-opts="--all"'),
        ("scan for secrets", 'gitleaks git --redact=100 --log-opts="--all"'),
        ("check current files for leaks", "gitleaks dir --redact=100"),
        ("check latest commit for leaks", 'gitleaks git --redact=100 --log-opts="-1"'),
        ("gitleaks version", "gitleaks version"),
        ("show gitleaks report", "Get-Content -LiteralPath"),
    ],
)
def test_gitleaks_requests_match_locally(message: str, fragment: str) -> None:
    result = LocalIntentMatcher().match(request(message))

    assert result.matched is True
    assert result.source == "local"
    assert result.intent is not None
    assert result.intent.action == "catalog_command"
    assert fragment in result.intent.parameters["command"]


def test_full_scan_uses_repo_specific_redacted_json_report() -> None:
    result = LocalIntentMatcher().match(request("check for any leaks"))

    assert result.intent is not None
    assert result.intent.parameters["command"] == (
        'gitleaks git --redact=100 --log-opts="--all" --report-format json '
        '--report-path "$env:TEMP\\waypoint-gitleaks-report.json" .'
    )
