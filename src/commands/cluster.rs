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

pub async fn get_scheduler_config(client: &NexaClient) -> Result<()> {
    let config: serde_json::Value = client.get("/api/v1/cluster/scheduler").await?;

    if output::is_json_mode() {
        output::print_json(&config);
        return Ok(());
    }

    println!("Scheduler configuration:");
    println!("  Strategy: {}", config["strategy"].as_str().unwrap_or("unknown"));
    println!("  Weights:");
    if let Some(weights) = config.get("weights") {
        println!("    cpu:     {}", weights["cpu"]);
        println!("    memory:  {}", weights["memory"]);
        println!("    load:    {}", weights["load"]);
        println!("    failure: {}", weights["failure"]);
    }
    Ok(())
}

pub async fn set_cluster_config(client: &NexaClient, key: &str, value: &str) -> Result<()> {
    let body = if key == "scheduler" {
        serde_json::json!({ "strategy": value }).to_string()
    } else if let Some(weight_name) = key.strip_prefix("scheduler.weights.") {
        let num: f64 = value
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid weight value: {value}"))?;
        serde_json::json!({ "weights": { "name": weight_name, "value": num } }).to_string()
    } else {
        anyhow::bail!("unknown config key: {key}. Supported: scheduler, scheduler.weights.<name>");
    };

    let config: serde_json::Value = client.post_json("/api/v1/cluster/scheduler", &body).await?;
    output::print_success(&format!("Scheduler config updated: {}", config["strategy"].as_str().unwrap_or("unknown")));
    Ok(())
}
