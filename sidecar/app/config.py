from __future__ import annotations

import os
from dataclasses import dataclass


@dataclass(frozen=True)
class Settings:
    session_token: str
    environment: str

    @classmethod
    def from_env(cls) -> "Settings":
        token = os.getenv("TERMINAL_MATE_SESSION_TOKEN", "")
        if len(token) < 32:
            raise RuntimeError(
                "TERMINAL_MATE_SESSION_TOKEN must contain at least 32 characters."
            )

        return cls(
            session_token=token,
            environment=os.getenv("TERMINAL_MATE_ENV", "development"),
        )

