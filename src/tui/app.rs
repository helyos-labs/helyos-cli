use serde::Deserialize;

use crate::client::NexaClient;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Pods,
    Nodes,
    Events,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct NodeStats {
    pub name: String,
    pub role: String,
    pub status: String,
    pub cpu_cores: f64,
    pub cpu_usage_percent: f64,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub pod_count: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ClusterEvent {
    pub timestamp: String,
    pub kind: String,
    pub name: String,
    pub action: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    ConfirmDelete(String),
    ScaleInput(String, String),
    LogView(String, Vec<String>),
}

pub struct App {
    pub client: NexaClient,
    pub active_panel: ActivePanel,
    pub pods: Vec<nexa_core::domain::models::Pod>,
    pub deployments: Vec<nexa_core::domain::models::Deployment>,
    pub nodes: Vec<NodeStats>,
    pub events: Vec<ClusterEvent>,
    pub pod_cursor: usize,
    pub node_cursor: usize,
    pub event_cursor: usize,
    pub connected: bool,
    pub show_help: bool,
    pub input_mode: InputMode,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(client: NexaClient) -> Self {
        Self {
            client,
            active_panel: ActivePanel::Pods,
            pods: Vec::new(),
            deployments: Vec::new(),
            nodes: Vec::new(),
            events: Vec::new(),
            pod_cursor: 0,
            node_cursor: 0,
            event_cursor: 0,
            connected: false,
            show_help: false,
            input_mode: InputMode::Normal,
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        let pods_result = self
            .client
            .get::<Vec<nexa_core::domain::models::Pod>>("/api/v1/pods")
            .await;
        let deployments_result = self
            .client
            .get::<Vec<nexa_core::domain::models::Deployment>>("/api/v1/deployments")
            .await;
        let nodes_result = self
            .client
            .get::<Vec<NodeStats>>("/api/v1/nodes/stats")
            .await;

        match (pods_result, deployments_result, nodes_result) {
            (Ok(pods), Ok(deployments), Ok(nodes)) => {
                self.pods = pods;
                self.deployments = deployments;
                self.nodes = nodes;
                self.connected = true;
            }
            _ => {
                self.connected = false;
            }
        }

        if !self.pods.is_empty() {
            self.pod_cursor = self.pod_cursor.min(self.pods.len() - 1);
        }
        if !self.nodes.is_empty() {
            self.node_cursor = self.node_cursor.min(self.nodes.len() - 1);
        }
    }

    pub fn cursor_up(&mut self) {
        match self.active_panel {
            ActivePanel::Pods => {
                self.pod_cursor = self.pod_cursor.saturating_sub(1);
            }
            ActivePanel::Nodes => {
                self.node_cursor = self.node_cursor.saturating_sub(1);
            }
            ActivePanel::Events => {
                self.event_cursor = self.event_cursor.saturating_sub(1);
            }
        }
    }

    pub fn cursor_down(&mut self) {
        match self.active_panel {
            ActivePanel::Pods => {
                if !self.pods.is_empty() {
                    self.pod_cursor = (self.pod_cursor + 1).min(self.pods.len() - 1);
                }
            }
            ActivePanel::Nodes => {
                if !self.nodes.is_empty() {
                    self.node_cursor = (self.node_cursor + 1).min(self.nodes.len() - 1);
                }
            }
            ActivePanel::Events => {
                if !self.events.is_empty() {
                    self.event_cursor = (self.event_cursor + 1).min(self.events.len() - 1);
                }
            }
        }
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Pods => ActivePanel::Nodes,
            ActivePanel::Nodes => ActivePanel::Events,
            ActivePanel::Events => ActivePanel::Pods,
        };
    }

    pub fn prev_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Pods => ActivePanel::Events,
            ActivePanel::Nodes => ActivePanel::Pods,
            ActivePanel::Events => ActivePanel::Nodes,
        };
    }

    pub fn selected_pod(&self) -> Option<&nexa_core::domain::models::Pod> {
        self.pods.get(self.pod_cursor)
    }

    pub async fn delete_selected_pod(&mut self) {
        if let Some(pod) = self.pods.get(self.pod_cursor) {
            let project = pod.project.clone();
            let deployment = pod.deployment_name.clone();
            let name = pod.container_name();
            let path = format!("/api/v1/projects/{project}/deployments/{deployment}");
            match self.client.delete(&path).await {
                Ok(()) => {
                    self.status_message = Some(format!("✓ Deleted {name}"));
                    self.refresh().await;
                }
                Err(e) => {
                    self.status_message = Some(format!("✗ {e}"));
                }
            }
        }
        self.input_mode = InputMode::Normal;
    }

    pub async fn scale_deployment(&mut self, replicas_str: &str) {
        if let Ok(replicas) = replicas_str.parse::<u32>() {
            if let Some(pod) = self.pods.get(self.pod_cursor) {
                let project = pod.project.clone();
                let deployment = pod.deployment_name.clone();
                let path = format!("/api/v1/projects/{project}/deployments/{deployment}/scale");
                let body = serde_json::json!({ "replicas": replicas }).to_string();
                match self
                    .client
                    .post_json::<nexa_core::domain::models::Deployment>(&path, &body)
                    .await
                {
                    Ok(d) => {
                        self.status_message = Some(format!(
                            "✓ Scaled {} to {} replicas",
                            deployment, d.spec.replicas
                        ));
                        self.refresh().await;
                    }
                    Err(e) => {
                        self.status_message = Some(format!("✗ {e}"));
                    }
                }
            }
        }
        self.input_mode = InputMode::Normal;
    }

    pub async fn open_log_view(&mut self) {
        if let Some(pod) = self.pods.get(self.pod_cursor) {
            let name = pod.container_name();
            let project = &pod.project;
            let deployment = &pod.deployment_name;
            let path = format!("/api/v1/projects/{project}/deployments/{deployment}/logs?tail=100");
            match self.client.get_stream(&path).await {
                Ok(resp) => {
                    use futures::StreamExt;
                    let mut stream = resp.bytes_stream();
                    let mut lines = Vec::new();
                    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), async {
                        while let Some(chunk) = stream.next().await {
                            if let Ok(bytes) = chunk {
                                let text = String::from_utf8_lossy(&bytes);
                                for line in text.lines() {
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        lines.push(data.to_string());
                                    }
                                }
                            }
                        }
                    })
                    .await;
                    self.input_mode = InputMode::LogView(name, lines);
                }
                Err(e) => {
                    self.status_message = Some(format!("✗ Failed to open logs: {e}"));
                }
            }
        }
    }

    pub fn push_event(&mut self, event: ClusterEvent) {
        self.events.push(event);
        if self.events.len() > 100 {
            self.events.remove(0);
        }
    }
}
