mod app_state;
mod commands;
mod models;
mod sidecar;
mod sidecar_proxy;

use std::sync::atomic::Ordering;

use tauri::RunEvent;

use app_state::AppState;
use commands::command_runs::RunManager;
use commands::sessions::SessionManager;
use sidecar_proxy::SidecarProxy;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::default();
    let proxy = SidecarProxy::new().expect("Unable to initialise the local assistant proxy.");
    let startup_state = state.clone();
    let shutdown_state = state.clone();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state.clone())
        .manage(proxy.clone())
        .manage(SessionManager::default())
        .manage(RunManager::default())
        .setup(move |app| {
            sidecar::launch(app.handle().clone(), startup_state.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspaces::pick_workspace,
            commands::workspaces::configure_workspace_runtime,
            commands::sessions::create_session,
            commands::sessions::close_session,
            commands::sessions::send_session_input,
            commands::sessions::set_session_working_directory,
            commands::policy::classify_command_text,
            commands::command_runs::run_command,
            commands::command_runs::stop_command,
            commands::command_runs::open_log_file,
            commands::command_runs::read_log_file,
            commands::command_runs::diagnose_command_failure,
            commands::command_runs::read_azure_subscriptions,
            commands::command_runs::detect_created_virtual_machine,
            commands::command_runs::read_azure_virtual_machines,
            commands::intents::resolve_command,
            commands::settings::get_ai_planner_settings,
            commands::settings::save_ai_planner_settings,
            commands::file_editor::read_editable_file,
            commands::file_editor::write_editable_file,
            commands::external_terminal::open_external_terminal
        ])
        .build(tauri::generate_context!())
        .expect("error while building TerminalMate");

    app.run(move |app_handle, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            // The first exit request is the user closing the application. Pause it
            // while Rust terminates the private sidecar process tree.
            if shutdown_state.shutdown_started.swap(true, Ordering::AcqRel) {
                // app_handle.exit(0) triggers a second ExitRequested event after
                // cleanup. Let that final event exit normally.
                return;
            }

            api.prevent_exit();

            let state = shutdown_state.clone();
            let app_handle = app_handle.clone();

            tauri::async_runtime::spawn(async move {
                sidecar::stop_sidecar(state).await;

                // Cleanup is complete, so allow Tauri to perform its normal exit.
                app_handle.exit(0);
            });
        }
    });
}
