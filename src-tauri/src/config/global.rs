use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpServerConfig {
    #[serde(default = "default_http_enabled")]
    pub enabled: bool,
    #[serde(default = "default_http_port")]
    pub port: u16,
    #[serde(default = "default_http_bind_address")]
    pub bind_address: String,
    #[serde(default)]
    pub auth_token: Option<String>,
}

fn default_http_enabled() -> bool {
    true
}

fn default_http_port() -> u16 {
    3100
}

fn default_http_bind_address() -> String {
    "127.0.0.1".to_string()
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            enabled: default_http_enabled(),
            port: default_http_port(),
            bind_address: default_http_bind_address(),
            auth_token: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    #[serde(default = "default_storage")]
    pub storage: StorageConfig,
    #[serde(default)]
    pub agents: HashMap<String, AgentProfile>,
    #[serde(default = "default_defaults")]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub http_server: HttpServerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    #[serde(default = "default_base_path")]
    pub base_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentProfile {
    pub binary: String,
    #[serde(default)]
    pub flags: Vec<String>,
    pub custom_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultsConfig {
    #[serde(default = "default_agent")]
    pub agent: String,
    #[serde(default)]
    pub last_project_id: String,
    #[serde(default)]
    pub status: HashMap<String, StatusGroupConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusGroupConfig {
    pub agent: Option<String>,
    pub model: Option<String>,
    pub instructions: Option<String>,
}

fn default_base_path() -> String {
    "~/.maestro".to_string()
}

fn default_agent() -> String {
    "claude-code".to_string()
}

fn default_storage() -> StorageConfig {
    StorageConfig {
        base_path: default_base_path(),
    }
}

fn default_defaults() -> DefaultsConfig {
    DefaultsConfig {
        agent: default_agent(),
        last_project_id: String::new(),
        status: HashMap::new(),
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        let mut agents = HashMap::new();
        agents.insert(
            "claude-code".to_string(),
            AgentProfile {
                binary: "claude".to_string(),
                flags: vec!["--dangerously-skip-permissions".to_string()],
                custom_command: None,
            },
        );
        agents.insert(
            "codex".to_string(),
            AgentProfile {
                binary: "codex".to_string(),
                flags: vec!["--full-auto".to_string()],
                custom_command: None,
            },
        );

        Self {
            storage: default_storage(),
            agents,
            defaults: default_defaults(),
            http_server: HttpServerConfig::default(),
        }
    }
}

#[allow(dead_code)]
impl GlobalConfig {
    pub fn resolve_base_path(&self) -> PathBuf {
        expand_tilde(&self.storage.base_path)
    }

    pub fn projects_dir(&self) -> PathBuf {
        self.resolve_base_path().join("projects")
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.resolve_base_path().join("config.toml")
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            let config = Self::default();
            config.save(path)?;
            return Ok(config);
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {e}"))?;

        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {e}"))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {e}"))?;

        std::fs::write(path, content).map_err(|e| format!("Failed to write config: {e}"))
    }
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

pub fn default_config_path() -> PathBuf {
    expand_tilde("~/.maestro/config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_serializes() {
        let config = GlobalConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("base_path"));
        assert!(toml_str.contains("claude-code"));
    }

    #[test]
    fn test_load_creates_default_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = GlobalConfig::load(&path).unwrap();
        assert_eq!(config.defaults.agent, "claude-code");
        assert!(path.exists());
    }

    #[test]
    fn test_load_parses_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let content = r#"
[storage]
base_path = "/custom/path"

[defaults]
agent = "codex"
last_project_id = "abc-123"
"#;
        std::fs::write(&path, content).unwrap();
        let config = GlobalConfig::load(&path).unwrap();
        assert_eq!(config.storage.base_path, "/custom/path");
        assert_eq!(config.defaults.agent, "codex");
        assert_eq!(config.defaults.last_project_id, "abc-123");
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/.maestro");
        assert!(!expanded.to_str().unwrap().starts_with('~'));

        let absolute = expand_tilde("/absolute/path");
        assert_eq!(absolute, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = GlobalConfig::default();
        config.defaults.last_project_id = "test-id".to_string();
        config.save(&path).unwrap();

        let loaded = GlobalConfig::load(&path).unwrap();
        assert_eq!(loaded.defaults.last_project_id, "test-id");
    }
}
