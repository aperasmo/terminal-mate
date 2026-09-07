use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel {
    Safe,
    Caution,
    HighRisk,
    Blocked,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskDecision {
    pub level: RiskLevel,
    pub reason: String,
    /// Populated only for the Blocked "interactive editor" case, when a file
    /// path could be extracted from the command (e.g. `nano main.tf`). Lets
    /// the frontend offer TerminalMate's built-in editor as a working
    /// alternative instead of just explaining why the command was refused.
    pub editable_path: Option<String>,
    /// True for a Blocked command that has no in-app workaround (a bare
    /// interactive `ssh` login, or a monitor like `top`/`htop`) but would
    /// work fine in a real terminal. Lets the frontend offer to open the
    /// exact same command in a new external terminal window instead of
    /// just explaining why it cannot run inside TerminalMate.
    pub external_terminal: bool,
}
