use anyhow::Result;

use crate::client::NexaClient;

pub async fn top(client: NexaClient, server_url: &str, token: Option<&str>) -> Result<()> {
    crate::tui::run(client, server_url, token).await
}
