mod commands;
mod models;

use commands::sessions::SessionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(SessionManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::workspaces::pick_workspace,
            commands::sessions::create_session,
            commands::sessions::close_session,
            commands::sessions::execute_command,
            commands::policy::classify_command_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running TerminalMate");
}

