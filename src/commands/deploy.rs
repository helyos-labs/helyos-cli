use anyhow::Result;
use helyos_core::config::parse_deployment_file;
use helyos_core::domain::models::{Deployment, PodStatus};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::client::HelyosClient;
use crate::output::deploy::{DeployStep, render_deploy_panel};
use crate::output::{self, Spinner};

pub async fn deploy(client: &HelyosClient, file: &str, timeout_secs: u64) -> Result<()> {
    let path = Path::new(file);
    if !path.exists() {
        anyhow::bail!("file not found: {file}");
    }

    let yaml = std::fs::read_to_string(path)?;
    let spec = parse_deployment_file(path).map_err(|e| {
        output::print_error_with_hint(
            &format!("Invalid deployment spec: {e}"),
            "Run 'helyos init' to generate a valid template",
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
        let pods = poll_until_ready(client, &project, &name, replicas, timeout_secs).await?;
        output::print_json(&serde_json::json!({
            "status": "ok",
            "deployment": deployment,
            "pods": pods,
        }));
        return Ok(());
    }

    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    let mut steps: Vec<DeployStep> = vec![DeployStep {
        icon: format!("{}", output::color("green").apply_to("✓")),
        label: "Image".to_string(),
        detail: format!("{} pulled", deployment.spec.image),
    }];

    loop {
        if start.elapsed() > timeout {
            output::print_warning(&format!(
                "Timed out waiting for all pods ({timeout_secs}s). Check: helyos pods -p {project}"
            ));
            anyhow::bail!("timed out waiting for deployment '{name}'");
        }

        let pods = client.get_pods_for_deployment(&project, &name).await?;

        for pod in &pods {
            let pod_name = pod.container_name();
            let already = steps.iter().any(|s| s.detail.contains(&pod_name));
            if pod.status == PodStatus::Running && !already {
                steps.push(DeployStep {
                    icon: format!("{}", output::color("green").apply_to("✓")),
                    label: "Pod".to_string(),
                    detail: format!(
                        "{} {}",
                        pod_name,
                        output::color("green").apply_to("running")
                    ),
                });
            }
        }

        let running = pods
            .iter()
            .filter(|p| p.status == PodStatus::Running)
            .count() as u32;
        let failed = pods.iter().any(|p| is_terminal_failure(&p.status));

        if running >= replicas {
            let elapsed = format!("{:.1}s", start.elapsed().as_secs_f64());
            let status = format!("{}", output::color("green").apply_to("● Deployed"));
            let timing = format!("{running}/{replicas} pods ready · {elapsed}");
            render_deploy_panel(&name, &steps, &status, &timing);
            return Ok(());
        }

        if failed {
            let status = format!("{}", output::color("red").apply_to("● Failed"));
            render_deploy_panel(&name, &steps, &status, "");
            anyhow::bail!("deployment '{name}' has failed pods");
        }

        sleep(Duration::from_millis(500)).await;
    }
}

fn is_terminal_failure(status: &PodStatus) -> bool {
    matches!(status, PodStatus::Failed | PodStatus::CrashLoopBackoff)
}

async fn poll_until_ready(
    client: &HelyosClient,
    project: &str,
    name: &str,
    replicas: u32,
    timeout_secs: u64,
) -> Result<Vec<helyos_core::domain::models::Pod>> {
    let timeout = Duration::from_secs(timeout_secs);
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
