use std::fs;
use std::path::PathBuf;

use crate::commands::workspaces::wsl_path_to_windows;

/// Full-screen interactive editors (nano, vim, ...) cannot run through
/// TerminalMate's execution model at all — see `policy::is_interactive_editor`.
/// This is the workaround offered instead: read/write the file directly, no
/// shell or terminal involved, so the safety story is simpler (a file write,
/// not a program TerminalMate cannot supervise) rather than weaker.
const MAX_EDITABLE_FILE_BYTES: u64 = 2 * 1024 * 1024;

#[tauri::command]
pub fn read_editable_file(working_directory: String, path: String) -> Result<String, String> {
    let resolved = resolve_editable_path(&working_directory, &path)?;

    let metadata = match fs::metadata(&resolved) {
        Ok(metadata) => metadata,
        // A filename that does not exist yet is exactly what `nano <newfile>`
        // does today: start with an empty buffer, not an error.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(String::new());
        }
        Err(error) => return Err(format!("Could not read \"{path}\": {error}")),
    };

    if !metadata.is_file() {
        return Err(format!("\"{path}\" is not a file."));
    }
    if metadata.len() > MAX_EDITABLE_FILE_BYTES {
        return Err(format!(
            "\"{path}\" is {} bytes, over the {MAX_EDITABLE_FILE_BYTES}-byte limit for TerminalMate's built-in editor.",
            metadata.len()
        ));
    }

    fs::read_to_string(&resolved)
        .map_err(|error| format!("Could not read \"{path}\" as text: {error}. It may not be a text file."))
}

#[tauri::command]
pub fn write_editable_file(
    working_directory: String,
    path: String,
    content: String,
) -> Result<(), String> {
    let resolved = resolve_editable_path(&working_directory, &path)?;
    fs::write(&resolved, content).map_err(|error| format!("Could not save \"{path}\": {error}"))
}

/// Resolves a (possibly relative) editable-file path against the session's
/// current working directory. That directory may itself be a WSL-style path
/// (`/mnt/d/...`) even though this Tauri command always runs natively on the
/// Windows host — there is no native `std::fs` access to a WSL path
/// directly, so a WSL working directory (or a WSL-style absolute path) is
/// converted back to its Windows equivalent first.
fn resolve_editable_path(working_directory: &str, path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();

    if is_wsl_style_path(trimmed) {
        return wsl_path_to_windows(trimmed)
            .ok_or_else(|| format!("Could not resolve the path \"{trimmed}\"."));
    }

    if is_windows_absolute_path(trimmed) {
        return Ok(PathBuf::from(trimmed));
    }

    let base = if is_wsl_style_path(working_directory) {
        wsl_path_to_windows(working_directory).ok_or_else(|| {
            format!("Could not resolve the WSL working directory \"{working_directory}\".")
        })?
    } else {
        PathBuf::from(working_directory)
    };

    Ok(base.join(trimmed))
}

fn is_wsl_style_path(value: &str) -> bool {
    value.starts_with('/')
}

fn is_windows_absolute_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_a_relative_path_against_a_windows_working_directory() {
        let resolved = resolve_editable_path(r"D:\APE\project", "infra/azure/main.tf").unwrap();
        assert_eq!(resolved, PathBuf::from(r"D:\APE\project\infra/azure/main.tf"));
    }

    #[test]
    fn resolves_a_relative_path_against_a_wsl_working_directory() {
        let resolved = resolve_editable_path(
            "/mnt/d/APE/YOOBEE/Capstone/glaucoma-detection",
            "infra/azure/main.tf",
        )
        .unwrap();
        assert_eq!(
            resolved,
            PathBuf::from(r"D:\APE\YOOBEE\Capstone\glaucoma-detection\infra/azure/main.tf")
        );
    }

    #[test]
    fn resolves_a_wsl_style_absolute_path_regardless_of_working_directory() {
        let resolved =
            resolve_editable_path(r"D:\unrelated", "/mnt/d/APE/project/main.tf").unwrap();
        assert_eq!(resolved, PathBuf::from(r"D:\APE\project\main.tf"));
    }

    #[test]
    fn resolves_a_windows_style_absolute_path_regardless_of_working_directory() {
        let resolved =
            resolve_editable_path("/mnt/d/unrelated", r"D:\APE\project\main.tf").unwrap();
        assert_eq!(resolved, PathBuf::from(r"D:\APE\project\main.tf"));
    }

    #[test]
    fn reading_a_missing_file_returns_an_empty_buffer_not_an_error() {
        let missing = std::env::temp_dir().join(format!(
            "terminal-mate-file-editor-test-missing-{}.tf",
            uuid::Uuid::new_v4()
        ));
        let result = read_editable_file(
            missing.parent().unwrap().to_string_lossy().into_owned(),
            missing.file_name().unwrap().to_string_lossy().into_owned(),
        );
        assert_eq!(result, Ok(String::new()));
    }

    #[test]
    fn writes_and_reads_back_file_content() {
        let dir = std::env::temp_dir();
        let file_name = format!("terminal-mate-file-editor-test-{}.tf", uuid::Uuid::new_v4());

        write_editable_file(
            dir.to_string_lossy().into_owned(),
            file_name.clone(),
            "resource \"azurerm_resource_group\" \"demo\" {}\n".to_owned(),
        )
        .expect("write should succeed");

        let content = read_editable_file(dir.to_string_lossy().into_owned(), file_name.clone())
            .expect("read should succeed");
        assert_eq!(content, "resource \"azurerm_resource_group\" \"demo\" {}\n");

        let _ = fs::remove_file(dir.join(file_name));
    }

    #[test]
    fn rejects_a_file_over_the_size_limit() {
        let dir = std::env::temp_dir();
        let file_name = format!("terminal-mate-file-editor-test-large-{}.tf", uuid::Uuid::new_v4());
        let path = dir.join(&file_name);
        fs::write(&path, vec![b'a'; (MAX_EDITABLE_FILE_BYTES + 1) as usize]).unwrap();

        let result = read_editable_file(dir.to_string_lossy().into_owned(), file_name.clone());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("byte limit"));

        let _ = fs::remove_file(&path);
    }
}
