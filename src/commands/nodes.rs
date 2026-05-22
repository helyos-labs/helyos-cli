use anyhow::Result;
use nexa_core::domain::models::Node;

use crate::client::NexaClient;
use crate::output;

pub async fn nodes(client: &NexaClient) -> Result<()> {
    let nodes: Vec<Node> = client.get("/api/v1/nodes").await?;

    if output::is_json_mode() {
        output::print_json(&nodes);
        return Ok(());
    }

    if nodes.is_empty() {
        output::print_warning("No nodes registered. Running in single-node mode.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = nodes
        .iter()
        .map(|n| {
            vec![
                n.name.clone(),
                n.role.to_string(),
                n.status.to_string(),
                n.address.clone(),
                format!("{:.1}", n.resources.cpu_cores),
                format_bytes(n.resources.memory_bytes),
                n.resources.running_pods.to_string(),
                output::format_age(&n.last_heartbeat),
            ]
        })
        .collect();

    output::print_table(
        &[
            "NAME", "ROLE", "STATUS", "ADDRESS", "CPUS", "MEMORY", "PODS", "AGE",
        ],
        &rows,
    );

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}Gi", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.0}Mi", bytes as f64 / 1_048_576.0)
    } else {
        format!("{bytes}B")
    }
}
