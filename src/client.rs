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
