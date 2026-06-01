use anyhow::Result;
use chrono::Local;
use futures::StreamExt;
use reqwest::Response;

use crate::client::NexaClient;
use crate::output;

pub async fn logs(
    client: &NexaClient,
    project: Option<&str>,
    name: &str,
    tail: Option<u64>,
) -> Result<()> {
    let project = project.unwrap_or("default");
    let mut path = format!(
        "/api/v1/projects/{}/deployments/{}/logs",
        urlencoding::encode(project),
        urlencoding::encode(name),
    );
    if let Some(t) = tail {
        path.push_str(&format!("?tail={t}"));
    }

    let resp: Response = client.get_stream(&path).await?;
    let mut stream = resp.bytes_stream();

    let time_style = output::color("text-secondary");
    let name_style = output::color("accent");
    let sep_style = output::color("border");

    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        let text = String::from_utf8_lossy(&bytes);
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if output::is_json_mode() {
                    println!("{data}");
                } else {
                    let timestamp = Local::now().format("%H:%M:%S");
                    println!(
                        "{} {} {} {}",
                        time_style.apply_to(timestamp),
                        name_style.apply_to(name),
                        sep_style.apply_to("│"),
                        data,
                    );
                }
            }
        }
    }

    Ok(())
}
