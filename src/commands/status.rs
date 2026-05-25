use anyhow::Result;
use nexa_core::domain::models::{Deployment, DeploymentStatus, Pod, PodStatus, Project};

use crate::client::NexaClient;
use crate::output;
use crate::output::Panel;

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

    let status_str = output::status_dot("running");

    Panel::new(&format!("{} Cluster Status", output::icon("cluster")))
        .kv(&[
            ("Mode", "single-node"),
            ("Status", &status_str),
            ("Projects", &projects.len().to_string()),
            (
                "Deployments",
                &format!(
                    "{} running · {} stopped",
                    running_deployments, stopped_deployments
                ),
            ),
            (
                "Pods",
                &format!("{} running · {} restarting", running_pods, restarting_pods),
            ),
        ])
        .render();

    Ok(())
}
