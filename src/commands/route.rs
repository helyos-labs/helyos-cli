use anyhow::Result;

use crate::client::NexaClient;
use crate::output;

pub async fn list(client: &NexaClient, project: Option<&str>) -> Result<()> {
    let path = match project {
        Some(p) => format!("/api/v1/routes?project={p}"),
        None => "/api/v1/routes".into(),
    };

    let routes: Vec<serde_json::Value> = client.get(&path).await?;

    if output::is_json_mode() {
        output::print_json(&routes);
        return Ok(());
    }

    let rows: Vec<Vec<String>> = routes
        .iter()
        .map(|r| {
            vec![
                r["domain"].as_str().unwrap_or("-").to_string(),
                r["project"].as_str().unwrap_or("-").to_string(),
                r["deployment"].as_str().unwrap_or("-").to_string(),
                r["tls_mode"].as_str().unwrap_or("none").to_string(),
                r["created_at"].as_str().unwrap_or("-").to_string(),
            ]
        })
        .collect();

    output::Panel::new(&format!("{} Routes", output::icon("event")))
        .count(&format!("{} total", routes.len()))
        .table(
            &["Domain", "Project", "Deployment", "TLS", "Created"],
            &rows,
        )
        .render();

    Ok(())
}

pub async fn add(
    client: &NexaClient,
    domain: &str,
    project: &str,
    deployment: &str,
    https: bool,
) -> Result<()> {
    let tls_mode = if https { "auto" } else { "none" };
    let body = serde_json::json!({
        "domain": domain,
        "project": project,
        "deployment": deployment,
        "tls_mode": tls_mode,
    })
    .to_string();

    let _: serde_json::Value = client.post_json("/api/v1/routes", &body).await?;
    output::print_success(&format!(
        "Route '{domain}' -> {project}/{deployment} ({tls_mode})"
    ));
    Ok(())
}

pub async fn remove(client: &NexaClient, domain: &str) -> Result<()> {
    client.delete(&format!("/api/v1/routes/{domain}")).await?;
    output::print_success(&format!("Route '{domain}' removed"));
    Ok(())
}

pub async fn import_cert(
    client: &NexaClient,
    domain: &str,
    cert_path: &str,
    key_path: &str,
) -> Result<()> {
    let cert_pem = std::fs::read_to_string(cert_path)?;
    let key_pem = std::fs::read_to_string(key_path)?;

    let body = serde_json::json!({
        "domain": domain,
        "cert_pem": cert_pem,
        "key_pem": key_pem,
    })
    .to_string();

    let _: serde_json::Value = client.post_json("/api/v1/certs/import", &body).await?;
    output::print_success(&format!("Certificate for '{domain}' imported"));
    Ok(())
}
