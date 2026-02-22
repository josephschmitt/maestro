use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::config::global::{default_config_path, GlobalConfig};
use crate::config::resolution::resolve_agent_config;
use crate::executor::{EventBus, MaestroEvent};

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

pub fn get_global_config_inner(config: &ConfigState) -> Result<GlobalConfigResponse, String> {
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
pub fn get_global_config(config: State<ConfigState>) -> Result<GlobalConfigResponse, String> {
    get_global_config_inner(&config)
}

pub fn set_last_project_inner(config: &ConfigState, project_id: &str) -> Result<(), String> {
    config.update(|c| {
        c.defaults.last_project_id = project_id.to_string();
    })?;
    Ok(())
}

#[tauri::command]
pub fn set_last_project(
    config: State<ConfigState>,
    event_bus: State<Arc<EventBus>>,
    project_id: String,
) -> Result<(), String> {
    set_last_project_inner(&config, &project_id)?;
    event_bus.emit_maestro(MaestroEvent::ConfigChanged);
    Ok(())
}

pub fn resolve_config_inner(
    config: &ConfigState,
    project_agent_config: &serde_json::Value,
    status_group: &str,
) -> Result<ResolvedAgentConfigResponse, String> {
    config.with_config(|c| {
        let resolved = resolve_agent_config(c, project_agent_config, status_group);
        Ok(ResolvedAgentConfigResponse {
            agent: resolved.agent,
            model: resolved.model,
            instructions: resolved.instructions,
        })
    })
}

#[tauri::command]
pub fn resolve_config(
    config: State<ConfigState>,
    project_agent_config: serde_json::Value,
    status_group: String,
) -> Result<ResolvedAgentConfigResponse, String> {
    resolve_config_inner(&config, &project_agent_config, &status_group)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpServerConfigResponse {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub auth_token: String,
    pub requires_auth: bool,
    pub server_url: String,
}

#[derive(Debug, Deserialize)]
pub struct HttpServerConfigUpdate {
    pub enabled: Option<bool>,
    pub port: Option<u16>,
    pub bind_address: Option<String>,
}

pub fn get_http_server_config_inner(config: &ConfigState) -> Result<HttpServerConfigResponse, String> {
    config.with_config(|c| {
        let http = &c.http_server;
        let local_ip = get_local_ip_address();
        let display_address = if http.bind_address == "0.0.0.0" {
            local_ip.as_deref().unwrap_or("localhost")
        } else {
            &http.bind_address
        };
        let server_url = format!("http://{}:{}", display_address, http.port);

        Ok(HttpServerConfigResponse {
            enabled: true,
            port: http.port,
            bind_address: http.bind_address.clone(),
            auth_token: http.auth_token.clone(),
            requires_auth: http.requires_auth(),
            server_url,
        })
    })
}

#[tauri::command]
pub fn get_http_server_config(config: State<ConfigState>) -> Result<HttpServerConfigResponse, String> {
    get_http_server_config_inner(&config)
}

pub fn update_http_server_config_inner(
    config: &ConfigState,
    update: HttpServerConfigUpdate,
) -> Result<HttpServerConfigResponse, String> {
    config.update(|c| {
        if let Some(port) = update.port {
            c.http_server.port = port;
        }
        if let Some(bind_address) = update.bind_address.clone() {
            c.http_server.bind_address = bind_address;
        }
        c.http_server.ensure_auth_token();
    })?;
    get_http_server_config_inner(config)
}

#[tauri::command]
pub fn update_http_server_config(
    config: State<ConfigState>,
    event_bus: State<Arc<EventBus>>,
    update: HttpServerConfigUpdate,
) -> Result<HttpServerConfigResponse, String> {
    let result = update_http_server_config_inner(&config, update)?;
    event_bus.emit_maestro(MaestroEvent::ConfigChanged);
    Ok(result)
}

pub fn regenerate_auth_token_inner(config: &ConfigState) -> Result<String, String> {
    let new_token = config.update(|c| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        c.http_server.auth_token = hex::encode(bytes);
    })?;
    Ok(new_token.http_server.auth_token)
}

#[tauri::command]
pub fn regenerate_auth_token(
    config: State<ConfigState>,
    event_bus: State<Arc<EventBus>>,
) -> Result<String, String> {
    let token = regenerate_auth_token_inner(&config)?;
    event_bus.emit_maestro(MaestroEvent::ConfigChanged);
    Ok(token)
}

fn get_local_ip_address() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

pub fn get_local_ip_inner() -> Result<String, String> {
    get_local_ip_address().ok_or_else(|| "Could not determine local IP address".to_string())
}

#[tauri::command]
pub fn get_local_ip() -> Result<String, String> {
    get_local_ip_inner()
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
