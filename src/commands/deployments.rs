use anyhow::Result;
use nexa_core::domain::models::Deployment;

use crate::client::NexaClient;
use crate::output;

pub async fn deployments(client: &NexaClient, project: Option<&str>) -> Result<()> {
    let path = match project {
        Some(p) => format!("/api/v1/deployments?project={p}"),
        None => "/api/v1/deployments".to_string(),
    };

    let deployments: Vec<Deployment> = client.get(&path).await?;

    if output::is_json_mode() {
        let json_deployments: Vec<serde_json::Value> = deployments
            .iter()
            .map(|d| serde_json::to_value(d).unwrap())
            .collect();
        output::print_json(&json_deployments);
        return Ok(());
    }

    let rows: Vec<Vec<String>> = deployments
        .iter()
        .map(|d| {
            vec![
                d.name().to_string(),
                d.project().to_string(),
                format!("{:?}", d.status),
                d.spec.replicas.to_string(),
                d.spec.image.clone(),
                output::format_age(&d.created_at),
            ]
        })
        .collect();

    output::print_table(
        &["Name", "Project", "Status", "Replicas", "Image", "Age"],
        &rows,
    );

    Ok(())
}
