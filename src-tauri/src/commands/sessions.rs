use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::models::session::{OutputStream, SessionOutputEvent, SessionSummary};
use crate::models::workspace::Workspace;

const BOUNDARY_PREFIX: &str = "__TERMINALMATE_BOUNDARY__";
const OUTPUT_EVENT: &str = "session-output";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

type PendingBoundary = Arc<Mutex<Option<(String, Sender<i32>)>>>;
type EmitFn = Arc<dyn Fn(SessionOutputEvent) + Send + Sync>;

struct SessionHandle {
    #[allow(dead_code)]
    workspace_id: String,
    child: Child,
    stdin: ChildStdin,
    pending_boundary: PendingBoundary,
}

#[derive(Default)]
pub struct SessionManager {
    sessions: Mutex<HashMap<String, SessionHandle>>,
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        for (_, handle) in self.sessions.lock().unwrap().drain() {
            let mut child = handle.child;
            let _ = child.kill();
        }
    }
}

#[tauri::command]
pub fn create_session(
    app: AppHandle,
    manager: State<'_, SessionManager>,
    workspace: Workspace,
) -> Result<SessionSummary, String> {
    let emit: EmitFn = Arc::new(move |event| {
        let _ = app.emit(OUTPUT_EVENT, event);
    });
    spawn_session(manager.inner(), emit, workspace)
}

#[tauri::command]
pub fn close_session(manager: State<'_, SessionManager>, session_id: String) -> Result<(), String> {
    terminate_session(manager.inner(), &session_id)
}

/// Runs a command the user already approved (or that classified as Safe) in
/// an existing session. Blocked commands are refused here regardless of what
/// the caller believes was already decided — the execution boundary is the
/// one place a Blocked classification cannot be bypassed.
#[tauri::command]
pub fn execute_command(
    manager: State<'_, SessionManager>,
    session_id: String,
    command: String,
) -> Result<i32, String> {
    let decision = crate::commands::policy::classify_command(&command);
    if decision.level == crate::models::policy::RiskLevel::Blocked {
        return Err(format!("Blocked: {}", decision.reason));
    }

    run_in_session(manager.inner(), &session_id, &command)
}

fn spawn_session(
    manager: &SessionManager,
    emit: EmitFn,
    workspace: Workspace,
) -> Result<SessionSummary, String> {
    let mut child = Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-Command", "-"])
        .current_dir(&workspace.profile.working_directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start PowerShell: {error}"))?;

    let session_id = Uuid::new_v4().to_string();
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to open the session input pipe.".to_owned())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to open the session output pipe.".to_owned())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to open the session error pipe.".to_owned())?;

    let pending_boundary: PendingBoundary = Arc::new(Mutex::new(None));

    spawn_reader(
        emit.clone(),
        session_id.clone(),
        stdout,
        OutputStream::Stdout,
        pending_boundary.clone(),
    );
    spawn_reader(
        emit,
        session_id.clone(),
        stderr,
        OutputStream::Stderr,
        pending_boundary.clone(),
    );

    manager.sessions.lock().unwrap().insert(
        session_id.clone(),
        SessionHandle {
            workspace_id: workspace.id.clone(),
            child,
            stdin,
            pending_boundary,
        },
    );

    // Prove the session is live end-to-end with a Safe, read-only command
    // before handing the session back to the caller.
    run_in_session(manager, &session_id, "Get-Location")?;

    Ok(SessionSummary {
        id: session_id,
        workspace_id: workspace.id,
        shell: "powershell".to_owned(),
    })
}

fn terminate_session(manager: &SessionManager, session_id: &str) -> Result<(), String> {
    let handle = manager.sessions.lock().unwrap().remove(session_id);
    let Some(mut handle) = handle else {
        return Ok(());
    };

    let _ = handle.child.kill();
    let _ = handle.child.wait();
    Ok(())
}

fn run_in_session(manager: &SessionManager, session_id: &str, command: &str) -> Result<i32, String> {
    let token = Uuid::new_v4().to_string();
    let (tx, rx) = channel::<i32>();

    {
        let mut sessions = manager.sessions.lock().unwrap();
        let handle = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Unknown terminal session.".to_owned())?;

        *handle.pending_boundary.lock().unwrap() = Some((token.clone(), tx));

        writeln!(handle.stdin, "{command}").map_err(|error| error.to_string())?;
        writeln!(
            handle.stdin,
            "Write-Output \"{BOUNDARY_PREFIX}{token}:$LASTEXITCODE\""
        )
        .map_err(|error| error.to_string())?;
        handle.stdin.flush().map_err(|error| error.to_string())?;
    }

    rx.recv_timeout(COMMAND_TIMEOUT)
        .map_err(|_| "The command did not complete in time.".to_owned())
}

fn spawn_reader(
    emit: EmitFn,
    session_id: String,
    stream: impl Read + Send + 'static,
    kind: OutputStream,
    pending: PendingBoundary,
) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let Ok(line) = line else {
                break;
            };

            if let Some(rest) = line.strip_prefix(BOUNDARY_PREFIX) {
                if let Some((token, exit_code)) = rest.split_once(':') {
                    let mut guard = pending.lock().unwrap();
                    if let Some((pending_token, sender)) = guard.take() {
                        if pending_token == token {
                            let _ = sender.send(exit_code.trim().parse().unwrap_or(-1));
                        }
                    }
                }
                continue;
            }

            emit(SessionOutputEvent {
                session_id: session_id.clone(),
                stream: kind,
                chunk: line,
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::workspace::ExecutionProfile;
    use std::time::Instant;

    fn windows_test_workspace() -> Workspace {
        Workspace {
            id: "test-workspace".to_owned(),
            name: "windows".to_owned(),
            path: r"C:\Windows".to_owned(),
            profile: ExecutionProfile {
                host_os: "windows".to_owned(),
                runtime: "local".to_owned(),
                runtime_name: "Windows".to_owned(),
                target_os: "windows".to_owned(),
                shell: "powershell".to_owned(),
                working_directory: r"C:\Windows".to_owned(),
                architecture: "x86_64".to_owned(),
                privilege: "standard-user".to_owned(),
            },
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn creates_a_live_session_and_streams_its_startup_command_output() {
        let manager = SessionManager::default();
        let captured: Arc<Mutex<Vec<SessionOutputEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_for_emit = captured.clone();
        let emit: EmitFn = Arc::new(move |event| captured_for_emit.lock().unwrap().push(event));

        let summary = spawn_session(&manager, emit, windows_test_workspace())
            .expect("a real PowerShell session should start on this Windows dev machine");
        assert_eq!(summary.shell, "powershell");
        assert_eq!(summary.workspace_id, "test-workspace");

        let deadline = Instant::now() + Duration::from_secs(5);
        while captured.lock().unwrap().is_empty() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }

        let events = captured.lock().unwrap();
        assert!(
            events.iter().any(|event| event.chunk.contains("Windows")),
            "expected the Get-Location startup command to stream a line containing 'Windows', got: {events:?}"
        );
        drop(events);

        terminate_session(&manager, &summary.id).expect("closing a live session should succeed");
        assert!(
            manager.sessions.lock().unwrap().get(&summary.id).is_none(),
            "closed session should be removed from the manager"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn closing_an_unknown_session_is_a_harmless_no_op() {
        let manager = SessionManager::default();
        assert!(terminate_session(&manager, "does-not-exist").is_ok());
    }
}
