use super::global::GlobalConfig;

pub struct ResolvedAgentConfig {
    pub agent: String,
    pub model: Option<String>,
    pub instructions: Option<String>,
}

pub fn resolve_agent_config(
    global: &GlobalConfig,
    project_config: &serde_json::Value,
    status_group: &str,
) -> ResolvedAgentConfig {
    let group_key = status_group.to_lowercase();

    // 1. Project-level status override
    if let Some(status_config) = project_config
        .get("status")
        .and_then(|s| s.get(&group_key))
    {
        let agent = status_config
            .get("agent")
            .and_then(|v| v.as_str())
            .map(String::from);
        let model = status_config
            .get("model")
            .and_then(|v| v.as_str())
            .map(String::from);
        let instructions = status_config
            .get("instructions")
            .and_then(|v| v.as_str())
            .map(String::from);

        if agent.is_some() || model.is_some() || instructions.is_some() {
            return ResolvedAgentConfig {
                agent: agent.unwrap_or_else(|| resolve_default_agent(global, project_config)),
                model,
                instructions,
            };
        }
    }

    // 2. Project-level default
    if let Some(agent) = project_config
        .get("agent")
        .and_then(|v| v.as_str())
        .map(String::from)
    {
        return ResolvedAgentConfig {
            agent,
            model: None,
            instructions: None,
        };
    }

    // 3. Global status override
    if let Some(global_status) = global.defaults.status.get(&group_key) {
        return ResolvedAgentConfig {
            agent: global_status
                .agent
                .clone()
                .unwrap_or_else(|| global.defaults.agent.clone()),
            model: global_status.model.clone(),
            instructions: global_status.instructions.clone(),
        };
    }

    // 4. Global default
    ResolvedAgentConfig {
        agent: global.defaults.agent.clone(),
        model: None,
        instructions: None,
    }
}

fn resolve_default_agent(global: &GlobalConfig, project_config: &serde_json::Value) -> String {
    project_config
        .get("agent")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| global.defaults.agent.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::global::StatusGroupConfig;
    use std::collections::HashMap;

    fn test_global() -> GlobalConfig {
        let mut status = HashMap::new();
        status.insert(
            "started".to_string(),
            StatusGroupConfig {
                agent: Some("claude-code".to_string()),
                model: Some("sonnet".to_string()),
                instructions: Some("Global started instructions".to_string()),
            },
        );

        GlobalConfig {
            defaults: super::super::global::DefaultsConfig {
                agent: "claude-code".to_string(),
                last_project_id: String::new(),
                status,
            },
            ..GlobalConfig::default()
        }
    }

    #[test]
    fn test_global_default_fallback() {
        let global = test_global();
        let project = serde_json::json!({});
        let resolved = resolve_agent_config(&global, &project, "Backlog");
        assert_eq!(resolved.agent, "claude-code");
        assert!(resolved.model.is_none());
    }

    #[test]
    fn test_global_status_override() {
        let global = test_global();
        let project = serde_json::json!({});
        let resolved = resolve_agent_config(&global, &project, "Started");
        assert_eq!(resolved.agent, "claude-code");
        assert_eq!(resolved.model.as_deref(), Some("sonnet"));
        assert!(resolved.instructions.is_some());
    }

    #[test]
    fn test_project_default_overrides_global() {
        let global = test_global();
        let project = serde_json::json!({ "agent": "codex" });
        let resolved = resolve_agent_config(&global, &project, "Backlog");
        assert_eq!(resolved.agent, "codex");
    }

    #[test]
    fn test_project_status_overrides_all() {
        let global = test_global();
        let project = serde_json::json!({
            "agent": "codex",
            "status": {
                "started": {
                    "agent": "opencode",
                    "model": "opus",
                    "instructions": "Project started instructions"
                }
            }
        });
        let resolved = resolve_agent_config(&global, &project, "Started");
        assert_eq!(resolved.agent, "opencode");
        assert_eq!(resolved.model.as_deref(), Some("opus"));
        assert_eq!(
            resolved.instructions.as_deref(),
            Some("Project started instructions")
        );
    }

    #[test]
    fn test_partial_project_status_inherits_agent() {
        let global = test_global();
        let project = serde_json::json!({
            "agent": "codex",
            "status": {
                "started": {
                    "model": "opus"
                }
            }
        });
        let resolved = resolve_agent_config(&global, &project, "Started");
        assert_eq!(resolved.agent, "codex");
        assert_eq!(resolved.model.as_deref(), Some("opus"));
    }
}
