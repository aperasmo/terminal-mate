use serde::{Deserialize, Serialize};

use crate::models::workspace::ExecutionProfile;

/// The sidecar's Pydantic schema declares plain snake_case field names (its
/// JSON body is not camelCase-aliased the way Tauri IPC responses to the
/// frontend are), so this mirrors `ExecutionProfile` without the
/// `rename_all = "camelCase"` used for the webview-facing copy.
#[derive(Debug, Serialize)]
pub struct SidecarExecutionProfile {
    pub host_os: String,
    pub runtime: String,
    pub runtime_name: String,
    pub target_os: String,
    pub shell: String,
    pub working_directory: String,
    pub architecture: String,
    pub privilege: String,
}

impl From<&ExecutionProfile> for SidecarExecutionProfile {
    fn from(profile: &ExecutionProfile) -> Self {
        Self {
            host_os: profile.host_os.clone(),
            runtime: profile.runtime.clone(),
            runtime_name: profile.runtime_name.clone(),
            target_os: profile.target_os.clone(),
            shell: profile.shell.clone(),
            working_directory: profile.working_directory.clone(),
            architecture: profile.architecture.clone(),
            privilege: profile.privilege.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IntentRequest {
    pub message: String,
    pub execution_profile: SidecarExecutionProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_config: Option<SidecarAiPlannerConfig>,
}

#[derive(Debug, Serialize)]
pub struct SidecarAiPlannerConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommandIntent {
    #[allow(dead_code)]
    pub schema_version: u8,
    pub action: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct IntentResponse {
    pub matched: bool,
    #[allow(dead_code)]
    pub source: String,
    pub intent: Option<CommandIntent>,
    #[allow(dead_code)]
    pub message: Option<String>,
    #[serde(default)]
    pub requires_clarification: bool,
}

/// What the command bar actually runs, whether it came from plain-English
/// intent matching or was typed as a literal command.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCommand {
    pub command: String,
    pub source: ResolvedCommandSource,
    pub note: Option<String>,
    pub execution_mode: ResolvedCommandExecutionMode,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResolvedCommandSource {
    Intent,
    AiIntent,
    Direct,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResolvedCommandExecutionMode {
    Execute,
    ExplainOnly,
}
