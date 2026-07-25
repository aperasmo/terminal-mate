from __future__ import annotations

import pytest
from fastapi.testclient import TestClient

from app.config import Settings
from app.main import create_app

TEST_TOKEN = "test-token-" + ("x" * 48)


@pytest.fixture
def client() -> TestClient:
    return TestClient(
        create_app(
            Settings(
                session_token=TEST_TOKEN,
                environment="test",
            )
        )
    )


@pytest.fixture
def auth_headers() -> dict[str, str]:
    return {"Authorization": f"Bearer {TEST_TOKEN}"}


@pytest.fixture
def windows_profile() -> dict[str, str]:
    return {
        "host_os": "windows",
        "runtime": "local",
        "runtime_name": "Windows",
        "target_os": "windows",
        "shell": "powershell",
        "working_directory": r"D:\APE\Portfolio-Project\terminal-mate",
        "architecture": "x86_64",
        "privilege": "standard-user",
    }

