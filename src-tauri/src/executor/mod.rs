pub mod context;
pub mod lifecycle;
pub mod monitor;
pub mod reattach;
pub mod spawn;
pub mod stream;

use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::{broadcast, mpsc};

use lifecycle::AgentExitEvent;
use monitor::AgentCrashedEvent;
use serde::{Deserialize, Serialize};
use stream::AgentOutputEvent;

#[derive(Clone)]
pub enum AgentEvent {
    Output(AgentOutputEvent),
    Exit(AgentExitEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLogEvent {
    pub card_id: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum MaestroEvent {
    #[serde(rename = "agent-output")]
    AgentOutput(AgentOutputEvent),
    #[serde(rename = "agent-exit")]
    AgentExit(AgentExitEvent),
    #[serde(rename = "agent-crashed")]
    AgentCrashed(AgentCrashedEvent),
    #[serde(rename = "agent-log")]
    AgentLog(AgentLogEvent),

    #[serde(rename = "cards-changed")]
    CardsChanged { project_id: String },
    #[serde(rename = "statuses-changed")]
    StatusesChanged { project_id: String },
    #[serde(rename = "questions-changed")]
    QuestionsChanged { project_id: String },
    #[serde(rename = "artifacts-changed")]
    ArtifactsChanged { project_id: String },
    #[serde(rename = "conversations-changed")]
    ConversationsChanged { project_id: String },
    #[serde(rename = "workspaces-changed")]
    WorkspacesChanged { project_id: String },
    #[serde(rename = "directories-changed")]
    DirectoriesChanged { project_id: String },

    #[serde(rename = "projects-changed")]
    ProjectsChanged,
    #[serde(rename = "config-changed")]
    ConfigChanged,
}

impl MaestroEvent {
    pub fn scope(&self) -> Option<&str> {
        match self {
            MaestroEvent::AgentOutput(e) => Some(&e.workspace_id),
            MaestroEvent::AgentExit(e) => Some(&e.workspace_id),
            MaestroEvent::AgentCrashed(e) => Some(&e.workspace_id),
            MaestroEvent::AgentLog(_) => None,
            MaestroEvent::CardsChanged { project_id } => Some(project_id),
            MaestroEvent::StatusesChanged { project_id } => Some(project_id),
            MaestroEvent::QuestionsChanged { project_id } => Some(project_id),
            MaestroEvent::ArtifactsChanged { project_id } => Some(project_id),
            MaestroEvent::ConversationsChanged { project_id } => Some(project_id),
            MaestroEvent::WorkspacesChanged { project_id } => Some(project_id),
            MaestroEvent::DirectoriesChanged { project_id } => Some(project_id),
            MaestroEvent::ProjectsChanged => None,
            MaestroEvent::ConfigChanged => None,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            MaestroEvent::AgentOutput(_) => "agent-output",
            MaestroEvent::AgentExit(_) => "agent-exit",
            MaestroEvent::AgentCrashed(_) => "agent-crashed",
            MaestroEvent::AgentLog(_) => "agent-log",
            MaestroEvent::CardsChanged { .. } => "cards-changed",
            MaestroEvent::StatusesChanged { .. } => "statuses-changed",
            MaestroEvent::QuestionsChanged { .. } => "questions-changed",
            MaestroEvent::ArtifactsChanged { .. } => "artifacts-changed",
            MaestroEvent::ConversationsChanged { .. } => "conversations-changed",
            MaestroEvent::WorkspacesChanged { .. } => "workspaces-changed",
            MaestroEvent::DirectoriesChanged { .. } => "directories-changed",
            MaestroEvent::ProjectsChanged => "projects-changed",
            MaestroEvent::ConfigChanged => "config-changed",
        }
    }
}

pub struct EventBus {
    agent_tx: broadcast::Sender<AgentEvent>,
    maestro_tx: broadcast::Sender<MaestroEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (agent_tx, _) = broadcast::channel(1024);
        let (maestro_tx, _) = broadcast::channel(1024);
        Self { agent_tx, maestro_tx }
    }

    pub fn emit(&self, event: AgentEvent) {
        let _ = self.agent_tx.send(event);
    }

    pub fn emit_maestro(&self, event: MaestroEvent) {
        let _ = self.maestro_tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.agent_tx.subscribe()
    }

    pub fn subscribe_maestro(&self) -> broadcast::Receiver<MaestroEvent> {
        self.maestro_tx.subscribe()
    }
}

pub struct AgentHandle {
    pub workspace_id: String,
    pub stdin_tx: mpsc::Sender<String>,
    pub pid: u32,
}

pub struct AgentRegistry {
    handles: Mutex<HashMap<String, AgentHandle>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            handles: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, handle: AgentHandle) {
        let mut handles = self.handles.lock().unwrap();
        handles.insert(handle.workspace_id.clone(), handle);
    }

    pub fn remove(&self, workspace_id: &str) -> Option<AgentHandle> {
        let mut handles = self.handles.lock().unwrap();
        handles.remove(workspace_id)
    }

    pub fn get_stdin_tx(&self, workspace_id: &str) -> Option<mpsc::Sender<String>> {
        let handles = self.handles.lock().unwrap();
        handles.get(workspace_id).map(|h| h.stdin_tx.clone())
    }

    pub fn has(&self, workspace_id: &str) -> bool {
        let handles = self.handles.lock().unwrap();
        handles.contains_key(workspace_id)
    }

    pub fn running_count(&self) -> usize {
        let handles = self.handles.lock().unwrap();
        handles.len()
    }

    pub fn all_pids(&self) -> Vec<u32> {
        let handles = self.handles.lock().unwrap();
        handles.values().map(|h| h.pid).collect()
    }
}
