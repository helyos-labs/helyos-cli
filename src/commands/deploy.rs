use anyhow::Result;
use nexa_core::config::parse_deployment_file;
use nexa_core::domain::models::{Deployment, PodStatus};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::client::NexaClient;
use crate::output::{self, Spinner};

pub async fn deploy(client: &NexaClient, file: &str) -> Result<()> {
    let path = Path::new(file);
    if !path.exists() {
        anyhow::bail!("file not found: {file}");
    }

    let yaml = std::fs::read_to_string(path)?;
    let spec = parse_deployment_file(path).map_err(|e| {
        output::print_error_with_hint(
            &format!("Invalid deployment spec: {e}"),
            "Run 'nexa init' to generate a valid template",
        );
        e
    })?;
    let project = spec.project.clone();
    let name = spec.deployment.name.clone();
    let replicas = spec.replicas;

    let spinner = if !output::is_json_mode() {
        Some(Spinner::new(&format!(
            "Deploying {name} to project '{project}'..."
        )))
    } else {
        None
    };

    let deployment: Deployment = client.post_yaml("/api/v1/deploy", &yaml).await?;

    if let Some(s) = &spinner {
        s.finish_clear();
    }

    if output::is_json_mode() {
        let pods = poll_until_ready(client, &project, &name, replicas).await?;
        output::print_json(&serde_json::json!({
            "status": "ok",
            "deployment": deployment,
            "pods": pods,
        }));
        return Ok(());
    }

    let timeout = Duration::from_secs(60);
    let start = Instant::now();
    let mut seen_running: std::collections::HashSet<String> = std::collections::HashSet::new();

    loop {
        if start.elapsed() > timeout {
            output::print_warning(&format!(
                "Timed out waiting for all pods (60s). Check status with: nexa pods -p {project}"
            ));
            anyhow::bail!("timed out waiting for deployment '{name}'");
        }

        let pods = client.get_pods_for_deployment(&project, &name).await?;

        for pod in &pods {
            let pod_name = pod.container_name();
            if pod.status == PodStatus::Running && !seen_running.contains(&pod_name) {
                output::print_success(&format!("Pod {} running", pod_name));
                seen_running.insert(pod_name);
            }
        }

        let running = pods
            .iter()
            .filter(|p| p.status == PodStatus::Running)
            .count() as u32;
        let failed = pods.iter().any(|p| is_terminal_failure(&p.status));

        if running >= replicas {
            output::print_success(&format!(
                "Deployment '{name}' is running ({running}/{replicas} replicas)"
            ));
            return Ok(());
        }

        if failed {
            output::print_error(&format!(
                "Deployment '{name}' has failed pods. Check: nexa pods -p {project}"
            ));
            anyhow::bail!("deployment '{name}' has failed pods");
        }

        sleep(Duration::from_millis(500)).await;
    }
}

fn is_terminal_failure(status: &PodStatus) -> bool {
    matches!(status, PodStatus::Failed | PodStatus::CrashLoopBackoff)
}

async fn poll_until_ready(
    client: &NexaClient,
    project: &str,
    name: &str,
    replicas: u32,
) -> Result<Vec<nexa_core::domain::models::Pod>> {
    let timeout = Duration::from_secs(60);
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            return client.get_pods_for_deployment(project, name).await;
        }

        let pods = client.get_pods_for_deployment(project, name).await?;
        let running = pods
            .iter()
            .filter(|p| p.status == PodStatus::Running)
            .count() as u32;
        if running >= replicas || pods.iter().any(|p| is_terminal_failure(&p.status)) {
            return Ok(pods);
        }

        sleep(Duration::from_millis(500)).await;
    }
}
