use anyhow::Result;

use crate::client::HelyosClient;

pub async fn top(client: HelyosClient, server_url: &str, token: Option<&str>) -> Result<()> {
    crate::tui::run(client, server_url, token).await
}
