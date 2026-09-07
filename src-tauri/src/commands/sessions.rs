use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::models::session::{OutputStream, SessionOutputEvent, SessionSummary};
use crate::models::workspace::Workspace;

const BOUNDARY_PREFIX: &str = "__TERMINALMATE_BOUNDARY__";
const OUTPUT_EVENT: &str = "session-output";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const PARTIAL_OUTPUT_FLUSH: Duration = Duration::from_millis(75);
const GH_TERMINAL_WIDTH: &str = "120";

type PendingBoundary = Arc<Mutex<Option<PendingExecution>>>;
pub(crate) type EmitFn = Arc<dyn Fn(SessionOutputEvent) + Send + Sync>;

struct PendingExecution {
    token: String,
    sender: Option<Sender<i32>>,
    stdout_boundary_seen: bool,
    stderr_boundary_seen: bool,
    exit_code: i32,
    run_id: Option<String>,
    emit: Option<EmitFn>,
    log_file: Option<Arc<Mutex<File>>>,
    had_output: Arc<AtomicBool>,
}

pub(crate) struct SessionRun {
    pub receiver: Receiver<i32>,
    pub had_output: Arc<AtomicBool>,
}

struct SessionHandle {
    #[allow(dead_code)]
    workspace_id: String,
    working_directory: String,
    shell: String,
    child: Child,
    stdin: ChildStdin,
    pending_boundary: PendingBoundary,
    workspace: Workspace,
    emit: EmitFn,
}

#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SessionHandle>>>,
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        // Worker threads hold short-lived clones so they can update shared
        // session state after a command finishes. Only the final owner may
        // terminate every shell; dropping a worker clone must be a no-op.
        if Arc::strong_count(&self.sessions) != 1 {
            return;
        }
        for (_, handle) in self.sessions.lock().unwrap().drain() {
            let mut child = handle.child;
            let _ = child.kill();
        }
    }
}

impl SessionManager {
    /// The current working directory tracked for this persistent shell.
    pub fn working_directory(&self, session_id: &str) -> Option<String> {
        self.sessions
            .lock()
            .unwrap()
            .get(session_id)
            .map(|handle| handle.working_directory.clone())
    }

    pub fn update_working_directory(
        &self,
        session_id: &str,
        working_directory: String,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        let handle = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Unknown terminal session.".to_owned())?;
        handle.working_directory = normalize_working_directory(&working_directory);
        Ok(())
    }
}

#[tauri::command]
pub fn set_session_working_directory(
    manager: State<'_, SessionManager>,
    session_id: String,
    working_directory: String,
) -> Result<String, String> {
    manager.update_working_directory(&session_id, working_directory)?;
    manager
        .working_directory(&session_id)
        .ok_or_else(|| "Unknown terminal session.".to_owned())
}

fn normalize_working_directory(working_directory: &str) -> String {
    let mut normalized = working_directory
        .trim_start_matches('\u{feff}')
        .trim()
        .strip_prefix("Microsoft.PowerShell.Core\\FileSystem::")
        .unwrap_or(working_directory.trim_start_matches('\u{feff}').trim())
        .to_owned();

    if let Some(unc_path) = normalized.strip_prefix(r"\\?\UNC\") {
        normalized = format!(r"\\{unc_path}");
    } else if let Some(path) = normalized.strip_prefix(r"\\?\") {
        normalized = path.to_owned();
    }

    normalized
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

#[tauri::command]
pub fn send_session_input(
    manager: State<'_, SessionManager>,
    session_id: String,
    input: String,
) -> Result<(), String> {
    send_input(manager.inner(), &session_id, &input)
}

pub(crate) fn send_input(
    manager: &SessionManager,
    session_id: &str,
    input: &str,
) -> Result<(), String> {
    let mut sessions = manager.sessions.lock().unwrap();
    let handle = sessions
        .get_mut(session_id)
        .ok_or_else(|| "Unknown terminal session.".to_owned())?;

    if handle.pending_boundary.lock().unwrap().is_none() {
        return Err("No command is currently waiting for input in this session.".to_owned());
    }

    // Interactive responses can contain passwords or tokens. Send them to the
    // existing shell stdin without emitting or recording their contents.
    writeln!(handle.stdin, "{input}").map_err(|error| error.to_string())?;
    handle.stdin.flush().map_err(|error| error.to_string())
}

pub(crate) fn spawn_session(
    manager: &SessionManager,
    emit: EmitFn,
    workspace: Workspace,
) -> Result<SessionSummary, String> {
    let session_id = Uuid::new_v4().to_string();
    let handle = spawn_session_handle(&session_id, workspace.clone(), emit)?;
    let shell = handle.shell.clone();

    manager
        .sessions
        .lock()
        .unwrap()
        .insert(session_id.clone(), handle);

    // Prove the session is live end-to-end with a Safe, read-only command
    // before handing the session back to the caller.
    let startup_command = if shell == "bash" { "pwd" } else { "Get-Location" };
    run_in_session(manager, &session_id, startup_command)?;

    Ok(SessionSummary {
        id: session_id,
        workspace_id: workspace.id,
        shell,
    })
}

fn spawn_session_handle(
    session_id: &str,
    workspace: Workspace,
    emit: EmitFn,
) -> Result<SessionHandle, String> {
    let shell = workspace.profile.shell.to_ascii_lowercase();
    let mut command = match (workspace.profile.runtime.as_str(), shell.as_str()) {
        ("local", "powershell") => {
            let mut command = Command::new("powershell.exe");
            command
                .args(["-NoLogo", "-NoProfile", "-Command", "-"])
                .current_dir(&workspace.profile.working_directory);
            command
        }
        ("wsl", "bash") => {
            let mut command = Command::new("wsl.exe");
            command.args([
                "--cd",
                &workspace.profile.working_directory,
                "--exec",
                "env",
                "PYTHONUNBUFFERED=1",
                "AWS_PAGER=",
                "GH_FORCE_TTY=120",
                "GH_SPINNER_DISABLED=1",
                "bash",
                "--noprofile",
                "--norc",
            ]);
            command
        }
        _ => {
            return Err(format!(
                "{} / {} is not a supported execution profile.",
                workspace.profile.runtime_name, workspace.profile.shell
            ));
        }
    };
    command
        .env("PYTHONUNBUFFERED", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONUTF8", "1")
        // AWS CLI v2 enables a client-side pager by default. TerminalMate
        // streams through pipes rather than a real TTY, so a pager can keep
        // an otherwise completed AWS command alive waiting for input.
        .env("AWS_PAGER", "")
        // GitHub CLI hides status messages such as `✓ Updated ...` when its
        // output is piped. TerminalMate streams pipes into its own terminal
        // surface, so explicitly request the equivalent terminal output.
        .env("GH_FORCE_TTY", GH_TERMINAL_WIDTH)
        .env("GH_SPINNER_DISABLED", "1");
    hide_console_window(&mut command);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| match shell.as_str() {
            "bash" => format!("Failed to start WSL Bash: {error}"),
            _ => format!("Failed to start PowerShell: {error}"),
        })?;

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
        session_id.to_owned(),
        stdout,
        OutputStream::Stdout,
        pending_boundary.clone(),
    );
    spawn_reader(
        emit.clone(),
        session_id.to_owned(),
        stderr,
        OutputStream::Stderr,
        pending_boundary.clone(),
    );

    Ok(SessionHandle {
        workspace_id: workspace.id.clone(),
        working_directory: workspace.profile.working_directory.clone(),
        shell,
        child,
        stdin,
        pending_boundary,
        workspace,
        emit,
    })
}

#[cfg(target_os = "windows")]
fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_console_window(_command: &mut Command) {}

fn terminate_session(manager: &SessionManager, session_id: &str) -> Result<(), String> {
    let handle = manager.sessions.lock().unwrap().remove(session_id);
    let Some(mut handle) = handle else {
        return Ok(());
    };

    complete_pending_execution(&handle.pending_boundary, 130);
    terminate_process_tree(&mut handle.child);
    let _ = handle.child.wait();
    Ok(())
}

fn run_in_session(manager: &SessionManager, session_id: &str, command: &str) -> Result<i32, String> {
    start_session_run(manager, session_id, command, None, None, None)?
        .receiver
        .recv_timeout(COMMAND_TIMEOUT)
        .map_err(|_| "The command did not complete in time.".to_owned())
}

pub(crate) fn start_session_run(
    manager: &SessionManager,
    session_id: &str,
    command: &str,
    run_id: Option<String>,
    log_path: Option<PathBuf>,
    emit: Option<EmitFn>,
) -> Result<SessionRun, String> {
    let token = Uuid::new_v4().to_string();
    let (tx, rx) = channel::<i32>();
    let had_output = Arc::new(AtomicBool::new(false));
    let log_file = log_path
        .as_deref()
        .map(open_log_file)
        .transpose()?
        .map(|file| Arc::new(Mutex::new(file)));

    {
        let mut sessions = manager.sessions.lock().unwrap();
        let handle = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Unknown terminal session.".to_owned())?;

        let mut pending = handle.pending_boundary.lock().unwrap();
        if pending.is_some() {
            return Err("Another command is already running in this session.".to_owned());
        }
        *pending = Some(PendingExecution {
            token: token.clone(),
            sender: Some(tx),
            stdout_boundary_seen: false,
            stderr_boundary_seen: false,
            exit_code: -1,
            run_id,
            emit,
            log_file,
            had_output: had_output.clone(),
        });
        drop(pending);

        let working_directory_path = log_path
            .as_deref()
            .map(|path| path.with_extension("cwd"));
        let wrapper = if handle.shell == "bash" {
            bash_command_wrapper(command, &token, working_directory_path.as_deref())
        } else {
            powershell_command_wrapper(command, &token, working_directory_path.as_deref())
        };
        if let Err(error) = writeln!(handle.stdin, "{wrapper}") {
            *handle.pending_boundary.lock().unwrap() = None;
            return Err(error.to_string());
        }
        if let Err(error) = handle.stdin.flush() {
            *handle.pending_boundary.lock().unwrap() = None;
            return Err(error.to_string());
        }
    }

    Ok(SessionRun {
        receiver: rx,
        had_output,
    })
}

fn spawn_reader(
    emit: EmitFn,
    session_id: String,
    stream: impl Read + Send + 'static,
    kind: OutputStream,
    pending: PendingBoundary,
) {
    std::thread::spawn(move || {
        let (chunk_sender, chunk_receiver) = channel::<Vec<u8>>();
        std::thread::spawn(move || read_output_chunks(stream, chunk_sender));
        let mut buffer = Vec::new();

        loop {
            match chunk_receiver.recv_timeout(PARTIAL_OUTPUT_FLUSH) {
                Ok(chunk) => {
                    buffer.extend_from_slice(&chunk);
                    process_output_buffer(
                        &emit,
                        &session_id,
                        &pending,
                        kind,
                        &mut buffer,
                        false,
                    );
                }
                Err(RecvTimeoutError::Timeout) => {
                    process_output_buffer(
                        &emit,
                        &session_id,
                        &pending,
                        kind,
                        &mut buffer,
                        true,
                    );
                }
                Err(RecvTimeoutError::Disconnected) => {
                    process_output_buffer(
                        &emit,
                        &session_id,
                        &pending,
                        kind,
                        &mut buffer,
                        true,
                    );
                    break;
                }
            }
        }
    });
}

fn read_output_chunks(mut stream: impl Read, sender: Sender<Vec<u8>>) {
    let mut chunk = vec![0_u8; 8 * 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(bytes_read) => {
                if sender.send(chunk[..bytes_read].to_vec()).is_err() {
                    break;
                }
            }
        }
    }
}

fn process_output_buffer(
    emit: &EmitFn,
    session_id: &str,
    pending: &PendingBoundary,
    kind: OutputStream,
    buffer: &mut Vec<u8>,
    flush_partial: bool,
) {
    loop {
        let expected_marker = pending
            .lock()
            .unwrap()
            .as_ref()
            .map(|execution| format!("{BOUNDARY_PREFIX}{}:", execution.token));
        let marker_index = expected_marker
            .as_deref()
            .and_then(|marker| find_bytes(buffer, marker.as_bytes()));
        let newline_index = buffer.iter().position(|byte| *byte == b'\n');

        if let (Some(marker), Some(marker_index)) = (expected_marker.as_deref(), marker_index) {
            if let Some(newline_index) = newline_index.filter(|index| *index < marker_index) {
                emit_buffered_output(
                    emit,
                    session_id,
                    pending,
                    kind,
                    buffer.drain(..=newline_index).collect(),
                );
                continue;
            }

            let Some(marker_line_end) = buffer[marker_index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map(|relative| marker_index + relative)
            else {
                return;
            };

            let marker_line: Vec<u8> = buffer.drain(..=marker_line_end).collect();
            if marker_index > 0 {
                emit_buffered_output(
                    emit,
                    session_id,
                    pending,
                    kind,
                    marker_line[..marker_index].to_vec(),
                );
            }
            let exit_code = crate::commands::command_runs::decode_output_line(
                marker_line[marker_index + marker.len()..].to_vec(),
            );
            let token = &marker[BOUNDARY_PREFIX.len()..marker.len() - 1];
            mark_boundary(pending, token, &exit_code, kind);
            continue;
        }

        if let Some(newline_index) = newline_index {
            emit_buffered_output(
                emit,
                session_id,
                pending,
                kind,
                buffer.drain(..=newline_index).collect(),
            );
            continue;
        }

        if flush_partial && !buffer.is_empty() {
            let retained = expected_marker
                .as_deref()
                .map(|marker| marker_prefix_suffix_len(buffer, marker.as_bytes()))
                .unwrap_or(0);
            let visible_len = buffer.len().saturating_sub(retained);
            if visible_len > 0 {
                emit_buffered_output(
                    emit,
                    session_id,
                    pending,
                    kind,
                    buffer.drain(..visible_len).collect(),
                );
            }
        }
        return;
    }
}

fn emit_buffered_output(
    emit: &EmitFn,
    session_id: &str,
    pending: &PendingBoundary,
    kind: OutputStream,
    bytes: Vec<u8>,
) {
    let line = crate::commands::command_runs::decode_output_line(bytes);
    let line = crate::commands::command_runs::strip_ansi_escapes(&line).into_owned();
    if !line.is_empty() {
        emit_output_line(emit, session_id, pending, kind, line);
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    (!needle.is_empty())
        .then(|| haystack.windows(needle.len()).position(|window| window == needle))
        .flatten()
}

fn marker_prefix_suffix_len(buffer: &[u8], marker: &[u8]) -> usize {
    let max_len = buffer.len().min(marker.len().saturating_sub(1));
    (1..=max_len)
        .rev()
        .find(|length| buffer[buffer.len() - length..] == marker[..*length])
        .unwrap_or(0)
}

fn emit_output_line(
    emit: &EmitFn,
    session_id: &str,
    pending: &PendingBoundary,
    kind: OutputStream,
    line: String,
) {
    let run_context = pending.lock().unwrap().as_ref().map(|execution| {
        (
            execution.run_id.clone(),
            execution.emit.clone(),
            execution.log_file.clone(),
            execution.had_output.clone(),
        )
    });
    if let Some((run_id, run_emit, log_file, had_output)) = run_context {
        had_output.fetch_or(!line.trim().is_empty(), Ordering::SeqCst);
        if let Some(log_file) = log_file {
            if let Ok(mut file) = log_file.lock() {
                let _ = writeln!(file, "{line}");
            }
        }
        run_emit.unwrap_or_else(|| emit.clone())(SessionOutputEvent {
            session_id: session_id.to_owned(),
            run_id,
            command: None,
            stream: kind,
            chunk: line,
        });
    } else {
        emit(SessionOutputEvent {
            session_id: session_id.to_owned(),
            run_id: None,
            command: None,
            stream: kind,
            chunk: line,
        });
    }
}

fn mark_boundary(pending: &PendingBoundary, token: &str, exit_code: &str, kind: OutputStream) {
    let sender = {
        let mut guard = pending.lock().unwrap();
        let Some(execution) = guard.as_mut() else {
            return;
        };
        if execution.token != token {
            return;
        }
        execution.exit_code = exit_code.trim().parse().unwrap_or(-1);
        match kind {
            OutputStream::Stdout => execution.stdout_boundary_seen = true,
            OutputStream::Stderr => execution.stderr_boundary_seen = true,
        }
        if execution.stdout_boundary_seen && execution.stderr_boundary_seen {
            guard.take().and_then(|mut execution| execution.sender.take())
        } else {
            None
        }
    };
    if let Some(sender) = sender {
        let _ = sender.send(exit_code.trim().parse().unwrap_or(-1));
    }
}

fn complete_pending_execution(pending: &PendingBoundary, exit_code: i32) {
    let sender = pending
        .lock()
        .unwrap()
        .take()
        .and_then(|mut execution| execution.sender.take());
    if let Some(sender) = sender {
        let _ = sender.send(exit_code);
    }
}

fn open_log_file(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())
}

fn powershell_command_wrapper(command: &str, token: &str, cwd_path: Option<&Path>) -> String {
    let script = powershell_single_quoted_literal(&BASE64_STANDARD.encode(command.as_bytes()));
    let cwd_write = cwd_path.map_or_else(String::new, |path| {
        format!(
            "[System.IO.File]::WriteAllText({}, (Get-Location).Path);",
            powershell_single_quoted_literal(&path.to_string_lossy())
        )
    });
    format!(
        "$__tmExitCode=1; $__tmPreviousErrorActionPreference=$ErrorActionPreference; try {{ $ErrorActionPreference='Stop'; $__tmCommand=[System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String({script})); $__tmScript=[ScriptBlock]::Create($__tmCommand); . $__tmScript; $__tmSucceeded=$?; if ($__tmSucceeded) {{ $__tmExitCode=0 }} elseif ($null -ne $LASTEXITCODE) {{ $__tmExitCode=[int]$LASTEXITCODE }} }} catch {{ [Console]::Error.WriteLine(($_ | Out-String)); $__tmExitCode=1 }} finally {{ $ErrorActionPreference=$__tmPreviousErrorActionPreference; {cwd_write} Write-Output \"{BOUNDARY_PREFIX}{token}:$__tmExitCode\"; [Console]::Error.WriteLine(\"{BOUNDARY_PREFIX}{token}:$__tmExitCode\") }}"
    )
}

fn bash_command_wrapper(command: &str, token: &str, cwd_path: Option<&Path>) -> String {
    let script = bash_single_quoted_literal(command);
    let cwd_write = cwd_path.map_or_else(String::new, |path| {
        format!("pwd > {};", bash_single_quoted_literal(&path.to_string_lossy()))
    });
    format!(
        "eval {script}; __tm_status=$?; {cwd_write} printf '{BOUNDARY_PREFIX}{token}:%s\\n' \"$__tm_status\"; printf '{BOUNDARY_PREFIX}{token}:%s\\n' \"$__tm_status\" >&2"
    )
}

fn powershell_single_quoted_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn bash_single_quoted_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub(crate) fn restart_session_after_stop(
    manager: &SessionManager,
    session_id: &str,
) -> Result<(), String> {
    let Some(mut old_handle) = manager.sessions.lock().unwrap().remove(session_id) else {
        return Ok(());
    };
    let workspace = old_handle.workspace.clone();
    let emit = old_handle.emit.clone();
    complete_pending_execution(&old_handle.pending_boundary, 130);
    terminate_process_tree(&mut old_handle.child);
    let _ = old_handle.child.wait();

    let new_handle = spawn_session_handle(session_id, workspace, emit.clone())?;
    manager
        .sessions
        .lock()
        .unwrap()
        .insert(session_id.to_owned(), new_handle);
    emit(SessionOutputEvent {
        session_id: session_id.to_owned(),
        run_id: None,
        command: None,
        stream: OutputStream::Stdout,
        chunk: "Shell session restarted after stopping the command; session variables were cleared."
            .to_owned(),
    });
    Ok(())
}

fn terminate_process_tree(child: &mut Child) {
    #[cfg(target_os = "windows")]
    {
        let tree_kill_succeeded = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !tree_kill_succeeded {
            let _ = child.kill();
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = child.kill();
    }
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

    #[test]
    fn normalizes_powershell_provider_and_extended_drive_paths() {
        assert_eq!(
            normalize_working_directory(
                r"Microsoft.PowerShell.Core\FileSystem::\\?\D:\Projects\TerminalMate",
            ),
            r"D:\Projects\TerminalMate",
        );
    }

    #[test]
    fn normalizes_extended_unc_paths_without_changing_regular_paths() {
        assert_eq!(
            normalize_working_directory(r"\\?\UNC\server\share\project"),
            r"\\server\share\project",
        );
        assert_eq!(
            normalize_working_directory("/mnt/d/projects/terminal-mate"),
            "/mnt/d/projects/terminal-mate",
        );
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
    fn disables_the_aws_cli_pager_in_terminal_sessions() {
        let manager = SessionManager::default();
        let captured: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let captured_for_emit = captured.clone();
        let emit: EmitFn =
            Arc::new(move |event| captured_for_emit.lock().unwrap().push(event));

        let summary = spawn_session(&manager, emit, windows_test_workspace())
            .expect("a real PowerShell session should start");
        captured.lock().unwrap().clear();

        let run = start_session_run(
            &manager,
            &summary.id,
            "Write-Output \"AWS_PAGER=[$env:AWS_PAGER]\"",
            Some("aws-pager-test-run".to_owned()),
            None,
            None,
        )
        .expect("the environment check should start");
        assert_eq!(
            run.receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
            0
        );
        assert!(captured
            .lock()
            .unwrap()
            .iter()
            .any(|event| event.chunk.contains("AWS_PAGER=[]")));

        terminate_session(&manager, &summary.id).expect("session should close");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn streams_a_partial_prompt_and_sends_input_to_the_running_command() {
        let manager = SessionManager::default();
        let captured: Arc<Mutex<Vec<SessionOutputEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let captured_for_emit = captured.clone();
        let emit: EmitFn = Arc::new(move |event| captured_for_emit.lock().unwrap().push(event));

        let summary = spawn_session(&manager, emit, windows_test_workspace())
            .expect("a real PowerShell session should start on this Windows dev machine");
        captured.lock().unwrap().clear();

        let run = start_session_run(
            &manager,
            &summary.id,
            "[Console]::Write('Continue (Y/n): '); $answer=[Console]::In.ReadLine(); Write-Output \"answer=$answer\"",
            Some("interactive-test-run".to_owned()),
            None,
            None,
        )
        .expect("the interactive command should start");

        let prompt_deadline = Instant::now() + Duration::from_secs(5);
        while !captured
            .lock()
            .unwrap()
            .iter()
            .any(|event| event.chunk.contains("Continue (Y/n):"))
            && Instant::now() < prompt_deadline
        {
            std::thread::sleep(Duration::from_millis(25));
        }
        assert!(
            captured
                .lock()
                .unwrap()
                .iter()
                .any(|event| event.chunk.contains("Continue (Y/n):")),
            "the prompt should stream before PowerShell receives a newline"
        );

        send_input(&manager, &summary.id, "Y")
            .expect("the response should be written to the existing shell stdin");
        assert_eq!(
            run.receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("the interactive command should complete"),
            0
        );

        let answer_deadline = Instant::now() + Duration::from_secs(5);
        while !captured
            .lock()
            .unwrap()
            .iter()
            .any(|event| event.chunk.contains("answer=Y"))
            && Instant::now() < answer_deadline
        {
            std::thread::sleep(Duration::from_millis(25));
        }
        assert!(
            captured
                .lock()
                .unwrap()
                .iter()
                .any(|event| event.chunk.contains("answer=Y")),
            "the running command should receive the response in the same session"
        );

        terminate_session(&manager, &summary.id).expect("closing a live session should succeed");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn closing_an_unknown_session_is_a_harmless_no_op() {
        let manager = SessionManager::default();
        assert!(terminate_session(&manager, "does-not-exist").is_ok());
    }

}
