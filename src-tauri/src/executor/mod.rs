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
use stream::AgentOutputEvent;

#[derive(Clone)]
pub enum AgentEvent {
    Output(AgentOutputEvent),
    Exit(AgentExitEvent),
}

pub struct EventBus {
    tx: broadcast::Sender<AgentEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }

    pub fn emit(&self, event: AgentEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.tx.subscribe()
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
