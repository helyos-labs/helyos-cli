use anyhow::Result;

use crate::client::NexaClient;
use crate::output;

pub async fn set(client: &NexaClient, project: &str, name: &str, value: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{project}/secrets");
    let body = serde_json::json!({
        "name": name,
        "value": value,
    })
    .to_string();

    client.post_json::<serde_json::Value>(&path, &body).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "status": "ok",
            "secret": name,
            "project": project,
        }));
        return Ok(());
    }

    output::print_success(&format!("Secret '{name}' set in project '{project}'"));
    Ok(())
}

pub async fn list(client: &NexaClient, project: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{project}/secrets");
    let secrets: Vec<String> = client.get(&path).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "project": project,
            "secrets": secrets,
        }));
        return Ok(());
    }

    let rows: Vec<Vec<String>> = secrets
        .iter()
        .map(|s| vec![s.clone(), project.to_string()])
        .collect();

    output::print_table(&["Name", "Project"], &rows);
    Ok(())
}

pub async fn remove(client: &NexaClient, project: &str, name: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{project}/secrets/{name}");
    client.delete(&path).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "status": "ok",
            "deleted": name,
            "project": project,
        }));
        return Ok(());
    }

    output::print_success(&format!("Secret '{name}' removed from project '{project}'"));
    Ok(())
}
