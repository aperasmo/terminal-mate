use std::process::{Child, Command};

/// The escape hatch for commands `policy::classify_command` marks
/// `external_terminal: true` — a bare interactive `ssh` login, or a monitor
/// like `top`/`htop` — that need a real terminal (live keystrokes, screen
/// redraw) TerminalMate's captured-output execution model cannot provide.
/// Rather than emulate one, this opens the exact same command in a real,
/// separate terminal window, where it can behave normally; TerminalMate
/// itself keeps running only commands it can supervise and classify.
#[tauri::command]
pub fn open_external_terminal(
    working_directory: String,
    shell: String,
    command: String,
) -> Result<(), String> {
    let mut process = match shell.to_ascii_lowercase().as_str() {
        "powershell" => {
            let mut process = Command::new("powershell.exe");
            process
                .args([
                    "-NoExit",
                    "-Command",
                    "Set-Location -LiteralPath $env:TERMINALMATE_TERMINAL_DIR; \
                     & ([ScriptBlock]::Create($env:TERMINALMATE_COMMAND))",
                ])
                .env("TERMINALMATE_TERMINAL_DIR", &working_directory)
                .env("TERMINALMATE_COMMAND", &command);
            process
        }
        "bash" => {
            let mut process = Command::new("wsl.exe");
            process.args([
                "--cd",
                &working_directory,
                "--exec",
                "bash",
                "-ic",
                "eval \"$1\"; exec bash",
                "terminalmate",
                &command,
            ]);
            process
        }
        other => {
            return Err(format!(
                "Cannot open an external terminal for the \"{other}\" shell."
            ));
        }
    };

    spawn_in_new_console(&mut process)
        .map(|_| ())
        .map_err(|error| format!("Could not open an external terminal: {error}"))
}

#[cfg(target_os = "windows")]
fn spawn_in_new_console(command: &mut Command) -> std::io::Result<Child> {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    command.creation_flags(CREATE_NEW_CONSOLE);
    command.spawn()
}

#[cfg(not(target_os = "windows"))]
fn spawn_in_new_console(command: &mut Command) -> std::io::Result<Child> {
    command.spawn()
}
