use anyhow::Result;
use helyos_core::domain::models::Pod;

use crate::client::HelyosClient;
use crate::output;
use crate::output::Panel;

pub async fn pods(client: &HelyosClient, project: Option<&str>) -> Result<()> {
    let path = match project {
        Some(p) => format!("/api/v1/pods?project={}", urlencoding::encode(p)),
        None => "/api/v1/pods".to_string(),
    };

    let pods: Vec<Pod> = client.get(&path).await?;

    if output::is_json_mode() {
        output::print_json(&pods);
        return Ok(());
    }

    let rows: Vec<Vec<String>> = pods
        .iter()
        .map(|p| {
            vec![
                p.container_name(),
                p.project.clone(),
                p.deployment_name.clone(),
                p.status.to_string(),
                p.image.clone(),
                output::format_age(&p.created_at),
            ]
        })
        .collect();

    Panel::new(&format!("{} Pods", output::icon("pod")))
        .count(&format!("{} total", pods.len()))
        .table(
            &["Name", "Project", "Deployment", "Status", "Image", "Age"],
            &rows,
        )
        .render();

    Ok(())
}
