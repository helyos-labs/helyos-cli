use anyhow::Result;
use nexa_core::domain::models::Deployment;

use crate::client::NexaClient;
use crate::output;
use crate::output::Panel;

pub async fn deployments(client: &NexaClient, project: Option<&str>) -> Result<()> {
    let path = match project {
        Some(p) => format!("/api/v1/deployments?project={}", urlencoding::encode(p)),
        None => "/api/v1/deployments".to_string(),
    };

    let deployments: Vec<Deployment> = client.get(&path).await?;

    if output::is_json_mode() {
        output::print_json(&deployments);
        return Ok(());
    }

    let rows: Vec<Vec<String>> = deployments
        .iter()
        .map(|d| {
            vec![
                d.name().to_string(),
                d.project().to_string(),
                d.status.to_string(),
                d.spec.replicas.to_string(),
                d.spec.image.clone(),
                output::format_age(&d.created_at),
            ]
        })
        .collect();

    Panel::new(&format!("{} Deployments", output::icon("deploy")))
        .count(&format!("{} total", deployments.len()))
        .table(
            &["Name", "Project", "Status", "Replicas", "Image", "Age"],
            &rows,
        )
        .render();

    Ok(())
}
