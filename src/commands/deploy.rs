use anyhow::Result;
use nexa_core::config::parse_deployment_file;
use nexa_core::domain::models::Deployment;
use std::path::Path;

use crate::client::NexaClient;
use crate::output;

pub async fn deploy(client: &NexaClient, file: &str) -> Result<()> {
    let path = Path::new(file);
    if !path.exists() {
        anyhow::bail!("file not found: {file}");
    }

    let spec = parse_deployment_file(path)?;
    let project = spec.project.clone();
    let name = spec.deployment.name.clone();
    let replicas = spec.replicas;

    let spinner = if !output::is_json_mode() {
        Some(output::Spinner::new(&format!(
            "Deploying {name} ({replicas} replica{}) to project '{project}'...",
            if replicas > 1 { "s" } else { "" }
        )))
    } else {
        None
    };

    let yaml = std::fs::read_to_string(path)?;
    let deployment: Deployment = client.post_yaml("/api/v1/deploy", &yaml).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({"status": "ok", "deployment": deployment}));
        return Ok(());
    }

    if let Some(s) = spinner {
        s.finish_success(&format!(
            "Deployment '{name}' is {} (id: {})",
            deployment.status,
            &deployment.id.to_string()[..8]
        ));
    }

    Ok(())
}
