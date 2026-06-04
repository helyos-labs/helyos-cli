use anyhow::Result;

use crate::client::HelyosClient;
use crate::output;

pub async fn stop(client: &HelyosClient, project: Option<&str>, name: &str) -> Result<()> {
    let project = project.unwrap_or("default");
    let path = format!(
        "/api/v1/projects/{}/deployments/{}/stop",
        urlencoding::encode(project),
        urlencoding::encode(name),
    );

    client.post_empty(&path).await?;

    if output::is_json_mode() {
        output::print_json(
            &serde_json::json!({"status": "ok", "action": "stop", "project": project, "deployment": name}),
        );
        return Ok(());
    }

    output::print_success(&format!("Deployment '{name}' stopped"));

    Ok(())
}

pub async fn remove(client: &HelyosClient, project: Option<&str>, name: &str) -> Result<()> {
    let project = project.unwrap_or("default");
    let path = format!(
        "/api/v1/projects/{}/deployments/{}",
        urlencoding::encode(project),
        urlencoding::encode(name),
    );

    client.delete(&path).await?;

    if output::is_json_mode() {
        output::print_json(
            &serde_json::json!({"status": "ok", "action": "remove", "project": project, "deployment": name}),
        );
        return Ok(());
    }

    output::print_success(&format!("Deployment '{name}' removed"));

    Ok(())
}
