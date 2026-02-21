pub mod context;
pub mod lifecycle;
pub mod monitor;
pub mod reattach;
pub mod spawn;
pub mod stream;

use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;

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
