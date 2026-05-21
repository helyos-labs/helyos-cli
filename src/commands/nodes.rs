use anyhow::Result;
use crate::client::NexaClient;
use crate::output;

pub async fn nodes(_client: &NexaClient) -> Result<()> {
    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "nodes": [],
            "message": "Multi-node support coming in Phase 2",
        }));
        return Ok(());
    }

    output::print_warning("Multi-node support is coming in Phase 2");
    println!("  Currently running in single-node mode.");
    Ok(())
}
