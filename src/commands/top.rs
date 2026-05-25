use anyhow::Result;

use crate::client::NexaClient;

pub async fn top(client: NexaClient) -> Result<()> {
    crate::tui::run(client).await
}
