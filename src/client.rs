use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::de::DeserializeOwned;

#[derive(Debug, serde::Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    error: String,
}

fn format_api_error(status: reqwest::StatusCode, body: &str) -> String {
    if let Ok(err) = serde_json::from_str::<ApiErrorBody>(body) {
        if !err.error.is_empty() {
            return err.error;
        }
    }

    match status.as_u16() {
        404 => "Resource not found".to_string(),
        409 => "Resource already exists or conflict".to_string(),
        422 => format!("Invalid request: {body}"),
        500 => "Internal server error — is nexad healthy?".to_string(),
        _ => format!("Request failed ({status}): {body}"),
    }
}

fn error_hint(status: reqwest::StatusCode) -> Option<&'static str> {
    match status.as_u16() {
        401 | 403 => Some("Check your authentication credentials"),
        404 => Some("Verify the resource name and project with: nexa pods / nexa deployments"),
        500 => Some("Check nexad logs: journalctl -u nexad -n 50"),
        502 | 503 => Some("nexad may be starting up. Retry in a few seconds"),
        _ => None,
    }
}

pub struct NexaClient {
    base_url: String,
    http: Client,
}

impl NexaClient {
    pub fn new(base_url: &str, token: Option<&str>) -> Self {
        let mut headers = HeaderMap::new();
        if let Some(t) = token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {t}")) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        let http = Client::builder()
            .default_headers(headers)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http,
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let msg = format_api_error(status, &body);
            if let Some(hint) = error_hint(status) {
                anyhow::bail!("{msg}\n  hint: {hint}");
            }
            anyhow::bail!("{msg}");
        }
        Ok(resp.json().await?)
    }

    pub async fn post_json<T: DeserializeOwned>(&self, path: &str, body: &str) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .body(body.to_string())
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let resp_body = resp.text().await.unwrap_or_default();
            let msg = format_api_error(status, &resp_body);
            if let Some(hint) = error_hint(status) {
                anyhow::bail!("{msg}\n  hint: {hint}");
            }
            anyhow::bail!("{msg}");
        }
        Ok(resp.json().await?)
    }

    pub async fn post_yaml<T: DeserializeOwned>(&self, path: &str, body: &str) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("content-type", "application/yaml")
            .body(body.to_string())
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let msg = format_api_error(status, &body);
            if let Some(hint) = error_hint(status) {
                anyhow::bail!("{msg}\n  hint: {hint}");
            }
            anyhow::bail!("{msg}");
        }
        Ok(resp.json().await?)
    }

    pub async fn post_empty_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let resp = self.http.post(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let msg = format_api_error(status, &body);
            if let Some(hint) = error_hint(status) {
                anyhow::bail!("{msg}\n  hint: {hint}");
            }
            anyhow::bail!("{msg}");
        }
        Ok(resp.json().await?)
    }

    pub async fn post_empty(&self, path: &str) -> Result<()> {
        let url = format!("{}{path}", self.base_url);
        let resp = self.http.post(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let msg = format_api_error(status, &body);
            if let Some(hint) = error_hint(status) {
                anyhow::bail!("{msg}\n  hint: {hint}");
            }
            anyhow::bail!("{msg}");
        }
        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{path}", self.base_url);
        let resp = self.http.delete(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let msg = format_api_error(status, &body);
            if let Some(hint) = error_hint(status) {
                anyhow::bail!("{msg}\n  hint: {hint}");
            }
            anyhow::bail!("{msg}");
        }
        Ok(())
    }

    /// Returns a reference to the underlying HTTP client.
    ///
    /// Use this when you need the shared client (with its configured auth
    /// headers, timeouts, and connection pool) outside the normal request
    /// helpers — for example, to consume an SSE event stream.
    pub fn http_client(&self) -> &Client {
        &self.http
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn get_stream(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("{}{path}", self.base_url);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let msg = format_api_error(status, &body);
            if let Some(hint) = error_hint(status) {
                anyhow::bail!("{msg}\n  hint: {hint}");
            }
            anyhow::bail!("{msg}");
        }
        Ok(resp)
    }

    pub async fn get_pods_for_deployment(
        &self,
        project: &str,
        deployment_name: &str,
    ) -> Result<Vec<nexa_core::domain::models::Pod>> {
        let pods: Vec<nexa_core::domain::models::Pod> =
            self.get(&format!("/api/v1/pods?project={project}")).await?;
        Ok(pods
            .into_iter()
            .filter(|p| p.deployment_name == deployment_name)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    // ---- error_hint: status code -> optional remediation hint ----

    #[test]
    fn hint_401_and_403_mention_auth() {
        let h = error_hint(StatusCode::UNAUTHORIZED).expect("401 should have a hint");
        assert!(h.to_lowercase().contains("auth"), "got: {h}");
        assert_eq!(
            error_hint(StatusCode::UNAUTHORIZED),
            error_hint(StatusCode::FORBIDDEN)
        );
    }

    #[test]
    fn hint_404_mentions_resource_lookup() {
        let h = error_hint(StatusCode::NOT_FOUND).expect("404 should have a hint");
        assert!(
            h.contains("nexa pods") || h.to_lowercase().contains("resource"),
            "got: {h}"
        );
    }

    #[test]
    fn hint_500_points_to_logs() {
        let h = error_hint(StatusCode::INTERNAL_SERVER_ERROR).expect("500 should have a hint");
        assert!(
            h.contains("journalctl") || h.to_lowercase().contains("log"),
            "got: {h}"
        );
    }

    #[test]
    fn hint_502_and_503_suggest_retry() {
        for code in [StatusCode::BAD_GATEWAY, StatusCode::SERVICE_UNAVAILABLE] {
            let h = error_hint(code).unwrap_or_else(|| panic!("{code} should have a hint"));
            assert!(h.to_lowercase().contains("retry"), "code {code}, got: {h}");
        }
        assert_eq!(
            error_hint(StatusCode::BAD_GATEWAY),
            error_hint(StatusCode::SERVICE_UNAVAILABLE)
        );
    }

    #[test]
    fn hint_none_for_unmapped_codes() {
        // 400, 409, 418, 200 have no hint in the match arms.
        assert!(error_hint(StatusCode::BAD_REQUEST).is_none());
        assert!(error_hint(StatusCode::CONFLICT).is_none());
        assert!(error_hint(StatusCode::OK).is_none());
        assert!(error_hint(StatusCode::from_u16(418).unwrap()).is_none());
    }

    // ---- format_api_error: prefers JSON body { "error": ... }, else code-based message ----

    #[test]
    fn format_prefers_json_error_field() {
        let body = r#"{"error":"deployment quota exceeded"}"#;
        // Even a 500 must surface the server-provided message verbatim.
        let msg = format_api_error(StatusCode::INTERNAL_SERVER_ERROR, body);
        assert_eq!(msg, "deployment quota exceeded");
    }

    #[test]
    fn format_ignores_empty_json_error_field() {
        // error present but empty -> fall through to the status-code branch.
        let msg = format_api_error(StatusCode::NOT_FOUND, r#"{"error":""}"#);
        assert_eq!(msg, "Resource not found");
    }

    #[test]
    fn format_404_default_message() {
        assert_eq!(
            format_api_error(StatusCode::NOT_FOUND, ""),
            "Resource not found"
        );
    }

    #[test]
    fn format_409_default_message() {
        let msg = format_api_error(StatusCode::CONFLICT, "<html>nope</html>");
        assert_eq!(msg, "Resource already exists or conflict");
    }

    #[test]
    fn format_422_includes_raw_body() {
        let msg = format_api_error(StatusCode::UNPROCESSABLE_ENTITY, "missing field: image");
        assert!(msg.starts_with("Invalid request:"), "got: {msg}");
        assert!(msg.contains("missing field: image"), "got: {msg}");
    }

    #[test]
    fn format_500_default_message() {
        let msg = format_api_error(StatusCode::INTERNAL_SERVER_ERROR, "plain text crash");
        assert!(msg.contains("Internal server error"), "got: {msg}");
    }

    #[test]
    fn format_unmapped_code_falls_back_to_generic() {
        // 502 is NOT in format_api_error's match -> generic branch with status + body.
        let msg = format_api_error(StatusCode::BAD_GATEWAY, "upstream down");
        assert!(msg.contains("Request failed"), "got: {msg}");
        assert!(msg.contains("upstream down"), "got: {msg}");
        assert!(msg.contains("502"), "got: {msg}");
    }

    #[test]
    fn format_invalid_json_body_is_treated_as_text() {
        // Not valid JSON -> serde_json::from_str fails -> status-code branch used.
        let msg = format_api_error(StatusCode::NOT_FOUND, "{not-json");
        assert_eq!(msg, "Resource not found");
    }

    // ---- base_url normalization (offline: Client::builder().build() does no network) ----

    #[test]
    fn base_url_trailing_slashes_trimmed() {
        let c = NexaClient::new("http://localhost:6443///", None);
        assert_eq!(c.base_url(), "http://localhost:6443");
    }

    #[test]
    fn base_url_without_trailing_slash_unchanged() {
        let c = NexaClient::new("http://10.0.0.1:6443", Some("tok"));
        assert_eq!(c.base_url(), "http://10.0.0.1:6443");
    }
}
