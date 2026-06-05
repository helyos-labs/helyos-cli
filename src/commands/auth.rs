//! `helyos auth token …` + `helyos whoami` — manage server-side API tokens
//! (M1 daemon endpoints) and show the calling identity.

use anyhow::Result;

use crate::client::HelyosClient;
use crate::output;

pub async fn token_create(client: &HelyosClient, name: &str, ttl_secs: Option<i64>) -> Result<()> {
    let body = match ttl_secs {
        Some(s) => serde_json::json!({ "name": name, "ttl_secs": s }).to_string(),
        None => serde_json::json!({ "name": name }).to_string(),
    };
    let resp: serde_json::Value = client.post_json("/api/v1/tokens", &body).await?;

    if output::is_json_mode() {
        output::print_json(&resp);
        return Ok(());
    }
    let token = resp["token"].as_str().unwrap_or("unknown");
    output::print_success(&format!("Created API token '{name}'"));
    println!("\nToken (shown once — save it now):\n  {token}\n");
    println!("Use it from another machine with:");
    println!("  helyos login <server> --token {token}");
    Ok(())
}

pub async fn token_list(client: &HelyosClient) -> Result<()> {
    let tokens: serde_json::Value = client.get("/api/v1/tokens").await?;
    if output::is_json_mode() {
        output::print_json(&tokens);
        return Ok(());
    }
    let empty = vec![];
    let arr = tokens.as_array().unwrap_or(&empty);
    let headers = ["name", "scope", "expires", "last used", "revoked"];
    let rows: Vec<Vec<String>> = arr
        .iter()
        .map(|t| {
            vec![
                t["name"].as_str().unwrap_or("-").to_string(),
                t["scope"].as_str().unwrap_or("-").to_string(),
                t["expires_at"].as_str().unwrap_or("never").to_string(),
                t["last_used_at"].as_str().unwrap_or("never").to_string(),
                if t["revoked_at"].is_string() { "yes".into() } else { "no".into() },
            ]
        })
        .collect();
    output::print_table(&headers, &rows);
    Ok(())
}

pub async fn token_revoke(client: &HelyosClient, name: &str) -> Result<()> {
    client
        .delete(&format!("/api/v1/tokens/{}", urlencoding::encode(name)))
        .await?;
    output::print_success(&format!("Revoked API token '{name}'"));
    Ok(())
}

pub async fn whoami(client: &HelyosClient) -> Result<()> {
    let resp: serde_json::Value = client.get("/api/v1/whoami").await?;
    if output::is_json_mode() {
        output::print_json(&resp);
        return Ok(());
    }
    println!("name:  {}", resp["name"].as_str().unwrap_or("-"));
    println!("scope: {}", resp["scope"].as_str().unwrap_or("-"));
    if let Some(e) = resp["expires_at"].as_str() {
        println!("expires: {e}");
    }
    if let Some(l) = resp["last_used_at"].as_str() {
        println!("last used: {l}");
    }
    Ok(())
}
