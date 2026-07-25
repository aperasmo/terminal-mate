from __future__ import annotations

from fastapi import FastAPI

from app.api import health, intents
from app.config import Settings


def create_app(settings: Settings) -> FastAPI:
    app = FastAPI(
        title="TerminalMate Local Sidecar",
        version="0.1.0",
        docs_url="/docs" if settings.environment == "development" else None,
        redoc_url=None,
        openapi_url="/openapi.json" if settings.environment == "development" else None,
    )
    app.state.settings = settings
    app.include_router(health.router)
    app.include_router(intents.router)
    return app

