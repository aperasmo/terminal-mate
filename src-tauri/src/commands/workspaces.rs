use std::path::{Path, PathBuf};
use std::process::Command;

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, FilePath};
use uuid::Uuid;

use crate::models::workspace::{ExecutionProfile, Workspace};

#[tauri::command]
pub fn pick_workspace(app: AppHandle) -> Result<Option<Workspace>, String> {
    let selected = app.dialog().file().blocking_pick_folder();
    let Some(selected) = selected else {
        return Ok(None);
    };

    let path = match selected {
        FilePath::Path(path) => path,
        FilePath::Url(_) => {
            return Err("The selected workspace must be a local filesystem path.".to_owned());
        }
    };

    let canonical = path.canonicalize().unwrap_or(path);
    let name = workspace_name(&canonical)?;
    let profile = default_execution_profile(&canonical);

    Ok(Some(Workspace {
        id: Uuid::new_v4().to_string(),
        name,
        path: canonical.to_string_lossy().into_owned(),
        profile,
    }))
}

#[tauri::command]
pub fn configure_workspace_runtime(
    mut workspace: Workspace,
    runtime: String,
) -> Result<Workspace, String> {
    workspace.profile = execution_profile_for_runtime(&workspace.path, &runtime)?;
    Ok(workspace)
}

fn workspace_name(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| "The selected workspace does not have a usable folder name.".to_owned())
}

fn default_execution_profile(path: &Path) -> ExecutionProfile {
    if cfg!(target_os = "windows") {
        ExecutionProfile {
            host_os: "windows".to_owned(),
            runtime: "local".to_owned(),
            runtime_name: "Windows".to_owned(),
            target_os: "windows".to_owned(),
            shell: "powershell".to_owned(),
            working_directory: path.to_string_lossy().into_owned(),
            architecture: std::env::consts::ARCH.to_owned(),
            privilege: "standard-user".to_owned(),
        }
    } else {
        ExecutionProfile {
            host_os: std::env::consts::OS.to_owned(),
            runtime: "local".to_owned(),
            runtime_name: "Local".to_owned(),
            target_os: std::env::consts::OS.to_owned(),
            shell: if cfg!(target_os = "macos") {
                "zsh".to_owned()
            } else {
                "bash".to_owned()
            },
            working_directory: path.to_string_lossy().into_owned(),
            architecture: std::env::consts::ARCH.to_owned(),
            privilege: "standard-user".to_owned(),
        }
    }
}

fn execution_profile_for_runtime(path: &str, runtime: &str) -> Result<ExecutionProfile, String> {
    let path = PathBuf::from(path);
    match runtime.to_ascii_lowercase().as_str() {
        "local" | "windows" => {
            if !cfg!(target_os = "windows") {
                return Err("Windows / PowerShell is only available on a Windows host.".to_owned());
            }
            Ok(default_execution_profile(&path))
        }
        "wsl" => {
            if !cfg!(target_os = "windows") {
                return Err("WSL is only available on a Windows host.".to_owned());
            }
            ensure_wsl_available()?;
            let working_directory = windows_path_to_wsl(&path).ok_or_else(|| {
                "This workspace path cannot be mapped into WSL. Choose a folder on a Windows drive."
                    .to_owned()
            })?;
            Ok(ExecutionProfile {
                host_os: "windows".to_owned(),
                runtime: "wsl".to_owned(),
                runtime_name: "WSL Linux".to_owned(),
                target_os: "linux".to_owned(),
                shell: "bash".to_owned(),
                working_directory,
                architecture: std::env::consts::ARCH.to_owned(),
                privilege: "standard-user".to_owned(),
            })
        }
        other => Err(format!("Unsupported workspace runtime: {other}.")),
    }
}

fn ensure_wsl_available() -> Result<(), String> {
    let mut command = Command::new("wsl.exe");
    command.arg("--status");
    hide_console_window(&mut command);
    let status = command
        .status()
        .map_err(|_| "WSL is not installed or wsl.exe is not available.".to_owned())?;
    if status.success() {
        Ok(())
    } else {
        Err("WSL is installed but not ready. Install or start a Linux distribution, then try again.".to_owned())
    }
}

#[cfg(target_os = "windows")]
fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_console_window(_command: &mut Command) {}

pub(crate) fn windows_path_to_wsl(path: &Path) -> Option<String> {
    let raw_value = path.to_string_lossy();
    let raw = raw_value.strip_prefix(r"\\?\").unwrap_or(&raw_value);
    let bytes = raw.as_bytes();
    if bytes.len() < 3 || bytes[1] != b':' || (bytes[2] != b'\\' && bytes[2] != b'/') {
        return None;
    }

    let drive = (bytes[0] as char).to_ascii_lowercase();
    if !drive.is_ascii_alphabetic() {
        return None;
    }

    let remainder = raw[3..].replace('\\', "/");
    Some(if remainder.is_empty() {
        format!("/mnt/{drive}")
    } else {
        format!("/mnt/{drive}/{remainder}")
    })
}

/// The reverse of `windows_path_to_wsl`. Needed because Tauri commands run
/// natively on the Windows host even for a WSL workspace — there is no
/// direct `std::fs` access to a WSL path, so anything that reads/writes a
/// file by path (e.g. the built-in file editor) must convert a WSL working
/// directory back to its Windows equivalent first.
pub(crate) fn wsl_path_to_windows(path: &str) -> Option<PathBuf> {
    let remainder = path.trim().strip_prefix("/mnt/")?;
    let mut chars = remainder.chars();
    let drive = chars.next()?;
    if !drive.is_ascii_alphabetic() {
        return None;
    }

    let rest = chars.as_str();
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    let windows_rest = rest.replace('/', "\\");
    let drive = drive.to_ascii_uppercase();
    Some(if windows_rest.is_empty() {
        PathBuf::from(format!("{drive}:\\"))
    } else {
        PathBuf::from(format!("{drive}:\\{windows_rest}"))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        default_execution_profile, execution_profile_for_runtime, windows_path_to_wsl,
        wsl_path_to_windows,
    };
    use std::path::PathBuf;

    #[test]
    fn converts_windows_workspace_path_to_wsl() {
        let path = PathBuf::from(r"D:\APE\Portfolio-Project\terminal-mate");
        assert_eq!(
            windows_path_to_wsl(&path).as_deref(),
            Some("/mnt/d/APE/Portfolio-Project/terminal-mate")
        );
    }

    #[test]
    fn converts_extended_windows_workspace_path_to_wsl() {
        let path = PathBuf::from(r"\\?\D:\APE\Portfolio-Project\terminal-mate");
        assert_eq!(
            windows_path_to_wsl(&path).as_deref(),
            Some("/mnt/d/APE/Portfolio-Project/terminal-mate")
        );
    }

    #[test]
    fn leaves_non_windows_paths_unmapped() {
        let path = PathBuf::from("/home/allan/terminal-mate");
        assert_eq!(windows_path_to_wsl(&path), None);
    }

    #[test]
    fn converts_wsl_path_back_to_windows() {
        assert_eq!(
            wsl_path_to_windows("/mnt/d/APE/YOOBEE/Capstone/glaucoma-detection"),
            Some(PathBuf::from(r"D:\APE\YOOBEE\Capstone\glaucoma-detection"))
        );
    }

    #[test]
    fn converts_bare_wsl_drive_root_back_to_windows() {
        assert_eq!(wsl_path_to_windows("/mnt/d"), Some(PathBuf::from(r"D:\")));
    }

    #[test]
    fn round_trips_windows_to_wsl_and_back() {
        let original = PathBuf::from(r"D:\APE\Portfolio-Project\terminal-mate");
        let wsl = windows_path_to_wsl(&original).expect("should convert to a WSL path");
        assert_eq!(wsl_path_to_windows(&wsl), Some(original));
    }

    #[test]
    fn leaves_non_wsl_paths_unmapped() {
        assert_eq!(wsl_path_to_windows(r"D:\APE\terminal-mate"), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn defaults_to_native_powershell_on_windows() {
        let path = PathBuf::from(r"D:\APE\Portfolio-Project\terminal-mate");
        let profile = default_execution_profile(&path);
        assert_eq!(profile.runtime, "local");
        assert_eq!(profile.shell, "powershell");
        assert_eq!(profile.target_os, "windows");
        assert_eq!(profile.working_directory, path.to_string_lossy());
    }


    #[cfg(target_os = "windows")]
    #[test]
    fn builds_a_native_windows_profile_explicitly() {
        let profile = execution_profile_for_runtime(r"D:\APE\terminal-mate", "windows")
            .expect("Windows should be available on a Windows host");
        assert_eq!(profile.runtime, "local");
        assert_eq!(profile.shell, "powershell");
    }
}
