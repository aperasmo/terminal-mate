use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::commands::policy::classify_command;
use crate::commands::sessions::{
    restart_session_after_stop, start_session_run, SessionManager,
};
use crate::models::command_run::{CommandRunFinishedEvent, CommandRunSummary};
use crate::models::policy::RiskLevel;
use crate::models::session::{OutputStream, SessionOutputEvent};

const OUTPUT_EVENT: &str = "session-output";
const FINISHED_EVENT: &str = "command-finished";

struct ActiveRun {
    stopped_by_user: Arc<AtomicBool>,
}

type ActiveRuns = Arc<Mutex<HashMap<String, ActiveRun>>>;
type EmitOutputFn = Arc<dyn Fn(SessionOutputEvent) + Send + Sync>;
type EmitFinishedFn = Arc<dyn Fn(CommandRunFinishedEvent) + Send + Sync>;

/// Tracks the one foreground command a session is currently running, if any.
/// Every command — quick or long-running — goes through here: it always logs
/// to a file and always reports a completion event, but only long-running
/// commands make that visible for long enough to matter in the UI.
///
/// The map lives behind an `Arc` (not just a `Mutex`) because the waiter
/// thread spawned per run needs to remove its own entry once the process
/// exits, long after the call that started it has returned.
#[derive(Default)]
pub struct RunManager {
    active: ActiveRuns,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandFailureDiagnosis {
    kind: String,
    title: String,
    explanation: String,
    next_steps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggested_request: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failed_subscription_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureSubscription {
    name: String,
    id: String,
    tenant_id: String,
    is_default: bool,
}

#[tauri::command]
pub fn run_command(
    app: AppHandle,
    sessions: State<'_, SessionManager>,
    runs: State<'_, RunManager>,
    session_id: String,
    command: String,
) -> Result<CommandRunSummary, String> {
    let log_root = app
        .path()
        .app_log_dir()
        .map_err(|error| format!("Could not resolve the log directory: {error}"))?
        .join("command-runs");

    let app_for_output = app.clone();
    let emit_output: EmitOutputFn = Arc::new(move |event| {
        let _ = app_for_output.emit(OUTPUT_EVENT, event);
    });
    let emit_finished: EmitFinishedFn = Arc::new(move |event| {
        let _ = app.emit(FINISHED_EVENT, event);
    });

    start_run(
        &sessions,
        runs.inner(),
        &log_root,
        emit_output,
        emit_finished,
        session_id,
        command,
    )
}

#[tauri::command]
pub fn stop_command(
    sessions: State<'_, SessionManager>,
    runs: State<'_, RunManager>,
    session_id: String,
) -> Result<(), String> {
    stop_run(&sessions, runs.inner(), &session_id)
}

#[tauri::command]
pub fn open_log_file(path: String) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Failed to open the log file: {error}"))
}

#[tauri::command]
pub fn read_log_file(app: AppHandle, path: String) -> Result<String, String> {
    let canonical_path = terminal_mate_log_path(&app, &path)?;
    read_complete_log(&canonical_path)
}

#[tauri::command]
pub fn diagnose_command_failure(
    app: AppHandle,
    path: String,
    command: String,
    exit_code: i32,
) -> Result<CommandFailureDiagnosis, String> {
    let canonical_path = terminal_mate_log_path(&app, &path)?;
    let output = read_log_tail(&canonical_path, 64 * 1024)?;
    let diagnosis = diagnose_failure(&command, exit_code, &output);

    // "nonzeroExit" is the fallback used when nothing more specific
    // matched — exactly the failures worth reviewing later to notice a new
    // pattern (like `docker exec -it` needing a PTY TerminalMate does not
    // have, the same root cause already fixed for ssh/nano/terraform, just
    // not yet recognized here) and turn it into its own diagnosis or
    // policy guardrail.
    if diagnosis.kind == "nonzeroExit" {
        if let Ok(log_dir) = app.path().app_log_dir() {
            record_diagnostic_gap(&log_dir, &command, exit_code, &output);
        }
    }

    Ok(diagnosis)
}

/// Best-effort, append-only record of every command failure with no
/// specific diagnosis — one JSON object per line in `diagnostic-gaps.log`
/// alongside the regular per-run logs. Never fails the actual diagnosis if
/// logging itself cannot succeed (missing directory, IO error, ...): this
/// is a convenience for spotting patterns later, not something a single
/// command's success should ever depend on.
fn record_diagnostic_gap(log_dir: &Path, command: &str, exit_code: i32, output: &str) {
    if fs::create_dir_all(log_dir).is_err() {
        return;
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DiagnosticGapEntry<'a> {
        timestamp_ms: u64,
        command: &'a str,
        exit_code: i32,
        snippet: &'a str,
    }

    let snippet = tail_snippet(output, 500);
    let entry = DiagnosticGapEntry {
        timestamp_ms: unix_epoch_millis(),
        command,
        exit_code,
        snippet: &snippet,
    };

    let Ok(mut line) = serde_json::to_string(&entry) else {
        return;
    };
    line.push('\n');

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("diagnostic-gaps.log"))
    {
        let _ = file.write_all(line.as_bytes());
    }
}

/// The last `max_chars` characters of `output`, trimmed — enough to scan
/// `diagnostic-gaps.log` at a glance without re-opening each run's own,
/// much larger, per-command log file. Slices on a char boundary, since
/// `output` is arbitrary command output and may contain multi-byte UTF-8.
fn tail_snippet(output: &str, max_chars: usize) -> String {
    let trimmed = output.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_owned();
    }

    let start = trimmed
        .char_indices()
        .rev()
        .nth(max_chars - 1)
        .map(|(index, _)| index)
        .unwrap_or(0);
    trimmed[start..].to_owned()
}

#[tauri::command]
pub fn read_azure_subscriptions(
    app: AppHandle,
    path: String,
) -> Result<Vec<AzureSubscription>, String> {
    let canonical_path = terminal_mate_log_path(&app, &path)?;
    let output = read_log_tail(&canonical_path, 256 * 1024)?;
    parse_azure_subscriptions(&output)
}

/// The result of a successful `terraform apply` that created (or recreated)
/// an Azure VM — the moment a guided lifecycle decision (leave it running,
/// deallocate it, or destroy it) actually matters, since before this the
/// resource does not exist yet to have an opinion about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedVirtualMachine {
    vm_name: String,
    resource_group: String,
}

#[tauri::command]
pub fn detect_created_virtual_machine(
    app: AppHandle,
    path: String,
) -> Result<Option<CreatedVirtualMachine>, String> {
    let canonical_path = terminal_mate_log_path(&app, &path)?;
    let output = read_log_tail(&canonical_path, 256 * 1024)?;
    Ok(find_created_virtual_machine(&output))
}

fn find_created_virtual_machine(output: &str) -> Option<CreatedVirtualMachine> {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        Regex::new(r"(?i)resourceGroups/([^/\s]+)/providers/Microsoft\.Compute/virtualMachines/([^/\s\]]+)")
            .expect("Azure VM resource-ID pattern should compile")
    });
    let captures = pattern.captures(output)?;
    Some(CreatedVirtualMachine {
        resource_group: captures.get(1)?.as_str().to_owned(),
        vm_name: captures.get(2)?.as_str().to_owned(),
    })
}

fn terminal_mate_log_path(app: &AppHandle, path: &str) -> Result<PathBuf, String> {
    let log_root = app
        .path()
        .app_log_dir()
        .map_err(|error| format!("Could not resolve the log directory: {error}"))?
        .join("command-runs");
    let canonical_root = log_root
        .canonicalize()
        .map_err(|_| "The command log directory is not available.".to_owned())?;
    let canonical_path = PathBuf::from(path)
        .canonicalize()
        .map_err(|_| "The command log is not available.".to_owned())?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err("Only TerminalMate command logs can be read.".to_owned());
    }

    Ok(canonical_path)
}

fn parse_azure_subscriptions(output: &str) -> Result<Vec<AzureSubscription>, String> {
    extract_json_array(output, "subscription list")
}

/// Shared by every Azure CLI list command TerminalMate parses (subscriptions,
/// virtual machines, ...): finds the outermost `[...]` in the command's
/// output — skipping any login/status text Azure prints before the actual
/// JSON — and deserializes it. Azure CLI's `--output json` always ends with
/// the JSON itself as the last thing printed, so the outermost brackets are
/// reliably the array TerminalMate wants, not some inner list among the
/// row's own fields.
fn extract_json_array<T: serde::de::DeserializeOwned>(
    output: &str,
    what: &str,
) -> Result<Vec<T>, String> {
    let json_start = output
        .find('[')
        .ok_or_else(|| format!("Azure did not return a {what}."))?;
    let json_end = output
        .rfind(']')
        .ok_or_else(|| format!("Azure returned an incomplete {what}."))?;
    serde_json::from_str(&output[json_start..=json_end])
        .map_err(|error| format!("Azure returned an invalid {what}: {error}"))
}

/// An Azure virtual machine surfaced by
/// `az vm list --show-details --output json`, enough to disambiguate and
/// then start or deallocate it (`resourceGroup`/`powerState` are the exact
/// field names Azure's own JSON already uses, so no rename is needed beyond
/// `rename_all`'s usual camelCase mapping). `powerState` only appears with
/// `--show-details` (an extra per-VM call Azure CLI makes), which is worth
/// the cost here since it lets the picker show which VMs are already
/// running vs. deallocated instead of leaving that to guesswork.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureVirtualMachine {
    name: String,
    resource_group: String,
    #[serde(default)]
    power_state: Option<String>,
}

#[tauri::command]
pub fn read_azure_virtual_machines(
    app: AppHandle,
    path: String,
) -> Result<Vec<AzureVirtualMachine>, String> {
    let canonical_path = terminal_mate_log_path(&app, &path)?;
    let output = read_log_tail(&canonical_path, 256 * 1024)?;
    parse_azure_virtual_machines(&output)
}

fn parse_azure_virtual_machines(output: &str) -> Result<Vec<AzureVirtualMachine>, String> {
    extract_json_array(output, "virtual machine list")
}

fn read_log_tail(path: &Path, max_bytes: u64) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("Could not read the command log: {error}"))?;
    let length = file
        .metadata()
        .map_err(|error| format!("Could not inspect the command log: {error}"))?
        .len();
    let start = length.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))
        .map_err(|error| format!("Could not seek within the command log: {error}"))?;

    let mut bytes = Vec::with_capacity((length - start) as usize);
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read the command log: {error}"))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_complete_log(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|error| format!("Could not read the command log: {error}"))
}

fn diagnose_failure(command: &str, exit_code: i32, output: &str) -> CommandFailureDiagnosis {
    let normalized = output.to_ascii_lowercase();

    if is_recursive_powershell_search(command)
        && (normalized.contains("dirunauthorizedaccesserror")
            || normalized.contains("unauthorizedaccessexception")
            || normalized.contains("access to the path") && normalized.contains("is denied"))
    {
        return CommandFailureDiagnosis {
            kind: "recursiveSearchAccessDenied".to_owned(),
            title: "A protected folder stopped the search".to_owned(),
            explanation: "The recursive read reached a folder that the current account cannot open. TerminalMate can retry the same search across accessible folders while leaving protected folders untouched.".to_owned(),
            next_steps: vec![
                "Review the corrected command, which skips inaccessible folders during this read-only search.".to_owned(),
                "Run it to return matches from every folder the current account can read.".to_owned(),
            ],
            suggested_request: suggest_skip_inaccessible_folders(command),
            failed_subscription_id: None,
        };
    }

    if normalized.contains("randomnumbergenerator")
        && normalized.contains("does not contain a method named")
        && (normalized.contains("'fill'") || command.contains("::Fill"))
    {
        return CommandFailureDiagnosis {
            kind: "powershellCryptoApiUnavailable".to_owned(),
            title: "RandomNumberGenerator.Fill is unavailable in this PowerShell runtime"
                .to_owned(),
            explanation: "Windows PowerShell 5.1 uses a .NET runtime that does not provide the static RandomNumberGenerator.Fill method. The byte buffer is valid; use the compatible Create().GetBytes(...) API instead."
                .to_owned(),
            next_steps: vec![
                "Review the compatible password-generation command before running it."
                    .to_owned(),
                "Keep the related variable assignments together so they execute in one PowerShell scope."
                    .to_owned(),
            ],
            suggested_request: Some(
                "$bytes = New-Object byte[] 32; $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create(); try { $rng.GetBytes($bytes) } finally { $rng.Dispose() }; $password = [Convert]::ToBase64String($bytes).TrimEnd('=').Replace('+','-').Replace('/','_'); $password"
                    .to_owned(),
            ),
            failed_subscription_id: None,
        };
    }

    if normalized.contains("azure login required")
        || normalized.contains("please run 'az login'")
        || normalized.contains("az login to setup account")
        || normalized.contains("authentication needed") && command.contains("az ")
    {
        return CommandFailureDiagnosis {
            kind: "azureLoginRequired".to_owned(),
            title: "Azure sign-in is required".to_owned(),
            explanation: "The Azure CLI is available, but the active workspace does not have an authenticated Azure session.".to_owned(),
            next_steps: vec![
                "Run the guided request `Sign in to Azure` or use the device-code login flow.".to_owned(),
                "Confirm the selected subscription before retrying the infrastructure command.".to_owned(),
            ],
            suggested_request: Some("Sign in to Azure with device code".to_owned()),
            failed_subscription_id: None,
        };
    }

    if command.trim_start().starts_with("az ")
        && (normalized.contains("subscriptionnotfound")
            || normalized.contains("subscription not found"))
    {
        return CommandFailureDiagnosis {
            kind: "azureSubscriptionNotFound".to_owned(),
            title: "Azure subscription is not available".to_owned(),
            explanation: "Azure CLI is signed in, but the selected subscription is not available to the active account or tenant.".to_owned(),
            next_steps: vec![
                "List the subscriptions available to the current Azure account.".to_owned(),
                "Select an accessible subscription, then retry the original command.".to_owned(),
                "If the expected subscription is missing, sign in with the account or tenant that owns it."
                    .to_owned(),
            ],
            suggested_request: Some("List Azure subscriptions".to_owned()),
            failed_subscription_id: azure_subscription_id(output),
        };
    }

    if command.trim_start().starts_with("scp")
        && (normalized.contains("realpath") || normalized.contains("path canonicalization failed"))
    {
        return CommandFailureDiagnosis {
            kind: "scpRemoteDirectoryMissing".to_owned(),
            title: "Remote destination folder does not exist".to_owned(),
            explanation: "scp could not resolve the destination path on the remote host. Its \
                upload protocol requires the destination directory to already exist — scp will \
                not create missing folders for you, even with -r."
                .to_owned(),
            next_steps: vec![
                "Create the destination folder on the remote host first.".to_owned(),
                "Then retry the original scp upload.".to_owned(),
            ],
            suggested_request: suggest_remote_mkdir_command(command),
            failed_subscription_id: None,
        };
    }

    if normalized.contains("cannot find path")
        || normalized.contains("does not exist")
        || normalized.contains("pathnotfound")
        || normalized.contains("itemnotfound")
    {
        return CommandFailureDiagnosis {
            kind: "missingPath".to_owned(),
            title: "File or folder not found".to_owned(),
            explanation: "The command referenced a path that is not available from the current working directory.".to_owned(),
            next_steps: vec![
                "Check the spelling and confirm which workspace is active.".to_owned(),
                "List the current folder before trying the request again.".to_owned(),
            ],
            suggested_request: Some("List files in this folder".to_owned()),
            failed_subscription_id: None,
        };
    }

    if normalized.contains("is not recognized as the name")
        || normalized.contains("commandnotfoundexception")
        || normalized.contains("the term '")
    {
        let executable = command.split_whitespace().next().unwrap_or("command");
        return CommandFailureDiagnosis {
            kind: "commandUnavailable".to_owned(),
            title: "Command is not available".to_owned(),
            explanation: format!(
                "The active shell could not find `{executable}` as an installed command, script, or executable."
            ),
            next_steps: vec![
                "Check that the required tool is installed.".to_owned(),
                "Confirm its installation folder is included in PATH, then restart TerminalMate."
                    .to_owned(),
            ],
            suggested_request: None,
            failed_subscription_id: None,
        };
    }

    if normalized.contains("access is denied")
        || normalized.contains("unauthorizedaccessexception")
        || normalized.contains("permission denied")
    {
        return CommandFailureDiagnosis {
            kind: "permissionDenied".to_owned(),
            title: "Permission was denied".to_owned(),
            explanation: "The operating system blocked access to a file, folder, process, or operation.".to_owned(),
            next_steps: vec![
                "Confirm the active account should have access to the target.".to_owned(),
                "Prefer a user-owned location; only elevate privileges when the operation is understood."
                    .to_owned(),
            ],
            suggested_request: None,
            failed_subscription_id: None,
        };
    }

    if normalized.contains("could not resolve host")
        || normalized.contains("name or service not known")
        || normalized.contains("timed out")
        || normalized.contains("network is unreachable")
    {
        return CommandFailureDiagnosis {
            kind: "networkFailure".to_owned(),
            title: "Network request could not connect".to_owned(),
            explanation: "The destination could not be resolved or reached before the command failed.".to_owned(),
            next_steps: vec![
                "Check the hostname and the machine's network connection.".to_owned(),
                "Retry after confirming that any VPN, proxy, or required service is available."
                    .to_owned(),
            ],
            suggested_request: None,
            failed_subscription_id: None,
        };
    }

    CommandFailureDiagnosis {
        kind: "nonzeroExit".to_owned(),
        title: "Command exited with an error".to_owned(),
        explanation: format!(
            "The command returned exit code {exit_code}. TerminalMate could not identify a more specific known failure."
        ),
        next_steps: vec![
            "Read the final error lines above or open the complete command log.".to_owned(),
            "Correct the command or its inputs, then run it again.".to_owned(),
        ],
        suggested_request: None,
        failed_subscription_id: None,
    }
}

fn is_recursive_powershell_search(command: &str) -> bool {
    let normalized = command.to_ascii_lowercase();
    normalized.contains("get-childitem") && normalized.contains("-recurse")
}

fn suggest_skip_inaccessible_folders(command: &str) -> Option<String> {
    if !is_recursive_powershell_search(command)
        || command.to_ascii_lowercase().contains("-erroraction")
    {
        return None;
    }

    let insertion_index = command.find('|').unwrap_or(command.len());
    let (search, remainder) = command.split_at(insertion_index);
    Some(format!(
        "{} -ErrorAction SilentlyContinue{}",
        search.trim_end(),
        if remainder.is_empty() {
            String::new()
        } else {
            format!(" {remainder}")
        }
    ))
}

/// Best-effort: reconstructs an `ssh ... mkdir -p <remote-dir>` suggestion
/// from the failed scp command's `-i <key>` flag and `user@host:path`
/// destination, so the fix is one click (well, one Enter) away instead of
/// just described. Included flags mirror `intent_adapter`'s `ssh_vm`
/// render: BatchMode/StrictHostKeyChecking avoid hanging on a prompt this
/// one-shot remote command couldn't answer either.
fn suggest_remote_mkdir_command(scp_command: &str) -> Option<String> {
    let identity = Regex::new(r"-i\s+(\S+)")
        .unwrap()
        .captures(scp_command)
        .map(|captures| captures[1].to_owned());
    let destination = Regex::new(r"([\w.-]+@[\w.-]+):(\S+)")
        .unwrap()
        .captures(scp_command)?;
    let host = destination.get(1)?.as_str();
    let remote_path = destination.get(2)?.as_str().trim_end_matches('/');
    let identity_flag = identity
        .map(|key| format!("-i {key} "))
        .unwrap_or_default();

    Some(format!(
        "ssh {identity_flag}-o BatchMode=yes -o StrictHostKeyChecking=accept-new {host} \"mkdir -p {remote_path}\""
    ))
}

fn azure_subscription_id(output: &str) -> Option<String> {
    let marker = "subscription ";
    let start = output.to_ascii_lowercase().find(marker)? + marker.len();
    let candidate = output[start..]
        .split_whitespace()
        .next()?
        .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '-');

    (!candidate.is_empty()).then(|| candidate.to_owned())
}

fn start_run(
    sessions: &SessionManager,
    runs: &RunManager,
    log_root: &Path,
    emit_output: EmitOutputFn,
    emit_finished: EmitFinishedFn,
    session_id: String,
    command: String,
) -> Result<CommandRunSummary, String> {
    let decision = classify_command(&command);
    if decision.level == RiskLevel::Blocked {
        return Err(format!("Blocked: {}", decision.reason));
    }

    {
        let active = runs.active.lock().unwrap();
        if active.contains_key(&session_id) {
            return Err("Another command is already running in this session.".to_owned());
        }
    }

    let run_id = Uuid::new_v4().to_string();
    let started_at_ms = unix_epoch_millis();
    let log_path = log_root.join(&session_id).join(format!("{run_id}.log"));
    let working_directory_path = log_root
        .join(&session_id)
        .join(format!("{run_id}.cwd"));
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&log_path, Vec::new()).map_err(|error| error.to_string())?;

    emit_output(SessionOutputEvent {
        session_id: session_id.clone(),
        run_id: Some(run_id.clone()),
        command: Some(command.clone()),
        stream: OutputStream::Stdout,
        chunk: String::new(),
    });

    let session_run = start_session_run(
        sessions,
        &session_id,
        &command,
        Some(run_id.clone()),
        Some(log_path.clone()),
        Some(emit_output.clone()),
    )?;

    let stopped_by_user = Arc::new(AtomicBool::new(false));

    runs.active.lock().unwrap().insert(
        session_id.clone(),
        ActiveRun {
            stopped_by_user: stopped_by_user.clone(),
        },
    );

    spawn_session_waiter(
        runs.active.clone(),
        emit_output,
        emit_finished,
        command.clone(),
        run_id.clone(),
        session_id.clone(),
        log_path.clone(),
        working_directory_path,
        stopped_by_user,
        sessions.clone(),
        session_run.receiver,
        session_run.had_output,
        started_at_ms,
    );

    Ok(CommandRunSummary {
        id: run_id,
        session_id,
        command,
        log_path: log_path.to_string_lossy().into_owned(),
        started_at_ms,
    })
}

fn stop_run(
    sessions: &SessionManager,
    runs: &RunManager,
    session_id: &str,
) -> Result<(), String> {
    let active = runs.active.lock().unwrap();
    let Some(run) = active.get(session_id) else {
        return Ok(());
    };

    run.stopped_by_user.store(true, Ordering::SeqCst);
    drop(active);
    restart_session_after_stop(sessions, session_id)
}

/// TerminalMate streams plain text, not a real terminal (no PTY), so ANSI
/// escape sequences that color-aware tools like Terraform, git, npm, and
/// cargo emit would otherwise show up as literal garbage (`[0m[1m...`)
/// instead of being interpreted as color/formatting. Strip them before a
/// line ever reaches the log file or the display.
pub(crate) fn strip_ansi_escapes(text: &str) -> std::borrow::Cow<'_, str> {
    static ANSI_PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = ANSI_PATTERN
        .get_or_init(|| Regex::new(r"\x1b\[[0-?]*[ -/]*[@-~]").expect("ANSI escape pattern should compile"));
    pattern.replace_all(text, "")
}

pub(crate) fn decode_output_line(mut bytes: Vec<u8>) -> String {
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }

    match String::from_utf8(bytes) {
        Ok(line) => line,
        Err(error) => error
            .into_bytes()
            .into_iter()
            .map(|byte| match byte {
                0x80 => '\u{20ac}',
                0x82 => '\u{201a}',
                0x83 => '\u{0192}',
                0x84 => '\u{201e}',
                0x85 => '\u{2026}',
                0x86 => '\u{2020}',
                0x87 => '\u{2021}',
                0x88 => '\u{02c6}',
                0x89 => '\u{2030}',
                0x8a => '\u{0160}',
                0x8b => '\u{2039}',
                0x8c => '\u{0152}',
                0x8e => '\u{017d}',
                0x91 => '\u{2018}',
                0x92 => '\u{2019}',
                0x93 => '\u{201c}',
                0x94 => '\u{201d}',
                0x95 => '\u{2022}',
                0x96 => '\u{2013}',
                0x97 => '\u{2014}',
                0x98 => '\u{02dc}',
                0x99 => '\u{2122}',
                0x9a => '\u{0161}',
                0x9b => '\u{203a}',
                0x9c => '\u{0153}',
                0x9e => '\u{017e}',
                0x9f => '\u{0178}',
                _ => char::from(byte),
            })
            .collect(),
    }
}

fn spawn_session_waiter(
    active_runs: ActiveRuns,
    emit_output: EmitOutputFn,
    emit_finished: EmitFinishedFn,
    command: String,
    run_id: String,
    session_id: String,
    log_path: PathBuf,
    working_directory_path: PathBuf,
    stopped_by_user: Arc<AtomicBool>,
    sessions: SessionManager,
    receiver: Receiver<i32>,
    had_output: Arc<AtomicBool>,
    started_at_ms: u64,
) {
    std::thread::spawn(move || {
        let mut exit_code = receiver.recv().unwrap_or(-1);
        active_runs.lock().unwrap().remove(&session_id);

        let finished_at_ms = unix_epoch_millis();
        let duration_ms = finished_at_ms.saturating_sub(started_at_ms);
        let produced_output = had_output.load(Ordering::SeqCst);
        if !produced_output {
            let empty_output_message = if !stopped_by_user.load(Ordering::SeqCst)
                && is_git_check_ignore_not_ignored(&command, exit_code, produced_output)
            {
                exit_code = 0;
                "Path is not ignored by Git."
            } else {
                "No output returned."
            };
            if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = writeln!(log_file, "{empty_output_message}");
            }
            emit_output(SessionOutputEvent {
                session_id: session_id.clone(),
                run_id: Some(run_id.clone()),
                command: None,
                stream: OutputStream::Stdout,
                chunk: empty_output_message.to_owned(),
            });
        }

        let mut working_directory = fs::read_to_string(&working_directory_path)
            .ok()
            .map(|path| path.trim_start_matches('\u{feff}').trim().to_owned())
            .filter(|path| !path.is_empty())
            .or_else(|| sessions.working_directory(&session_id))
            .unwrap_or_default();
        let _ = fs::remove_file(&working_directory_path);
        if !working_directory.is_empty() {
            let _ = sessions.update_working_directory(&session_id, working_directory.clone());
            working_directory = sessions
                .working_directory(&session_id)
                .unwrap_or(working_directory);
        }

        emit_finished(CommandRunFinishedEvent {
            id: run_id,
            session_id,
            exit_code,
            success: exit_code == 0,
            stopped_by_user: stopped_by_user.load(Ordering::SeqCst),
            log_path: log_path.to_string_lossy().into_owned(),
            working_directory,
            started_at_ms,
            finished_at_ms,
            duration_ms,
        });
    });
}

fn is_git_check_ignore_not_ignored(command: &str, exit_code: i32, had_output: bool) -> bool {
    let normalized = command.trim_start().to_ascii_lowercase();
    exit_code == 1
        && !had_output
        && (normalized == "git check-ignore" || normalized.starts_with("git check-ignore "))
}

fn unix_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::policy::RiskLevel as PolicyRiskLevel;
    use crate::models::workspace::{ExecutionProfile, Workspace};
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};

    #[cfg(target_os = "windows")]
    struct ClipboardGuard {
        backup_path: PathBuf,
    }

    #[cfg(target_os = "windows")]
    impl ClipboardGuard {
        fn replace_with(value: &str) -> Self {
            let backup_path = std::env::temp_dir()
                .join(format!("terminal-mate-clipboard-{}.txt", Uuid::new_v4()));
            let output = Command::new("powershell.exe")
                .args([
                    "-NoLogo",
                    "-NoProfile",
                    "-Command",
                    "$raw=[string](Get-Clipboard -Raw); [IO.File]::WriteAllText($env:TM_CLIPBOARD_BACKUP,[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($raw))); Set-Clipboard -Value $env:TM_CLIPBOARD_TEST_VALUE",
                ])
                .env("TM_CLIPBOARD_BACKUP", &backup_path)
                .env("TM_CLIPBOARD_TEST_VALUE", value)
                .output()
                .expect("the Windows clipboard fixture should start");
            assert!(
                output.status.success(),
                "the Windows clipboard fixture should set harmless test text"
            );
            Self { backup_path }
        }
    }

    #[cfg(target_os = "windows")]
    impl Drop for ClipboardGuard {
        fn drop(&mut self) {
            let _ = Command::new("powershell.exe")
                .args([
                    "-NoLogo",
                    "-NoProfile",
                    "-Command",
                    "$encoded=[IO.File]::ReadAllText($env:TM_CLIPBOARD_BACKUP); $raw=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($encoded)); Set-Clipboard -Value $raw; Remove-Item -LiteralPath $env:TM_CLIPBOARD_BACKUP -Force",
                ])
                .env("TM_CLIPBOARD_BACKUP", &self.backup_path)
                .output();
        }
    }

    #[test]
    fn strips_ansi_color_codes_from_terraform_style_output() {
        let line = "\u{1b}[0m\u{1b}[1mInitializing the backend...\u{1b}[0m";
        assert_eq!(strip_ansi_escapes(line), "Initializing the backend...");

        let bold_green =
            "\u{1b}[1m\u{1b}[32mTerraform has been successfully initialized!\u{1b}[0m\u{1b}[32m\u{1b}[0m";
        assert_eq!(
            strip_ansi_escapes(bold_green),
            "Terraform has been successfully initialized!"
        );

        assert_eq!(strip_ansi_escapes("plain output, no escapes"), "plain output, no escapes");
    }

    #[test]
    fn records_a_diagnostic_gap_entry_as_a_json_line() {
        let dir = std::env::temp_dir().join(format!("tm-test-diag-gaps-{}", Uuid::new_v4()));

        record_diagnostic_gap(
            &dir,
            "docker exec -it glaucoma_backend pip show slowapi",
            1,
            "cannot attach stdin to a TTY-enabled container because stdin is not a terminal",
        );

        let logged = fs::read_to_string(dir.join("diagnostic-gaps.log"))
            .expect("the gap log file should have been created");
        assert!(logged.contains("\"exitCode\":1"));
        assert!(logged.contains("docker exec -it glaucoma_backend pip show slowapi"));
        assert!(logged.contains("cannot attach stdin to a TTY-enabled container"));
        assert_eq!(logged.lines().count(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn appends_multiple_diagnostic_gap_entries_on_separate_lines() {
        let dir = std::env::temp_dir().join(format!("tm-test-diag-gaps-{}", Uuid::new_v4()));

        record_diagnostic_gap(&dir, "first-command", 1, "first output");
        record_diagnostic_gap(&dir, "second-command", 2, "second output");

        let logged = fs::read_to_string(dir.join("diagnostic-gaps.log")).unwrap();
        assert_eq!(logged.lines().count(), 2);
        assert!(logged.lines().next().unwrap().contains("first-command"));
        assert!(logged.lines().nth(1).unwrap().contains("second-command"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn tail_snippet_keeps_only_the_last_n_characters() {
        let long_output = "a".repeat(600);
        let snippet = tail_snippet(&long_output, 500);
        assert_eq!(snippet.len(), 500);
    }

    #[test]
    fn tail_snippet_returns_the_whole_trimmed_string_when_short_enough() {
        assert_eq!(tail_snippet("  short output  \n", 500), "short output");
    }

    #[test]
    fn reads_the_complete_command_log_without_truncating_long_lines() {
        let dir = std::env::temp_dir().join(format!("tm-test-copy-log-{}", Uuid::new_v4()));
        let path = dir.join("long-output.log");
        let output = format!("start\n{}\nend\n", "x".repeat(50_000));

        fs::create_dir_all(&dir).expect("the test log directory should be created");
        fs::write(&path, output.as_bytes()).expect("the test log should be written");

        let copied = read_complete_log(&path).expect("the complete log should be readable");
        assert_eq!(copied, output);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diagnoses_a_missing_path_with_a_safe_recovery_request() {
        let diagnosis = diagnose_failure(
            "Get-Content -LiteralPath 'missing.txt'",
            1,
            "Get-Content : Cannot find path 'missing.txt' because it does not exist.",
        );

        assert_eq!(diagnosis.kind, "missingPath");
        assert_eq!(diagnosis.title, "File or folder not found");
        assert_eq!(
            diagnosis.suggested_request.as_deref(),
            Some("List files in this folder")
        );
    }

    #[test]
    fn diagnoses_an_unavailable_command_without_suggesting_execution() {
        let diagnosis = diagnose_failure(
            "missing-tool --version",
            1,
            "The term 'missing-tool' is not recognized as the name of a cmdlet.",
        );

        assert_eq!(diagnosis.kind, "commandUnavailable");
        assert!(diagnosis.explanation.contains("missing-tool"));
        assert_eq!(diagnosis.suggested_request, None);
    }

    #[test]
    fn diagnoses_random_number_generator_fill_on_windows_powershell() {
        let diagnosis = diagnose_failure(
            "[System.Security.Cryptography.RandomNumberGenerator]::Fill($bytes)",
            1,
            "Method invocation failed because [System.Security.Cryptography.RandomNumberGenerator] does not contain a method named 'Fill'.",
        );

        assert_eq!(diagnosis.kind, "powershellCryptoApiUnavailable");
        assert!(diagnosis.title.contains("Fill is unavailable"));
        let suggestion = diagnosis
            .suggested_request
            .expect("a compatible command should be suggested");
        assert!(suggestion.contains("::Create()"));
        assert!(suggestion.contains("GetBytes($bytes)"));
    }

    #[test]
    fn treats_git_check_ignore_exit_one_without_output_as_not_ignored() {
        assert!(is_git_check_ignore_not_ignored(
            "git check-ignore .env.production.example",
            1,
            false,
        ));
        assert!(!is_git_check_ignore_not_ignored(
            "git check-ignore .env.production.example",
            1,
            true,
        ));
        assert!(!is_git_check_ignore_not_ignored("git status", 1, false));
    }

    #[test]
    fn diagnoses_recursive_search_access_denied_with_a_safe_retry() {
        let command = "Get-ChildItem -Recurse -File | Where-Object { $_.Name -match \"categor|taxonom\" } | Select-Object FullName";
        let diagnosis = diagnose_failure(
            command,
            1,
            "Get-ChildItem : Access to the path 'D:\\repo\\.pytest_cache' is denied.\nFullyQualifiedErrorId : DirUnauthorizedAccessError,Microsoft.PowerShell.Commands.GetChildItemCommand",
        );

        assert_eq!(diagnosis.kind, "recursiveSearchAccessDenied");
        assert_eq!(
            diagnosis.suggested_request.as_deref(),
            Some("Get-ChildItem -Recurse -File -ErrorAction SilentlyContinue | Where-Object { $_.Name -match \"categor|taxonom\" } | Select-Object FullName")
        );
    }

    #[test]
    fn diagnoses_an_unavailable_azure_subscription() {
        let diagnosis = diagnose_failure(
            "az storage account create --name example --resource-group example-rg --location australiaeast --sku Standard_LRS --encryption-services blob",
            3,
            "ERROR: (SubscriptionNotFound) Subscription d4a4e2c5-4f66-49e9-ae94-d140c3693703 was not found.\nCode: SubscriptionNotFound",
        );

        assert_eq!(diagnosis.kind, "azureSubscriptionNotFound");
        assert_eq!(diagnosis.title, "Azure subscription is not available");
        assert_eq!(
            diagnosis.failed_subscription_id.as_deref(),
            Some("d4a4e2c5-4f66-49e9-ae94-d140c3693703")
        );
        assert_eq!(
            diagnosis.suggested_request.as_deref(),
            Some("List Azure subscriptions")
        );
    }

    #[test]
    fn parses_structured_azure_subscriptions_after_cli_notices() {
        let subscriptions = parse_azure_subscriptions(
            "Checking Azure account...\n[\n  {\n    \"name\": \"Development\",\n    \"id\": \"subscription-one\",\n    \"tenantId\": \"tenant-one\",\n    \"isDefault\": true\n  },\n  {\n    \"name\": \"Production\",\n    \"id\": \"subscription-two\",\n    \"tenantId\": \"tenant-two\",\n    \"isDefault\": false\n  }\n]\n",
        )
        .expect("valid Azure JSON should be parsed");

        assert_eq!(subscriptions.len(), 2);
        assert_eq!(subscriptions[0].name, "Development");
        assert!(subscriptions[0].is_default);
        assert_eq!(subscriptions[1].tenant_id, "tenant-two");
    }

    #[test]
    fn parses_azure_virtual_machines_from_az_vm_list_json_output() {
        let vms = parse_azure_virtual_machines(
            "[\n  {\n    \"name\": \"glaucoma-ai-azure-vm\",\n    \"resourceGroup\": \"glaucoma-ai-learning-rg\",\n    \"location\": \"australiaeast\",\n    \"powerState\": \"VM running\"\n  }\n]\n",
        )
        .expect("valid Azure VM JSON should be parsed");

        assert_eq!(vms.len(), 1);
        assert_eq!(vms[0].name, "glaucoma-ai-azure-vm");
        assert_eq!(vms[0].resource_group, "glaucoma-ai-learning-rg");
        assert_eq!(vms[0].power_state.as_deref(), Some("VM running"));
    }

    #[test]
    fn parses_azure_virtual_machines_without_power_state_when_show_details_was_not_used() {
        let vms = parse_azure_virtual_machines(
            "[\n  {\n    \"name\": \"glaucoma-ai-azure-vm\",\n    \"resourceGroup\": \"glaucoma-ai-learning-rg\"\n  }\n]\n",
        )
        .expect("valid Azure VM JSON should be parsed even without --show-details");

        assert_eq!(vms[0].power_state, None);
    }

    #[test]
    fn parses_an_empty_virtual_machine_list() {
        let vms = parse_azure_virtual_machines("[]\n").expect("an empty list is still valid JSON");
        assert!(vms.is_empty());
    }

    #[test]
    fn detects_a_created_virtual_machine_from_a_successful_terraform_apply_log() {
        let log = "azurerm_linux_virtual_machine.main: Still creating... [00m51s elapsed]\n\
                    azurerm_linux_virtual_machine.main: Creation complete after 53s [id=/subscriptions/d4a4e2c5-4f66-49e9-ae94-d140c3693703/resourceGroups/glaucoma-ai-learning-rg/providers/Microsoft.Compute/virtualMachines/glaucoma-ai-azure-vm]\n\
                    Apply complete! Resources: 1 added, 0 changed, 0 destroyed.\n";

        let detected = find_created_virtual_machine(log).expect("a VM should be detected");
        assert_eq!(detected.vm_name, "glaucoma-ai-azure-vm");
        assert_eq!(detected.resource_group, "glaucoma-ai-learning-rg");
    }

    #[test]
    fn does_not_detect_a_vm_when_apply_only_touched_unrelated_resources() {
        let log = "azurerm_resource_group.main: Creation complete after 2s [id=/subscriptions/example/resourceGroups/demo-rg]\n\
                    Apply complete! Resources: 1 added, 0 changed, 0 destroyed.\n";

        assert_eq!(find_created_virtual_machine(log), None);
    }

    #[test]
    fn diagnoses_an_scp_upload_to_a_missing_remote_directory() {
        let diagnosis = diagnose_failure(
            "scp -i ~/glaucoma-ai-azure-key -r backend/models/ azureuser@20.213.57.180:~/glaucoma-detection/backend/",
            1,
            "scp: realpath glaucoma-detection/backend/: No such file\n\
             scp: upload \"glaucoma-detection/backend/\": path canonicalization failed\n\
             scp: failed to upload directory backend/models to ~/glaucoma-detection/backend/",
        );

        assert_eq!(diagnosis.kind, "scpRemoteDirectoryMissing");
        assert_eq!(
            diagnosis.suggested_request.as_deref(),
            Some(
                "ssh -i ~/glaucoma-ai-azure-key -o BatchMode=yes -o StrictHostKeyChecking=accept-new azureuser@20.213.57.180 \"mkdir -p ~/glaucoma-detection/backend\""
            )
        );
    }

    #[test]
    fn does_not_confuse_a_local_missing_path_with_an_scp_remote_one() {
        let diagnosis = diagnose_failure(
            "Get-Content -LiteralPath 'missing.txt'",
            1,
            "Get-Content : Cannot find path 'missing.txt' because it does not exist.",
        );
        assert_eq!(diagnosis.kind, "missingPath");
    }

    #[test]
    fn falls_back_to_generic_nonzero_exit_guidance() {
        let diagnosis = diagnose_failure("example-command", 23, "Unexpected failure.");

        assert_eq!(diagnosis.kind, "nonzeroExit");
        assert!(diagnosis.explanation.contains("23"));
    }

    fn windows_test_workspace(id: &str, working_directory: &str) -> Workspace {
        Workspace {
            id: id.to_owned(),
            name: "test".to_owned(),
            path: working_directory.to_owned(),
            profile: ExecutionProfile {
                host_os: "windows".to_owned(),
                runtime: "local".to_owned(),
                runtime_name: "Windows".to_owned(),
                target_os: "windows".to_owned(),
                shell: "powershell".to_owned(),
                working_directory: working_directory.to_owned(),
                architecture: "x86_64".to_owned(),
                privilege: "standard-user".to_owned(),
            },
        }
    }

    /// `spawn_session` assigns its own session ID internally (independent of
    /// the workspace ID passed in), so tests must use the ID it returns
    /// rather than assuming it matches anything they supplied.
    fn spawned_test_session(sessions: &SessionManager, workspace_id: &str) -> String {
        let emit: crate::commands::sessions::EmitFn = Arc::new(|_| {});
        crate::commands::sessions::spawn_session(
            sessions,
            emit,
            windows_test_workspace(workspace_id, r"C:\Windows"),
        )
        .expect("test session should start")
        .id
    }

    #[cfg(target_os = "windows")]
    fn run_test_command(
        sessions: &SessionManager,
        runs: &RunManager,
        log_root: &Path,
        emit_output: &EmitOutputFn,
        emit_finished: &EmitFinishedFn,
        finished_rx: &std::sync::mpsc::Receiver<CommandRunFinishedEvent>,
        session_id: &str,
        command: &str,
    ) -> CommandRunSummary {
        let summary = start_run(
            sessions,
            runs,
            log_root,
            emit_output.clone(),
            emit_finished.clone(),
            session_id.to_owned(),
            command.to_owned(),
        )
        .expect("the test command should start");
        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the test command should report finished");
        assert!(
            finished.success,
            "command should succeed: {command} (exit {})",
            finished.exit_code
        );
        summary
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn runs_a_quick_command_and_reports_success() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-a");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn = Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        let summary = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "Write-Output hello-from-run".to_owned(),
        )
        .expect("a quick command should start");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the command should report finished");
        assert!(finished.success);
        assert_eq!(finished.exit_code, 0);
        assert!(!finished.stopped_by_user);
        assert_eq!(finished.started_at_ms, summary.started_at_ms);
        assert!(finished.finished_at_ms >= finished.started_at_ms);
        assert_eq!(
            finished.duration_ms,
            finished.finished_at_ms - finished.started_at_ms
        );

        let captured_output = output_lines.lock().unwrap();
        assert_eq!(
            captured_output.first().and_then(|event| event.command.as_deref()),
            Some("Write-Output hello-from-run"),
            "the command marker should be emitted before process output"
        );
        assert!(
            captured_output
                .iter()
                .any(|event| event.chunk.contains("hello-from-run")),
            "expected streamed output to include the command's own text"
        );
        assert!(
            captured_output
                .iter()
                .all(|event| event.run_id.as_deref() == Some(summary.id.as_str())),
            "every streamed line should identify its command run"
        );
        drop(captured_output);

        let logged = fs::read_to_string(&summary.log_path).expect("log file should exist and be readable");
        assert!(logged.contains("hello-from-run"), "log file should contain the command's output");
        assert!(
            !logged.contains("Write-Output hello-from-run"),
            "the UI-only command marker must not be written to the raw log"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn completes_when_stdout_has_no_trailing_newline() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-no-newline");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));
        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));
        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        let summary = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "[Console]::Out.Write('no-newline-payload')".to_owned(),
        )
        .expect("the no-newline command should start");
        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the no-newline command should report finished");

        assert!(finished.success);
        assert_eq!(finished.exit_code, 0);
        let captured_output = output_lines.lock().unwrap();
        assert!(captured_output.iter().any(|event| {
            event.run_id.as_deref() == Some(summary.id.as_str())
                && event.chunk == "no-newline-payload"
        }));
        assert!(captured_output
            .iter()
            .all(|event| !event.chunk.contains("__TERMINALMATE_BOUNDARY__")));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn captures_github_style_unicode_status_from_stderr_without_newline() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-gh-status");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));
        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));
        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        let summary = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            concat!(
                "Write-Output \"force-tty=$env:GH_FORCE_TTY\"; ",
                "python -c \"import sys; sys.stderr.write(chr(0x2713) + ",
                "' Updated variable TEST for owner/repo!')\""
            )
            .to_owned(),
        )
        .expect("the GitHub-style status command should start");
        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the GitHub-style status command should report finished");

        assert!(finished.success);
        let streamed = output_lines
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event.run_id.as_deref() == Some(summary.id.as_str()))
            .map(|event| event.chunk.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(streamed.contains("force-tty=120"), "streamed output: {streamed}");
        assert!(
            streamed.contains("✓ Updated variable TEST for owner/repo!"),
            "streamed output: {streamed}"
        );
        assert!(!streamed.contains("No output returned."));

        let logged =
            fs::read_to_string(&summary.log_path).expect("the GitHub-style status log should exist");
        assert!(logged.contains("✓ Updated variable TEST for owner/repo!"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn preserves_powershell_variables_between_submissions_in_one_session() {
        let _clipboard = ClipboardGuard::replace_with("hello-terminal-mate");
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-powershell-state");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output.clone(),
            emit_finished.clone(),
            session_id.clone(),
            "$foo = 'bar'".to_owned(),
        )
        .expect("the assignment should start");
        assert!(
            finished_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("the assignment should finish")
                .success
        );

        let read_summary = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output.clone(),
            emit_finished.clone(),
            session_id.clone(),
            "Write-Output $foo".to_owned(),
        )
        .expect("the variable read should start");
        assert!(
            finished_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("the variable read should finish")
                .success
        );

        let streamed = output_lines
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event.run_id.as_deref() == Some(read_summary.id.as_str()))
            .map(|event| event.chunk.trim().to_owned())
            .collect::<Vec<_>>();
        assert!(
            streamed.iter().any(|line| line == "bar"),
            "the second submission should read state from the same PowerShell process; got: {streamed:?}"
        );

        run_test_command(
            &sessions,
            &runs,
            &log_root,
            &emit_output,
            &emit_finished,
            &finished_rx,
            &session_id,
            "$n = 123",
        );
        let expression_summary = run_test_command(
            &sessions,
            &runs,
            &log_root,
            &emit_output,
            &emit_finished,
            &finished_rx,
            &session_id,
            "$n + 1",
        );
        assert!(
            output_lines.lock().unwrap().iter().any(|event| {
                event.run_id.as_deref() == Some(expression_summary.id.as_str())
                    && event.chunk.trim() == "124"
            }),
            "PowerShell expressions should use variables from earlier submissions"
        );

        run_test_command(
            &sessions,
            &runs,
            &log_root,
            &emit_output,
            &emit_finished,
            &finished_rx,
            &session_id,
            "$clip = Get-Clipboard -Raw",
        );
        let clipboard_length_summary = run_test_command(
            &sessions,
            &runs,
            &log_root,
            &emit_output,
            &emit_finished,
            &finished_rx,
            &session_id,
            "$clip.Length",
        );
        assert!(
            output_lines.lock().unwrap().iter().any(|event| {
                event.run_id.as_deref() == Some(clipboard_length_summary.id.as_str())
                    && event.chunk.trim() == "19"
            }),
            "clipboard text assigned in one submission should be available in the next"
        );
        let clipboard_text_summary = run_test_command(
            &sessions,
            &runs,
            &log_root,
            &emit_output,
            &emit_finished,
            &finished_rx,
            &session_id,
            "$clip",
        );
        let clipboard_lines = output_lines
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event.run_id.as_deref() == Some(clipboard_text_summary.id.as_str()))
            .map(|event| event.chunk.clone())
            .collect::<Vec<_>>();
        assert!(
            clipboard_lines.iter().any(|line| line.trim() == "hello-terminal-mate"),
            "clipboard content should survive as ordinary PowerShell session state; got: {clipboard_lines:?}"
        );
        let same_line_summary = run_test_command(
            &sessions,
            &runs,
            &log_root,
            &emit_output,
            &emit_finished,
            &finished_rx,
            &session_id,
            "$sameLine=[string](Get-Clipboard -Raw); \"Length: $($sameLine.Length)\"",
        );
        assert!(
            output_lines.lock().unwrap().iter().any(|event| {
                event.run_id.as_deref() == Some(same_line_summary.id.as_str())
                    && event.chunk.trim() == "Length: 19"
            }),
            "same-line assignments, parentheses, semicolons, and interpolation should remain intact"
        );

        let fresh_session_id = spawned_test_session(&sessions, "fresh-powershell-session");
        let fresh_read_summary = run_test_command(
            &sessions,
            &runs,
            &log_root,
            &emit_output,
            &emit_finished,
            &finished_rx,
            &fresh_session_id,
            "Write-Output $foo",
        );
        assert!(
            !output_lines.lock().unwrap().iter().any(|event| {
                event.run_id.as_deref() == Some(fresh_read_summary.id.as_str())
                    && event.chunk.trim() == "bar"
            }),
            "a newly opened terminal must start with fresh PowerShell state"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn preserves_formatted_powershell_object_pipeline_output() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-powershell-objects");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        let summary = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "[PSCustomObject]@{ Path = 'app/config.py'; LineNumber = 21; Line = 'openai_api_key' } | Select-Object Path,LineNumber,Line".to_owned(),
        )
        .expect("the PowerShell object pipeline should start");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the PowerShell object pipeline should report finished");
        assert!(finished.success);

        let streamed = output_lines
            .lock()
            .unwrap()
            .iter()
            .map(|event| event.chunk.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            streamed.contains("app/config.py") && streamed.contains("openai_api_key"),
            "formatted PowerShell objects should reach the terminal, got: {streamed}"
        );
        assert!(!streamed.contains("No output returned."));

        let logged = fs::read_to_string(&summary.log_path)
            .expect("the PowerShell object pipeline log should be readable");
        assert!(logged.contains("app/config.py"));
        assert!(logged.contains("openai_api_key"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn scoped_powershell_environment_survives_for_the_complete_script() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-scoped-environment");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "& {\n$env:TERMINALMATE_SCOPE_TEST = 'available'\nWrite-Output $env:TERMINALMATE_SCOPE_TEST\nRemove-Item Env:TERMINALMATE_SCOPE_TEST\nif (Test-Path Env:TERMINALMATE_SCOPE_TEST) { throw 'cleanup failed' }\n}".to_owned(),
        )
        .expect("the scoped environment script should start");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the scoped environment script should report finished");
        assert!(finished.success);
        assert!(
            output_lines
                .lock()
                .unwrap()
                .iter()
                .any(|event| event.chunk.trim() == "available"),
            "the temporary environment value should remain available until cleanup"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn command_processes_inherit_unbuffered_python_output() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-python-unbuffered");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "Write-Output $env:PYTHONUNBUFFERED".to_owned(),
        )
        .expect("the environment check should start");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the environment check should report finished");
        assert!(finished.success);
        assert!(
            output_lines
                .lock()
                .unwrap()
                .iter()
                .any(|event| event.chunk.trim() == "1"),
            "foreground commands should inherit PYTHONUNBUFFERED=1"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn python_processes_write_unicode_symbols_as_utf8() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-python-utf8");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "python -c \"import os; assert os.environ.get('PYTHONIOENCODING') == 'utf-8'; print(chr(0x2713) + ' ' + chr(0x2717))\"".to_owned(),
        )
        .expect("the Unicode Python command should start");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the Unicode Python command should report finished");
        assert!(finished.success, "Python should write Unicode symbols successfully");

        let expected = format!("{} {}", '\u{2713}', '\u{2717}');
        assert!(
            output_lines
                .lock()
                .unwrap()
                .iter()
                .any(|event| event.chunk.trim() == expected),
            "the check and cross symbols should reach TerminalMate intact"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn preserves_a_single_fifty_thousand_character_python_log_line() {
        const LINE_LENGTH: usize = 50_000;

        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-long-python-log-line");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        let summary = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "python -c \"import logging; logging.basicConfig(level=logging.INFO, format='%(message)s'); logging.info('X' * 50000)\"".to_owned(),
        )
        .expect("the Python logging command should start through the normal command path");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the long-line command should report finished");
        assert!(finished.success, "Python logging should not fail the child process");

        let expected = "X".repeat(LINE_LENGTH);
        let streamed = output_lines
            .lock()
            .unwrap()
            .iter()
            .filter(|event| matches!(event.stream, OutputStream::Stderr))
            .map(|event| event.chunk.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            streamed, expected,
            "the full Python log line should reach the frontend event path intact"
        );

        let logged = fs::read_to_string(&summary.log_path)
            .expect("the long-line command log should be readable");
        assert_eq!(
            logged.trim_end_matches(['\r', '\n']),
            expected,
            "the full Python log line should reach the command log intact"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn preserves_a_long_windows_encoded_python_log_line() {
        const ASCII_LENGTH: usize = 50_000;

        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-long-windows-log-line");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        let summary = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "python -c \"import sys; sys.stderr.buffer.write((b'X' * 25000) + bytes([0x96]) + (b'Y' * 25000) + b'\\n'); sys.stderr.flush()\"".to_owned(),
        )
        .expect("the Windows-encoded Python log command should start through the normal command path");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the Windows-encoded long-line command should report finished");
        assert!(finished.success, "Python should finish writing the encoded line");

        let expected = format!("{}\u{2013}{}", "X".repeat(25_000), "Y".repeat(25_000));
        assert_eq!(expected.chars().count(), ASCII_LENGTH + 1);

        let streamed = output_lines
            .lock()
            .unwrap()
            .iter()
            .filter(|event| matches!(event.stream, OutputStream::Stderr))
            .map(|event| event.chunk.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            streamed, expected,
            "a Windows-encoded long line should reach the frontend event path intact"
        );

        let logged = fs::read_to_string(&summary.log_path)
            .expect("the Windows-encoded command log should be readable");
        assert_eq!(
            logged.trim_end_matches(['\r', '\n']),
            expected,
            "a Windows-encoded long line should reach the command log intact"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn gives_python_pipe_handles_for_stdout_and_stderr() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-python-handle-types");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "python -c \"import ctypes, msvcrt, sys; get_type=ctypes.windll.kernel32.GetFileType; print('stdout=' + str(get_type(msvcrt.get_osfhandle(sys.stdout.fileno())))); print('stderr=' + str(get_type(msvcrt.get_osfhandle(sys.stderr.fileno()))))\"".to_owned(),
        )
        .expect("the Python handle inspection should start through the normal command path");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the handle inspection should report finished");
        assert!(finished.success);

        let output = output_lines
            .lock()
            .unwrap()
            .iter()
            .map(|event| event.chunk.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            output.contains("stdout=3"),
            "GetFileType should report FILE_TYPE_PIPE (3) for stdout, got: {output}"
        );
        assert!(
            output.contains("stderr=3"),
            "GetFileType should report FILE_TYPE_PIPE (3) for stderr, got: {output}"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn reports_when_a_successful_command_returns_no_output() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-empty-output");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        let summary = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "Write-Output $null".to_owned(),
        )
        .expect("an empty-output command should start");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the empty-output command should report finished");
        assert!(finished.success);
        assert!(
            output_lines
                .lock()
                .unwrap()
                .iter()
                .any(|event| event.chunk == "No output returned."),
            "the terminal should explicitly report an empty successful result"
        );

        let logged =
            fs::read_to_string(&summary.log_path).expect("the command log should be readable");
        assert!(logged.contains("No output returned."));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn reports_a_powershell_cmdlet_error_as_failed() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-cmdlet-error");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> =
            Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "Get-Content -LiteralPath 'terminal-mate-file-that-does-not-exist.txt'".to_owned(),
        )
        .expect("a safe command with a runtime error should start");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the failed command should report finished");
        assert!(!finished.success);
        assert_ne!(finished.exit_code, 0);
        assert!(
            output_lines
                .lock()
                .unwrap()
                .iter()
                .any(|event| event.chunk.contains("does not exist")),
            "the terminal should retain the PowerShell error"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn persists_the_final_working_directory_for_the_next_command() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-directory");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let output_lines: Arc<Mutex<Vec<SessionOutputEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let output_lines_clone = output_lines.clone();
        let emit_output: EmitOutputFn =
            Arc::new(move |event| output_lines_clone.lock().unwrap().push(event));

        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output.clone(),
            emit_finished.clone(),
            session_id.clone(),
            "Set-Location -LiteralPath 'System32'".to_owned(),
        )
        .expect("the directory-change command should start");

        let changed = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the directory-change command should finish");
        assert!(changed.success);
        assert!(
            changed.working_directory.ends_with(r"\Windows\System32"),
            "expected the final directory to be System32, got: {}",
            changed.working_directory
        );
        assert_eq!(
            sessions.working_directory(&session_id),
            Some(changed.working_directory.clone())
        );

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "Write-Output (Get-Location).Path".to_owned(),
        )
        .expect("the follow-up command should start");

        let follow_up = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the follow-up command should finish");
        assert!(follow_up.success);
        assert!(
            output_lines
                .lock()
                .unwrap()
                .iter()
                .any(|event| event.chunk.ends_with(r"\Windows\System32")),
            "the next command should start in the directory selected by the previous command"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn refuses_to_start_a_blocked_command() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-blocked");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let emit_output: EmitOutputFn = Arc::new(|_| {});
        let emit_finished: EmitFinishedFn = Arc::new(|_| {});

        let result = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id,
            "diskpart".to_owned(),
        );

        assert!(result.is_err());
        assert_eq!(classify_command("diskpart").level, PolicyRiskLevel::Blocked);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn stop_command_ends_a_long_running_process_and_marks_it_stopped_by_user() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-b");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let emit_output: EmitOutputFn = Arc::new(|_| {});
        let (finished_tx, finished_rx) = channel::<CommandRunFinishedEvent>();
        let emit_finished: EmitFinishedFn = Arc::new(move |event| {
            let _ = finished_tx.send(event);
        });

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id.clone(),
            "Start-Sleep -Seconds 120".to_owned(),
        )
        .expect("a long-running command should start");

        // Give the process a moment to actually be running before stopping it.
        std::thread::sleep(Duration::from_millis(300));
        stop_run(&sessions, &runs, &session_id)
            .expect("stopping a running command should succeed");

        let finished = finished_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("stopping should still produce a finished event");
        assert!(finished.stopped_by_user);
        assert!(!finished.success);

        let deadline = Instant::now() + Duration::from_secs(5);
        while runs.active.lock().unwrap().contains_key(&session_id) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(!runs.active.lock().unwrap().contains_key(&session_id));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn refuses_a_second_concurrent_run_in_the_same_session() {
        let sessions = SessionManager::default();
        let session_id = spawned_test_session(&sessions, "session-c");
        let runs = RunManager::default();
        let log_root = std::env::temp_dir().join(format!("tm-test-logs-{}", Uuid::new_v4()));

        let emit_output: EmitOutputFn = Arc::new(|_| {});
        let emit_finished: EmitFinishedFn = Arc::new(|_| {});

        start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output.clone(),
            emit_finished.clone(),
            session_id.clone(),
            "Start-Sleep -Seconds 5".to_owned(),
        )
        .expect("first run should start");

        let second = start_run(
            &sessions,
            &runs,
            &log_root,
            emit_output,
            emit_finished,
            session_id.clone(),
            "Get-Location".to_owned(),
        );

        assert!(second.is_err());
        stop_run(&sessions, &runs, &session_id).expect("cleanup stop should succeed");
    }
}
