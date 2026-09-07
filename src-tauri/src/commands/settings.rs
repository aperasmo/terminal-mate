use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPlannerSettingsSnapshot {
    enabled: bool,
    endpoint: String,
    model: String,
    api_key_set: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPlannerSettingsUpdate {
    enabled: bool,
    endpoint: String,
    model: String,
    api_key: Option<String>,
    #[serde(default)]
    clear_api_key: bool,
}

#[tauri::command]
pub async fn get_ai_planner_settings(
    state: State<'_, AppState>,
) -> Result<AiPlannerSettingsSnapshot, String> {
    let settings = state.ai_planner.lock().await;
    Ok(snapshot(&settings))
}

#[tauri::command]
pub async fn save_ai_planner_settings(
    request: AiPlannerSettingsUpdate,
    state: State<'_, AppState>,
) -> Result<AiPlannerSettingsSnapshot, String> {
    let endpoint = validate_endpoint(&request.endpoint)?;
    let model = request.model.trim();
    if model.is_empty() || model.len() > 200 {
        return Err("Enter an AI model name containing at most 200 characters.".to_owned());
    }

    let replacement_key = request
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if replacement_key.is_some_and(|value| value.len() > 4_000) {
        return Err("The AI API key is too long.".to_owned());
    }

    let mut settings = state.ai_planner.lock().await;
    settings.enabled = request.enabled;
    settings.endpoint = endpoint;
    settings.model = model.to_owned();
    if request.clear_api_key {
        settings.api_key = None;
    } else if let Some(api_key) = replacement_key {
        settings.api_key = Some(api_key.to_owned());
    }
    Ok(snapshot(&settings))
}

fn snapshot(settings: &crate::app_state::AiPlannerSettings) -> AiPlannerSettingsSnapshot {
    AiPlannerSettingsSnapshot {
        enabled: settings.enabled,
        endpoint: settings.endpoint.clone(),
        model: settings.model.clone(),
        api_key_set: settings.api_key.is_some(),
    }
}

fn validate_endpoint(value: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() || normalized.len() > 2_000 {
        return Err("Enter an AI endpoint containing at most 2000 characters.".to_owned());
    }

    let url = reqwest::Url::parse(normalized)
        .map_err(|_| "Enter a valid AI endpoint URL.".to_owned())?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err("Do not include credentials in the AI endpoint URL.".to_owned());
    }
    let local_http = url.scheme() == "http"
        && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !local_http {
        return Err("Use HTTPS, or HTTP only for a local AI endpoint.".to_owned());
    }
    Ok(normalized.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_and_local_http_endpoints() {
        assert!(validate_endpoint("https://example.com/v1/chat/completions").is_ok());
        assert!(validate_endpoint("http://localhost:1234/v1/chat/completions").is_ok());
    }

    #[test]
    fn rejects_remote_http_and_embedded_credentials() {
        assert!(validate_endpoint("http://example.com/v1/chat/completions").is_err());
        assert!(validate_endpoint("https://secret@example.com/v1/chat/completions").is_err());
    }
}
