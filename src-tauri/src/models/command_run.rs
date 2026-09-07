use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRunSummary {
    pub id: String,
    pub session_id: String,
    pub command: String,
    pub log_path: String,
    pub started_at_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRunFinishedEvent {
    pub id: String,
    pub session_id: String,
    pub exit_code: i32,
    pub success: bool,
    pub stopped_by_user: bool,
    pub log_path: String,
    pub working_directory: String,
    pub started_at_ms: u64,
    pub finished_at_ms: u64,
    pub duration_ms: u64,
}
