from __future__ import annotations

from typing import Any, Literal

from pydantic import BaseModel, Field


class ExecutionProfile(BaseModel):
    host_os: str
    runtime: Literal["local", "wsl", "ssh", "container", "cloud"]
    runtime_name: str
    target_os: str
    shell: str
    working_directory: str
    architecture: str
    privilege: str


class IntentRequest(BaseModel):
    message: str = Field(min_length=1, max_length=4_000)
    execution_profile: ExecutionProfile


class CommandIntent(BaseModel):
    schema_version: Literal[1]
    action: str
    parameters: dict[str, Any]


class IntentResponse(BaseModel):
    matched: bool
    source: Literal["local", "ai"]
    intent: CommandIntent | None
    message: str | None

