use anyhow::Result;
use nexa_core::domain::models::Deployment;

use crate::client::NexaClient;
use crate::output;

pub async fn scale(
    client: &NexaClient,
    project: Option<&str>,
    name: &str,
    replicas: u32,
) -> Result<()> {
    let project = project.unwrap_or("default");
    let path = format!("/api/v1/projects/{project}/deployments/{name}/scale");
    let body = serde_json::json!({ "replicas": replicas }).to_string();

    let spinner = output::Spinner::new(&format!("Scaling '{name}' to {replicas} replica{}...", if replicas > 1 { "s" } else { "" }));

    let deployment: Deployment = client.post_json(&path, &body).await?;

    if output::is_json_mode() {
        spinner.finish_clear();
        output::print_json(&serde_json::json!({"status": "ok", "deployment": deployment}));
        return Ok(());
    }

    spinner.finish_success(&format!(
        "Scaled '{name}' to {} replica{}",
        deployment.spec.replicas,
        if deployment.spec.replicas > 1 { "s" } else { "" }
    ));

    Ok(())
}
