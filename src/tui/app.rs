use serde::Deserialize;

use crate::client::NexaClient;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Pods,
    Nodes,
    Events,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct ClusterEvent {
    pub timestamp: String,
    pub kind: String,
    pub name: String,
    pub action: String,
    pub message: String,
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
        let nodes_result = self.client.get::<Vec<NodeStats>>("/api/v1/nodes/stats").await;

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
}
