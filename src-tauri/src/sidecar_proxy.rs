use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Clone)]
pub struct SidecarProxy {
    client: Client,
}

impl SidecarProxy {
    pub fn new() -> Result<Self, String> {
        // The Rust layer owns this HTTP client. React never receives either
        // the sidecar endpoint or the per-session token used below.
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(35))
            .build()
            .map_err(|error| format!("Unable to create local HTTP client: {error}"))?;
        Ok(Self { client })
    }

    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str, token: &str, path: &str) -> Result<T, String> {
        self.request::<(), T>(endpoint, token, Method::GET, path, None).await
    }

    pub async fn post<Req: Serialize + ?Sized, Res: DeserializeOwned>(
        &self,
        endpoint: &str,
        token: &str,
        path: &str,
        payload: &Req,
    ) -> Result<Res, String> {
        self.request(endpoint, token, Method::POST, path, Some(payload)).await
    }

    /// Convenience wrapper that reads the endpoint/token from `AppState` and
    /// resolves to `None` (rather than an error) when the sidecar is not
    /// currently ready — the caller falls back to direct command handling.
    pub async fn post_if_ready<Req: Serialize + ?Sized, Res: DeserializeOwned>(
        &self,
        state: &AppState,
        path: &str,
        payload: &Req,
    ) -> Option<Res> {
        let (endpoint, token) = state.sidecar_details().await?;
        self.post(&endpoint, &token, path, payload).await.ok()
    }

    async fn request<Req: Serialize + ?Sized, Res: DeserializeOwned>(
        &self,
        endpoint: &str,
        token: &str,
        method: Method,
        path: &str,
        payload: Option<&Req>,
    ) -> Result<Res, String> {
        let url = format!("{endpoint}{path}");

        // The bearer token is attached inside Rust only. The webview cannot
        // create arbitrary sidecar requests or read this token from state.
        let mut request = self
            .client
            .request(method, url)
            .bearer_auth(token)
            .header("Content-Type", "application/json");

        if let Some(body) = payload {
            request = request.json(body);
        }

        let response = request
            .send()
            .await
            .map_err(|error| format!("The local assistant service could not be reached: {error}"))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            let detail = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|json| json.get("detail").cloned())
                .map(|detail| match detail {
                    serde_json::Value::String(message) => message,
                    other => other.to_string(),
                })
                .unwrap_or_else(|| "The local assistant service rejected the request.".to_owned());
            return Err(detail);
        }

        response
            .json::<Res>()
            .await
            .map_err(|error| format!("The local assistant service returned an invalid response: {error}"))
    }
}
