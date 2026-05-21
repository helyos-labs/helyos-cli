use anyhow::Result;
use futures::StreamExt;
use reqwest::Response;

use crate::client::NexaClient;

pub async fn logs(
    client: &NexaClient,
    project: Option<&str>,
    name: &str,
    tail: Option<u64>,
) -> Result<()> {
    let project = project.unwrap_or("default");
    let mut path = format!("/api/v1/projects/{project}/deployments/{name}/logs");
    if let Some(t) = tail {
        path.push_str(&format!("?tail={t}"));
    }

    let resp: Response = client.get_stream(&path).await?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        let text = String::from_utf8_lossy(&bytes);
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                print!("{data}");
            }
        }
    }

    Ok(())
}
