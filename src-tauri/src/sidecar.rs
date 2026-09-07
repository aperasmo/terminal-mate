use std::time::Duration;

use serde::Deserialize;
use tauri::AppHandle;
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};
use uuid::Uuid;

use crate::{app_state::AppState, sidecar_proxy::SidecarProxy};

const READY_PREFIX: &str = "TERMINAL_MATE_READY:";

#[derive(Deserialize)]
struct ReadyMessage {
    port: u16,
    protocol_version: String,
}

// TerminalMate's Python schemas
// (schemas/system.py, schemas/intents.py) are plain pydantic BaseModels with
// no camelCase alias generator, so the JSON body really is snake_case —
// matching these field names directly, with no rename_all needed.
#[derive(Deserialize)]
struct HealthResponse {
    status: String,
    protocol_version: String,
    #[allow(dead_code)]
    service_version: String,
}

pub fn launch(app: AppHandle, state: AppState) {
    tauri::async_runtime::spawn(async move {
        state.starting().await;

        // A session token exists only in memory for this desktop-app session.
        // It is passed directly to the child process and is never exposed to React.
        let token = Uuid::new_v4().to_string() + &Uuid::new_v4().to_string();
        let sidecar = match app.shell().sidecar("terminal-mate-sidecar") {
            Ok(command) => command
                .env("TERMINAL_MATE_SESSION_TOKEN", &token)
                .env("TERMINAL_MATE_ENV", "production")
                .env("TERMINAL_MATE_PARENT_PID", std::process::id().to_string()),
            Err(error) => {
                state
                    .failed(format!("Unable to configure local assistant service: {error}"))
                    .await;
                return;
            }
        };

        let (mut events, child) = match sidecar.spawn() {
            Ok(spawned) => spawned,
            Err(error) => {
                state
                    .failed(format!("Unable to start local assistant service: {error}"))
                    .await;
                return;
            }
        };
        // Keep the child handle before waiting for readiness. A shutdown request
        // during startup must still be able to terminate the process.
        state.store_sidecar_child(child).await;
        let proxy = match SidecarProxy::new() {
            Ok(proxy) => proxy,
            Err(message) => {
                state.failed(message).await;
                return;
            }
        };

        while let Some(event) = events.recv().await {
            match event {
                CommandEvent::Stdout(bytes) => {
                    // Tauri exposes sidecar stdout as bytes, not String. Decode it
                    // strictly so malformed process output cannot be silently accepted.
                    let line = match std::str::from_utf8(&bytes) {
                        Ok(line) => line.trim(),
                        Err(_) => {
                            state
                                .failed("Local assistant service returned non-UTF-8 startup output.")
                                .await;
                            return;
                        }
                    };

                    // stdout is reserved for one readiness event. Any other output
                    // is a protocol violation rather than something to ignore.
                    let Some(raw_payload) = line.strip_prefix(READY_PREFIX) else {
                        state
                            .failed("Local assistant service returned an unexpected startup message.")
                            .await;
                        return;
                    };

                    let ready = match serde_json::from_str::<ReadyMessage>(raw_payload) {
                        Ok(ready) if ready.port > 0 && ready.protocol_version == "1" => ready,
                        Ok(_) => {
                            state
                                .failed("Local assistant service returned an unsupported startup protocol.")
                                .await;
                            return;
                        }
                        Err(_) => {
                            state
                                .failed("Local assistant service returned an invalid startup message.")
                                .await;
                            return;
                        }
                    };

                    // Rust retains both the loopback endpoint and token after this
                    // point. The React webview receives only typed command results.
                    let endpoint = format!("http://127.0.0.1:{}", ready.port);
                    state
                        .configure_sidecar(endpoint.clone(), token.clone(), ready.protocol_version)
                        .await;

                    // The process is not considered ready merely because it bound a
                    // port. It must also accept an authenticated health request.
                    for _ in 0..20 {
                        let health = proxy
                            .get::<HealthResponse>(&endpoint, &token, "/v1/health")
                            .await;
                        if let Ok(response) = health {
                            if response.status == "ok" && response.protocol_version == "1" {
                                state.ready().await;
                                return;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(150)).await;
                    }

                    state
                        .failed("The local assistant service started but did not pass authenticated health checks.")
                        .await;
                    return;
                }
                CommandEvent::Stderr(bytes) => {
                    // Keep only a small local diagnostic buffer, not surfaced to the
                    // command bar today — useful if this needs debugging later.
                    if let Ok(line) = std::str::from_utf8(&bytes) {
                        state.record_sidecar_log(line).await;
                    }
                }
                CommandEvent::Error(error) => {
                    state
                        .failed(format!("Local assistant service error: {error}"))
                        .await;
                    return;
                }
                CommandEvent::Terminated(payload) => {
                    state
                        .failed(format!(
                            "The local assistant service stopped unexpectedly (exit code {:?}).",
                            payload.code
                        ))
                        .await;
                    return;
                }
                _ => {}
            }
        }

        state
            .failed("The local assistant service stopped before completing startup.")
            .await;
    });
}

#[cfg(target_os = "windows")]
async fn force_stop_sidecar_tree(pid: u32) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let _ = tokio::task::spawn_blocking(move || {
        let pid_argument = pid.to_string();

        let _ = std::process::Command::new("taskkill")
            .args(["/PID", pid_argument.as_str(), "/T", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    })
    .await;
}

#[cfg(not(target_os = "windows"))]
async fn force_stop_sidecar_child(child: CommandChild) {
    let _ = child.kill();
}

#[cfg(target_os = "windows")]
async fn force_stop_sidecar_child(child: CommandChild) {
    force_stop_sidecar_tree(child.pid()).await;
}

pub async fn stop_sidecar(state: AppState) {
    // The desktop app owns this sidecar PID. Windows needs taskkill /T to
    // remove PyInstaller child processes; Unix-like systems can terminate the
    // sidecar child directly through Tauri's shell process handle.
    let child = {
        let mut runtime = state.runtime.lock().await;
        runtime.child.take()
    };

    if let Some(child) = child {
        force_stop_sidecar_child(child).await;
    }

    state.stopped().await;
}

#[cfg(test)]
mod tests {
    use super::HealthResponse;
    use crate::sidecar_proxy::SidecarProxy;
    use std::io::BufRead;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    /// Spawns the real, compiled sidecar binary the same way `launch()` does
    /// conceptually (minus going through `tauri_plugin_shell`, which needs a
    /// live `AppHandle` unit tests don't have), and hits its authenticated
    /// `/v1/health` endpoint the exact same way the startup health-check loop
    /// does. This exists because a prior version of `HealthResponse` assumed
    /// camelCase JSON keys (copied from a sibling project whose Python schemas
    /// actually alias to camelCase) while this sidecar's schemas are plain
    /// snake_case — every health check silently failed, the sidecar was
    /// marked "failed" after exhausting its retries, and every command
    /// silently fell back to direct handling with no visible error. A test
    /// that only checks `try_interpret` against an already-configured,
    /// already-"ready" `AppState` (as `commands::intents::tests` does) can
    /// never catch this, because it never exercises the health check itself.
    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn health_response_deserializes_from_the_real_sidecar() {
        let binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join("terminal-mate-sidecar-x86_64-pc-windows-msvc.exe");
        assert!(
            binary.exists(),
            "sidecar binary not found at {binary:?} — run `npm run sidecar:build` first"
        );

        let token = "test-token-0123456789abcdef0123456789abcdef".to_owned();
        let mut child = Command::new(&binary)
            .env("TERMINAL_MATE_SESSION_TOKEN", &token)
            .env("TERMINAL_MATE_ENV", "test")
            .env("TERMINAL_MATE_PARENT_PID", std::process::id().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("the real sidecar binary should spawn");

        let stdout = child.stdout.take().expect("sidecar stdout should be piped");
        let mut reader = std::io::BufReader::new(stdout);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("the sidecar should print a readiness line");
        let payload = line
            .trim()
            .strip_prefix(super::READY_PREFIX)
            .expect("the sidecar's first stdout line should be the readiness message");
        let parsed: serde_json::Value =
            serde_json::from_str(payload).expect("the readiness payload should be valid JSON");
        let port = parsed["port"]
            .as_u64()
            .expect("the readiness payload should include a port");
        let endpoint = format!("http://127.0.0.1:{port}");

        let proxy = SidecarProxy::new().expect("the proxy should build");
        let health: HealthResponse = proxy
            .get(&endpoint, &token, "/v1/health")
            .await
            .expect("the real sidecar's health response should deserialize");

        assert_eq!(health.status, "ok");
        assert_eq!(health.protocol_version, "1");

        let _ = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .output();
    }
}
