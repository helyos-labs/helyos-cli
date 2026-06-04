use anyhow::Result;
use helyos_core::domain::models::Project;

use crate::client::HelyosClient;
use crate::output;

pub async fn list_projects(client: &HelyosClient) -> Result<()> {
    let projects: Vec<Project> = client.get("/api/v1/projects").await?;

    if output::is_json_mode() {
        output::print_json(&projects);
        return Ok(());
    }

    let rows: Vec<Vec<String>> = projects
        .iter()
        .map(|p| vec![p.name.clone(), output::format_age(&p.created_at)])
        .collect();

    output::Panel::new(&format!("{} Projects", output::icon("cluster")))
        .count(&format!("{} total", projects.len()))
        .table(&["Name", "Age"], &rows)
        .render();

    Ok(())
}

pub async fn create_project(client: &HelyosClient, name: &str) -> Result<()> {
    let body = serde_json::json!({ "name": name }).to_string();
    let project: Project = client.post_json("/api/v1/projects", &body).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({"status": "ok", "project": project}));
        return Ok(());
    }

    output::print_success(&format!("Project '{name}' created"));

    Ok(())
}

pub async fn suspend(client: &HelyosClient, name: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{}/suspend", urlencoding::encode(name));
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

pub async fn resume(client: &HelyosClient, name: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{}/resume", urlencoding::encode(name));
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

pub async fn delete_project(client: &HelyosClient, name: &str) -> Result<()> {
    let path = format!("/api/v1/projects/{}", urlencoding::encode(name));
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
