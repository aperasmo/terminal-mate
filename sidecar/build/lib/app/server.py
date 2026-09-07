from __future__ import annotations

import json

import uvicorn

from app.config import Settings
from app.main import create_app
from app.process_guard import start_parent_process_guard


class ReadyServer(uvicorn.Server):
    async def startup(self, sockets=None) -> None:  # type: ignore[override]
        await super().startup(sockets=sockets)
        if not self.started:
            return

        server = next(iter(self.servers), None)
        if server is None or not server.sockets:
            raise RuntimeError("Sidecar did not expose a loopback socket.")

        port = int(server.sockets[0].getsockname()[1])
        payload = json.dumps({"port": port, "protocol_version": "1"}, separators=(",", ":"))
        print(f"TERMINAL_MATE_READY:{payload}", flush=True)


def run_sidecar() -> None:
    start_parent_process_guard()
    settings = Settings.from_env()
    app = create_app(settings)
    config = uvicorn.Config(
        app,
        host="127.0.0.1",
        port=0,
        log_config=None,
        access_log=False,
        lifespan="on",
    )
    ReadyServer(config).run()
