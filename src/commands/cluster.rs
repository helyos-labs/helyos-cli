use anyhow::Result;

use crate::client::NexaClient;
use crate::output;

pub async fn init(client: &NexaClient) -> Result<()> {
    let resp: serde_json::Value = client.post_empty_json("/api/v1/cluster/init").await?;
    let token = resp["token"].as_str().unwrap_or("unknown");

    if output::is_json_mode() {
        output::print_json(&resp);
        return Ok(());
    }

    output::print_success("Cluster initialized");
    println!("\nJoin token (save this — it won't be shown again):\n");
    println!("  {token}\n");
    println!("Join workers with:");
    println!("  nexad --mode worker --join <master-ip>:6444 --token {token}");
    Ok(())
}

pub async fn token_show(client: &NexaClient) -> Result<()> {
    let resp: serde_json::Value = client.get("/api/v1/cluster/token").await?;
    let token = resp["token"].as_str().unwrap_or("not set");

    if output::is_json_mode() {
        output::print_json(&resp);
        return Ok(());
    }

    println!("{token}");
    Ok(())
}

pub async fn token_rotate(client: &NexaClient) -> Result<()> {
    let resp: serde_json::Value = client.post_empty_json("/api/v1/cluster/token/rotate").await?;
    let token = resp["token"].as_str().unwrap_or("unknown");

    if output::is_json_mode() {
        output::print_json(&resp);
        return Ok(());
    }

    output::print_success("Token rotated");
    println!("\nNew join token:\n  {token}");
    Ok(())
}
