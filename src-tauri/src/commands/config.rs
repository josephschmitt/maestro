use serde::{Deserialize, Serialize};
use tauri::State;

use crate::config::global::{default_config_path, GlobalConfig};
use crate::config::resolution::resolve_agent_config;

use std::path::PathBuf;
use std::sync::Mutex;

pub struct ConfigState {
    pub(crate) config: Mutex<GlobalConfig>,
    pub(crate) config_path: PathBuf,
}

impl ConfigState {
    pub fn load() -> Result<Self, String> {
        let config_path = default_config_path();
        let config = GlobalConfig::load(&config_path)?;
        Ok(Self {
            config: Mutex::new(config),
            config_path,
        })
    }

    pub fn with_config<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&GlobalConfig) -> Result<T, String>,
    {
        let config = self
            .config
            .lock()
            .map_err(|e| format!("Config lock poisoned: {e}"))?;
        f(&config)
    }

    pub fn update<F>(&self, f: F) -> Result<GlobalConfig, String>
    where
        F: FnOnce(&mut GlobalConfig),
    {
        let mut config = self
            .config
            .lock()
            .map_err(|e| format!("Config lock poisoned: {e}"))?;
        f(&mut config);
        config.save(&self.config_path)?;
        Ok(config.clone())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfigResponse {
    pub storage_base_path: String,
    pub default_agent: String,
    pub last_project_id: String,
    pub agents: Vec<AgentProfileResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentProfileResponse {
    pub name: String,
    pub binary: String,
    pub flags: Vec<String>,
    pub custom_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedAgentConfigResponse {
    pub agent: String,
    pub model: Option<String>,
    pub instructions: Option<String>,
}

#[tauri::command]
pub fn get_global_config(config: State<ConfigState>) -> Result<GlobalConfigResponse, String> {
    config.with_config(|c| {
        let agents = c
            .agents
            .iter()
            .map(|(name, profile)| AgentProfileResponse {
                name: name.clone(),
                binary: profile.binary.clone(),
                flags: profile.flags.clone(),
                custom_command: profile.custom_command.clone(),
            })
            .collect();

        Ok(GlobalConfigResponse {
            storage_base_path: c.storage.base_path.clone(),
            default_agent: c.defaults.agent.clone(),
            last_project_id: c.defaults.last_project_id.clone(),
            agents,
        })
    })
}

#[tauri::command]
pub fn set_last_project(config: State<ConfigState>, project_id: String) -> Result<(), String> {
    config.update(|c| {
        c.defaults.last_project_id = project_id;
    })?;
    Ok(())
}

#[tauri::command]
pub fn resolve_config(
    config: State<ConfigState>,
    project_agent_config: serde_json::Value,
    status_group: String,
) -> Result<ResolvedAgentConfigResponse, String> {
    config.with_config(|c| {
        let resolved = resolve_agent_config(c, &project_agent_config, &status_group);
        Ok(ResolvedAgentConfigResponse {
            agent: resolved.agent,
            model: resolved.model,
            instructions: resolved.instructions,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_state_load_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let config = GlobalConfig::load(&config_path).unwrap();
        let state = ConfigState {
            config: Mutex::new(config),
            config_path,
        };

        let result = state
            .with_config(|c| Ok(c.defaults.agent.clone()))
            .unwrap();
        assert_eq!(result, "claude-code");
    }

    #[test]
    fn test_config_state_update() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let config = GlobalConfig::load(&config_path).unwrap();
        let state = ConfigState {
            config: Mutex::new(config),
            config_path: config_path.clone(),
        };

        state
            .update(|c| {
                c.defaults.last_project_id = "test-123".to_string();
            })
            .unwrap();

        let reloaded = GlobalConfig::load(&config_path).unwrap();
        assert_eq!(reloaded.defaults.last_project_id, "test-123");
    }
}
