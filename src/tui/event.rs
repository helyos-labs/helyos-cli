use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};
use tokio::sync::mpsc;

pub enum AppEvent {
    Tick,
    Key(KeyEvent),
    ClusterEvent(super::app::ClusterEvent),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration, server_url: &str, token: Option<&str>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let key_tx = tx.clone();
        let tick = tick_rate;
        std::thread::spawn(move || {
            loop {
                if event::poll(tick).unwrap_or(false) {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key_tx.send(AppEvent::Key(key)).is_err() {
                            return;
                        }
                    }
                } else if key_tx.send(AppEvent::Tick).is_err() {
                    return;
                }
            }
        });

        let sse_tx = tx.clone();
        let url = format!("{}/api/v1/events", server_url.trim_end_matches('/'));
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(t) = token {
            if let Ok(val) = reqwest::header::HeaderValue::from_str(&format!("Bearer {t}")) {
                headers.insert(reqwest::header::AUTHORIZATION, val);
            }
        }
        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap_or_default();
            loop {
                match client.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        use futures::StreamExt;
                        let mut stream = resp.bytes_stream();
                        let mut buffer = String::new();
                        while let Some(chunk) = stream.next().await {
                            if let Ok(bytes) = chunk {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));
                                while let Some(pos) = buffer.find("\n\n") {
                                    let msg = buffer[..pos].to_string();
                                    buffer = buffer[pos + 2..].to_string();
                                    if let Some(data) = msg.strip_prefix("data: ") {
                                        if let Ok(evt) =
                                            serde_json::from_str::<super::app::ClusterEvent>(data)
                                        {
                                            if sse_tx.send(AppEvent::ClusterEvent(evt)).is_err() {
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> anyhow::Result<AppEvent> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("event channel closed"))
    }
}
