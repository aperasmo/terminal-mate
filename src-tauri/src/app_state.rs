use std::sync::{atomic::AtomicBool, Arc};

use tauri_plugin_shell::process::CommandChild;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AiPlannerSettings {
    pub enabled: bool,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl Default for AiPlannerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: "https://api.openai.com/v1/chat/completions".to_owned(),
            model: "gpt-4.1-mini".to_owned(),
            api_key: None,
        }
    }
}

pub struct SidecarRuntime {
    pub status: String,
    pub message: String,
    pub protocol_version: Option<String>,
    pub endpoint: Option<String>,
    pub session_token: Option<String>,
    pub child: Option<CommandChild>,
    pub recent_logs: Vec<String>,
}

impl Default for SidecarRuntime {
    fn default() -> Self {
        Self {
            status: "starting".to_owned(),
            message: "Starting local assistant service…".to_owned(),
            protocol_version: None,
            endpoint: None,
            session_token: None,
            child: None,
            recent_logs: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<Mutex<SidecarRuntime>>,
    pub ai_planner: Arc<Mutex<AiPlannerSettings>>,
    pub shutdown_started: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            runtime: Arc::new(Mutex::new(SidecarRuntime::default())),
            ai_planner: Arc::new(Mutex::new(AiPlannerSettings::default())),
            shutdown_started: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl AppState {
    pub async fn starting(&self) {
        let mut runtime = self.runtime.lock().await;
        runtime.status = "starting".to_owned();
        runtime.message = "Starting local assistant service…".to_owned();
        runtime.protocol_version = None;
        runtime.endpoint = None;
        runtime.session_token = None;
        runtime.child = None;
        runtime.recent_logs.clear();
    }

    pub async fn store_sidecar_child(&self, child: CommandChild) {
        // Store the handle immediately after spawn. Shutdown must be able to
        // terminate the sidecar even when startup fails before readiness.
        let mut runtime = self.runtime.lock().await;
        runtime.child = Some(child);
    }

    pub async fn configure_sidecar(&self, endpoint: String, token: String, protocol_version: String) {
        let mut runtime = self.runtime.lock().await;
        runtime.endpoint = Some(endpoint);
        runtime.session_token = Some(token);
        runtime.protocol_version = Some(protocol_version);
        runtime.status = "starting".to_owned();
        runtime.message = "Verifying local assistant service…".to_owned();
    }

    pub async fn ready(&self) {
        let mut runtime = self.runtime.lock().await;
        runtime.status = "ready".to_owned();
        runtime.message = "Local assistant service is ready.".to_owned();
    }

    pub async fn failed(&self, message: impl Into<String>) {
        let mut runtime = self.runtime.lock().await;
        let message = message.into();
        runtime.status = "failed".to_owned();
        runtime.message = message.clone();
        runtime.endpoint = None;
        runtime.session_token = None;
        runtime.protocol_version = None;
        push_recent_log(&mut runtime.recent_logs, message);
    }

    pub async fn stopped(&self) {
        let mut runtime = self.runtime.lock().await;
        runtime.status = "stopped".to_owned();
        runtime.message = "Local assistant service stopped.".to_owned();
        runtime.endpoint = None;
        runtime.session_token = None;
        runtime.protocol_version = None;
        runtime.child = None;
    }

    pub async fn record_sidecar_log(&self, message: impl Into<String>) {
        let mut runtime = self.runtime.lock().await;
        push_recent_log(&mut runtime.recent_logs, message.into());
    }

    /// `None` means the sidecar is not ready to receive requests right now
    /// (starting, failed, or stopped) — callers should fall back to direct
    /// command handling rather than error out, since plain-English intent
    /// matching is an enhancement on top of the always-available literal
    /// command path, not a requirement for it.
    pub async fn sidecar_details(&self) -> Option<(String, String)> {
        let runtime = self.runtime.lock().await;
        match (&runtime.endpoint, &runtime.session_token) {
            (Some(endpoint), Some(token)) if runtime.status == "ready" => {
                Some((endpoint.clone(), token.clone()))
            }
            _ => None,
        }
    }
}

fn push_recent_log(logs: &mut Vec<String>, message: String) {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return;
    }

    logs.push(trimmed.chars().take(500).collect());
    if logs.len() > 30 {
        let overflow = logs.len() - 30;
        logs.drain(0..overflow);
    }
}
