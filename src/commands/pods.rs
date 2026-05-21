use anyhow::Result;
use nexa_core::domain::models::Pod;

use crate::client::NexaClient;
use crate::output;

pub async fn pods(client: &NexaClient, project: Option<&str>) -> Result<()> {
    let path = match project {
        Some(p) => format!("/api/v1/pods?project={p}"),
        None => "/api/v1/pods".to_string(),
    };

    let pods: Vec<Pod> = client.get(&path).await?;

    if output::is_json_mode() {
        let json_pods: Vec<serde_json::Value> = pods
            .iter()
            .map(|p| serde_json::to_value(p).unwrap())
            .collect();
        output::print_json(&json_pods);
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

    output::print_table(
        &["Name", "Project", "Deployment", "Status", "Image", "Age"],
        &rows,
    );

    Ok(())
}
