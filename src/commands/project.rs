use anyhow::Result;
use nexa_core::domain::models::Project;

use crate::client::NexaClient;
use crate::output;

pub async fn list_projects(client: &NexaClient) -> Result<()> {
    let projects: Vec<Project> = client.get("/api/v1/projects").await?;

    if output::is_json_mode() {
        output::print_json(&projects);
        return Ok(());
    }

    let rows: Vec<Vec<String>> = projects
        .iter()
        .map(|p| vec![p.name.clone(), output::format_age(&p.created_at)])
        .collect();

    output::print_table(&["Name", "Age"], &rows);

    Ok(())
}

pub async fn create_project(client: &NexaClient, name: &str) -> Result<()> {
    let body = serde_json::json!({ "name": name }).to_string();
    let project: Project = client.post_json("/api/v1/projects", &body).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({"status": "ok", "project": project}));
        return Ok(());
    }

    output::print_success(&format!("Project '{name}' created"));

    Ok(())
}

pub async fn suspend(client: &NexaClient, name: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{name}/suspend");
    client.post_empty(&path).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "status": "ok",
            "project": name,
            "action": "suspended",
        }));
        return Ok(());
    }

    output::print_success(&format!("Project '{name}' suspended"));
    Ok(())
}

pub async fn resume(client: &NexaClient, name: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{name}/resume");
    client.post_empty(&path).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "status": "ok",
            "project": name,
            "action": "resumed",
        }));
        return Ok(());
    }

    output::print_success(&format!("Project '{name}' resumed"));
    Ok(())
}

pub async fn delete_project(client: &NexaClient, name: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{name}");
    client.delete(&path).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "status": "ok",
            "project": name,
            "action": "deleted",
        }));
        return Ok(());
    }

    output::print_success(&format!("Project '{name}' deleted"));
    Ok(())
}
