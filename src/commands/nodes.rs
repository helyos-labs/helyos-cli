use anyhow::Result;
use nexa_core::domain::models::Node;

use crate::client::NexaClient;
use crate::output;
use crate::output::Panel;

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
            let cpu_pct = if n.resources.cpu_cores > 0.0 {
                ((n.resources.cpu_cores - n.resources.cpu_available) / n.resources.cpu_cores * 100.0)
                    as u32
            } else {
                0
            };
            let mem_pct = if n.resources.memory_bytes > 0 {
                ((n.resources.memory_bytes - n.resources.memory_available) as f64
                    / n.resources.memory_bytes as f64
                    * 100.0) as u32
            } else {
                0
            };
            vec![
                n.name.clone(),
                n.role.to_string(),
                n.status.to_string(),
                format_gauge(cpu_pct),
                format_gauge(mem_pct),
                n.resources.running_pods.to_string(),
                output::format_age(&n.last_heartbeat),
            ]
        })
        .collect();

    Panel::new(&format!("{} Nodes", output::icon("node")))
        .count(&format!("{} total", nodes.len()))
        .table(
            &["Name", "Role", "Status", "CPU", "MEM", "Pods", "Age"],
            &rows,
        )
        .render();

    Ok(())
}

fn format_gauge(percent: u32) -> String {
    let bar_width = 16;
    let filled = (percent as usize * bar_width) / 100;
    let empty = bar_width - filled;

    let color_name = if percent >= 80 {
        "red"
    } else if percent >= 60 {
        "yellow"
    } else {
        "green"
    };

    let fill_style = output::color(color_name);
    let empty_style = output::color("border-subtle");

    format!(
        "{}{} {:>3}%",
        fill_style.apply_to("█".repeat(filled)),
        empty_style.apply_to("░".repeat(empty)),
        percent,
    )
}

#[allow(dead_code)]
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}Gi", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.0}Mi", bytes as f64 / 1_048_576.0)
    } else {
        format!("{bytes}B")
    }
}
