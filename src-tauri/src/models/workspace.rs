use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionProfile {
    pub host_os: String,
    pub runtime: String,
    pub runtime_name: String,
    pub target_os: String,
    pub shell: String,
    pub working_directory: String,
    pub architecture: String,
    pub privilege: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub profile: ExecutionProfile,
}

