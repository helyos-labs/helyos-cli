use anyhow::Result;
use nexa_core::domain::models::{Deployment, DeploymentStatus, Pod, PodStatus, Project};

use crate::client::NexaClient;
use crate::output;

pub async fn status(client: &NexaClient) -> Result<()> {
    let projects: Vec<Project> = client.get("/api/v1/projects").await?;
    let deployments: Vec<Deployment> = client.get("/api/v1/deployments").await?;
    let pods: Vec<Pod> = client.get("/api/v1/pods").await?;

    let running_deployments = deployments
        .iter()
        .filter(|d| d.status == DeploymentStatus::Running)
        .count();
    let stopped_deployments = deployments
        .iter()
        .filter(|d| d.status == DeploymentStatus::Stopped)
        .count();

    let running_pods = pods
        .iter()
        .filter(|p| p.status == PodStatus::Running)
        .count();
    let restarting_pods = pods
        .iter()
        .filter(|p| p.status == PodStatus::Restarting)
        .count();

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "cluster": "single-node",
            "projects": projects.len(),
            "deployments": {
                "total": deployments.len(),
                "running": running_deployments,
                "stopped": stopped_deployments,
            },
            "pods": {
                "total": pods.len(),
                "running": running_pods,
                "restarting": restarting_pods,
            },
        }));
        return Ok(());
    }

    output::print_header("Cluster Status");
    println!();
    output::print_kv("Cluster", "single-node");
    output::print_kv("Projects", &projects.len().to_string());
    output::print_kv(
        "Deployments",
        &format!(
            "{} ({} running, {} stopped)",
            deployments.len(),
            running_deployments,
            stopped_deployments,
        ),
    );
    output::print_kv(
        "Pods",
        &format!(
            "{} ({} running, {} restarting)",
            pods.len(),
            running_pods,
            restarting_pods,
        ),
    );

    Ok(())
}
