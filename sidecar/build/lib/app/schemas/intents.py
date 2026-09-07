from __future__ import annotations

from typing import Any, Literal
from urllib.parse import urlparse

from pydantic import BaseModel, ConfigDict, Field, field_validator


class ExecutionProfile(BaseModel):
    host_os: str
    runtime: Literal["local", "wsl", "ssh", "container", "cloud"]
    runtime_name: str
    target_os: str
    shell: str
    working_directory: str
    architecture: str
    privilege: str


class AiPlannerConfig(BaseModel):
    model_config = ConfigDict(extra="forbid")

    enabled: bool = False
    endpoint: str = Field(min_length=1, max_length=2_000)
    model: str = Field(min_length=1, max_length=200)
    api_key: str | None = Field(default=None, max_length=4_000)

    @field_validator("endpoint")
    @classmethod
    def validate_endpoint(cls, value: str) -> str:
        normalized = value.strip()
        parsed = urlparse(normalized)
        is_https = parsed.scheme == "https" and bool(parsed.hostname)
        is_loopback_http = parsed.scheme == "http" and parsed.hostname in {
            "localhost",
            "127.0.0.1",
            "::1",
        }
        if not (is_https or is_loopback_http):
            raise ValueError("AI endpoint must use HTTPS, or HTTP on localhost.")
        return normalized


class IntentRequest(BaseModel):
    message: str = Field(min_length=1, max_length=4_000)
    execution_profile: ExecutionProfile
    ai_config: AiPlannerConfig | None = None


class CommandIntent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    schema_version: Literal[1]
    action: str
    parameters: dict[str, Any]


class IntentClarification(BaseModel):
    model_config = ConfigDict(extra="forbid")

    schema_version: Literal[1]
    action: str
    parameters: dict[str, Any]
    missing_parameters: list[str]
    question: str = Field(min_length=1, max_length=1_000)


class IntentResponse(BaseModel):
    matched: bool
    source: Literal["local", "ai"]
    intent: CommandIntent | None
    message: str | None
    requires_clarification: bool = False
