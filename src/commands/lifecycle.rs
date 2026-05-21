use anyhow::Result;

use crate::client::NexaClient;
use crate::output;

pub async fn stop(client: &NexaClient, project: Option<&str>, name: &str) -> Result<()> {
    let project = project.unwrap_or("default");
    let path = format!("/api/v1/projects/{project}/deployments/{name}/stop");

    client.post_empty(&path).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({"status": "ok", "action": "stop", "deployment": name}));
        return Ok(());
    }

    output::print_success(&format!("Deployment '{name}' stopped"));

    Ok(())
}

pub async fn remove(client: &NexaClient, project: Option<&str>, name: &str) -> Result<()> {
    let project = project.unwrap_or("default");
    let path = format!("/api/v1/projects/{project}/deployments/{name}");

    client.delete(&path).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({"status": "ok", "action": "remove", "deployment": name}));
        return Ok(());
    }

    output::print_success(&format!("Deployment '{name}' removed"));

    Ok(())
}
