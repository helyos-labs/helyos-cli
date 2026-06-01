use anyhow::Result;

use crate::client::NexaClient;
use crate::output;

pub async fn drain(client: &NexaClient, name: &str) -> Result<()> {
    client
        .post_empty(&format!("/api/v1/nodes/{}/drain", urlencoding::encode(name)))
        .await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({ "node": name, "status": "draining" }));
        return Ok(());
    }

    output::print_success(&format!("Node '{name}' is draining"));
    Ok(())
}

pub async fn remove(client: &NexaClient, name: &str) -> Result<()> {
    client.delete(&format!("/api/v1/nodes/{}", urlencoding::encode(name))).await?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({ "node": name, "status": "removed" }));
        return Ok(());
    }

    output::print_success(&format!("Node '{name}' removed"));
    Ok(())
}
