use std::path::Path;

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

// Reserved for the WSL adapter (a later phase); the Windows-native default
// profile no longer calls this.
#[allow(dead_code)]
fn windows_path_to_wsl(path: &Path) -> Option<String> {
    let raw = path.to_string_lossy();
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

#[cfg(test)]
mod tests {
    use super::{default_execution_profile, windows_path_to_wsl};
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
    fn leaves_non_windows_paths_unmapped() {
        let path = PathBuf::from("/home/allan/terminal-mate");
        assert_eq!(windows_path_to_wsl(&path), None);
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
}
