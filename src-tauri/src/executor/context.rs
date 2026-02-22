use crate::config::global::GlobalConfig;
use crate::config::resolution::{resolve_agent_config, ResolvedAgentConfig};

#[derive(Debug)]
pub struct AgentContext {
    pub binary: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub env: Vec<(String, String)>,
    pub system_prompt: String,
}

pub struct CardInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub parent_title: Option<String>,
    pub parent_description: Option<String>,
}

pub fn assemble_context(
    global_config: &GlobalConfig,
    project_agent_config: &serde_json::Value,
    status_group: &str,
    card: &CardInfo,
    working_dir: &str,
    artifact_contents: &[(String, String)],
    socket_path: Option<&str>,
    worktree_name: Option<&str>,
    status_prompts: &[String],
) -> Result<AgentContext, String> {
    let resolved = resolve_agent_config(global_config, project_agent_config, status_group);

    let (binary, base_flags) = resolve_binary_and_flags(global_config, &resolved)?;

    let system_prompt = build_system_prompt(&resolved, card, artifact_contents, socket_path.is_some(), status_prompts);

    let mut args = base_flags;
    args.push("--print".to_string());
    args.push(system_prompt.clone());

    if let Some(ref model) = resolved.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }

    if let Some(name) = worktree_name {
        args.push("--worktree".to_string());
        args.push(name.to_string());
    }

    let mut env = vec![("MAESTRO_CARD_ID".to_string(), card.id.clone())];
    env.push(("MAESTRO_WORKING_DIR".to_string(), working_dir.to_string()));

    if let Some(socket) = socket_path {
        env.push(("MAESTRO_SOCKET".to_string(), socket.to_string()));
    }

    Ok(AgentContext {
        binary,
        args,
        working_dir: working_dir.to_string(),
        env,
        system_prompt,
    })
}

fn resolve_binary_and_flags(
    global_config: &GlobalConfig,
    resolved: &ResolvedAgentConfig,
) -> Result<(String, Vec<String>), String> {
    let profile = global_config
        .agents
        .get(&resolved.agent)
        .ok_or_else(|| format!("Agent profile '{}' not found in config", resolved.agent))?;

    if let Some(ref custom_command) = profile.custom_command {
        Ok((custom_command.clone(), vec![]))
    } else {
        Ok((profile.binary.clone(), profile.flags.clone()))
    }
}

const MAESTRO_SKILL: &str = include_str!("../../../assets/maestro-skill.md");

const PROMPT_BRAINSTORMING: &str = include_str!("../../../assets/status-prompts/brainstorming.md");
const PROMPT_TDD: &str = include_str!("../../../assets/status-prompts/tdd.md");
const PROMPT_DEBUGGING: &str = include_str!("../../../assets/status-prompts/systematic-debugging.md");
const PROMPT_VERIFICATION: &str = include_str!("../../../assets/status-prompts/verification.md");
const PROMPT_CODE_REVIEW: &str = include_str!("../../../assets/status-prompts/code-review.md");
const PROMPT_IMPLEMENTATION_PLANNING: &str = include_str!("../../../assets/status-prompts/implementation-planning.md");

fn get_status_prompt_content(prompt_id: &str) -> Option<&'static str> {
    match prompt_id {
        "brainstorming" => Some(PROMPT_BRAINSTORMING),
        "tdd" => Some(PROMPT_TDD),
        "systematic-debugging" => Some(PROMPT_DEBUGGING),
        "verification" => Some(PROMPT_VERIFICATION),
        "code-review" => Some(PROMPT_CODE_REVIEW),
        "implementation-planning" => Some(PROMPT_IMPLEMENTATION_PLANNING),
        _ => None,
    }
}

fn build_system_prompt(
    resolved: &ResolvedAgentConfig,
    card: &CardInfo,
    artifact_contents: &[(String, String)],
    include_skill: bool,
    status_prompts: &[String],
) -> String {
    let mut parts = Vec::new();

    if let Some(ref instructions) = resolved.instructions {
        parts.push(instructions.clone());
    }

    if include_skill {
        parts.push(MAESTRO_SKILL.to_string());
    }

    for prompt_id in status_prompts {
        if let Some(content) = get_status_prompt_content(prompt_id) {
            parts.push(content.to_string());
        }
    }

    parts.push(format!("# Task: {}", card.title));

    if !card.description.is_empty() {
        parts.push(format!("\n## Description\n\n{}", card.description));
    }

    if let Some(ref parent_title) = card.parent_title {
        parts.push(format!("\n## Parent Card: {}", parent_title));
        if let Some(ref parent_desc) = card.parent_description {
            if !parent_desc.is_empty() {
                parts.push(parent_desc.clone());
            }
        }
    }

    if !artifact_contents.is_empty() {
        parts.push("\n## Exploration Artifacts\n".to_string());
        for (name, content) in artifact_contents {
            parts.push(format!("### {name}\n\n{content}"));
        }
    }

    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::global::{AgentProfile, DefaultsConfig, StatusGroupConfig};
    use std::collections::HashMap;

    fn test_config() -> GlobalConfig {
        let mut agents = HashMap::new();
        agents.insert(
            "claude-code".to_string(),
            AgentProfile {
                binary: "claude".to_string(),
                flags: vec!["--dangerously-skip-permissions".to_string()],
                custom_command: None,
                env_vars: None,
            },
        );

        let mut status = HashMap::new();
        status.insert(
            "backlog".to_string(),
            StatusGroupConfig {
                agent: Some("claude-code".to_string()),
                model: Some("sonnet".to_string()),
                instructions: Some("You are in exploration mode.".to_string()),
            },
        );

        GlobalConfig {
            agents,
            defaults: DefaultsConfig {
                agent: "claude-code".to_string(),
                last_project_id: String::new(),
                status,
            },
            ..GlobalConfig::default()
        }
    }

    #[test]
    fn test_assemble_context_basic() {
        let config = test_config();
        let card = CardInfo {
            id: "card-123".to_string(),
            title: "Build feature X".to_string(),
            description: "Implement the new feature".to_string(),
            parent_title: None,
            parent_description: None,
        };

        let ctx =
            assemble_context(&config, &serde_json::json!({}), "Backlog", &card, "/tmp/work", &[], None, None, &[])
                .unwrap();

        assert_eq!(ctx.binary, "claude");
        assert!(ctx.args.contains(&"--dangerously-skip-permissions".to_string()));
        assert!(ctx.args.contains(&"--model".to_string()));
        assert!(ctx.args.contains(&"sonnet".to_string()));
        assert!(!ctx.args.contains(&"--worktree".to_string()));
        assert_eq!(ctx.working_dir, "/tmp/work");
        assert!(ctx.system_prompt.contains("Build feature X"));
        assert!(ctx.system_prompt.contains("exploration mode"));

        let has_card_env = ctx.env.iter().any(|(k, v)| k == "MAESTRO_CARD_ID" && v == "card-123");
        assert!(has_card_env);
    }

    #[test]
    fn test_assemble_context_with_parent() {
        let config = test_config();
        let card = CardInfo {
            id: "card-456".to_string(),
            title: "Sub-task A".to_string(),
            description: "".to_string(),
            parent_title: Some("Parent Feature".to_string()),
            parent_description: Some("The parent description".to_string()),
        };

        let ctx =
            assemble_context(&config, &serde_json::json!({}), "Backlog", &card, "/tmp/work", &[], None, None, &[])
                .unwrap();

        assert!(ctx.system_prompt.contains("Parent Card: Parent Feature"));
        assert!(ctx.system_prompt.contains("The parent description"));
    }

    #[test]
    fn test_assemble_context_missing_agent_profile() {
        let config = test_config();
        let card = CardInfo {
            id: "card-789".to_string(),
            title: "Test".to_string(),
            description: "".to_string(),
            parent_title: None,
            parent_description: None,
        };

        let project_config = serde_json::json!({ "agent": "nonexistent" });
        let result = assemble_context(&config, &project_config, "Backlog", &card, "/tmp/work", &[], None, None, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_assemble_context_with_artifacts() {
        let config = test_config();
        let card = CardInfo {
            id: "card-123".to_string(),
            title: "Build feature X".to_string(),
            description: "Implement the new feature".to_string(),
            parent_title: None,
            parent_description: None,
        };

        let artifacts = vec![
            ("plan.md".to_string(), "# Implementation Plan\n\nStep 1: Do stuff".to_string()),
            ("notes.md".to_string(), "Research notes here".to_string()),
        ];

        let ctx = assemble_context(
            &config,
            &serde_json::json!({}),
            "Backlog",
            &card,
            "/tmp/work",
            &artifacts,
            None,
            None,
            &[],
        )
        .unwrap();

        assert!(ctx.system_prompt.contains("Exploration Artifacts"));
        assert!(ctx.system_prompt.contains("plan.md"));
        assert!(ctx.system_prompt.contains("Implementation Plan"));
        assert!(ctx.system_prompt.contains("notes.md"));
        assert!(ctx.system_prompt.contains("Research notes here"));
    }

    #[test]
    fn test_assemble_context_with_worktree_name() {
        let config = test_config();
        let card = CardInfo {
            id: "card-123".to_string(),
            title: "Build feature X".to_string(),
            description: "Implement the new feature".to_string(),
            parent_title: None,
            parent_description: None,
        };

        let ctx = assemble_context(
            &config,
            &serde_json::json!({}),
            "Backlog",
            &card,
            "/home/user/repo",
            &[],
            None,
            Some("a1b2c3d4-build-feature-x"),
            &[],
        )
        .unwrap();

        assert!(ctx.args.contains(&"--worktree".to_string()));
        assert!(ctx.args.contains(&"a1b2c3d4-build-feature-x".to_string()));
        assert_eq!(ctx.working_dir, "/home/user/repo");
    }

    #[test]
    fn test_assemble_context_with_status_prompts() {
        let config = test_config();
        let card = CardInfo {
            id: "card-123".to_string(),
            title: "Build feature X".to_string(),
            description: "Implement the new feature".to_string(),
            parent_title: None,
            parent_description: None,
        };

        let prompts = vec!["tdd".to_string(), "verification".to_string()];
        let ctx = assemble_context(
            &config,
            &serde_json::json!({}),
            "Backlog",
            &card,
            "/tmp/work",
            &[],
            None,
            None,
            &prompts,
        )
        .unwrap();

        assert!(ctx.system_prompt.contains("NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST"));
        assert!(ctx.system_prompt.contains("NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE"));
    }

    #[test]
    fn test_assemble_context_empty_status_prompts() {
        let config = test_config();
        let card = CardInfo {
            id: "card-123".to_string(),
            title: "Build feature X".to_string(),
            description: "Implement the new feature".to_string(),
            parent_title: None,
            parent_description: None,
        };

        let ctx = assemble_context(
            &config,
            &serde_json::json!({}),
            "Backlog",
            &card,
            "/tmp/work",
            &[],
            None,
            None,
            &[],
        )
        .unwrap();

        assert!(!ctx.system_prompt.contains("NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST"));
        assert!(!ctx.system_prompt.contains("NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE"));
    }

    #[test]
    fn test_assemble_context_unknown_status_prompt_skipped() {
        let config = test_config();
        let card = CardInfo {
            id: "card-123".to_string(),
            title: "Build feature X".to_string(),
            description: "Implement the new feature".to_string(),
            parent_title: None,
            parent_description: None,
        };

        let prompts = vec!["nonexistent-prompt".to_string(), "tdd".to_string()];
        let ctx = assemble_context(
            &config,
            &serde_json::json!({}),
            "Backlog",
            &card,
            "/tmp/work",
            &[],
            None,
            None,
            &prompts,
        )
        .unwrap();

        assert!(ctx.system_prompt.contains("NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST"));
        assert!(!ctx.system_prompt.contains("nonexistent-prompt"));
    }

    #[test]
    fn test_status_prompts_injected_before_card_info() {
        let config = test_config();
        let card = CardInfo {
            id: "card-123".to_string(),
            title: "Build feature X".to_string(),
            description: "Implement the new feature".to_string(),
            parent_title: None,
            parent_description: None,
        };

        let prompts = vec!["brainstorming".to_string()];
        let ctx = assemble_context(
            &config,
            &serde_json::json!({}),
            "Backlog",
            &card,
            "/tmp/work",
            &[],
            None,
            None,
            &prompts,
        )
        .unwrap();

        let brainstorming_pos = ctx.system_prompt.find("# Brainstorming").unwrap();
        let task_pos = ctx.system_prompt.find("# Task: Build feature X").unwrap();
        assert!(brainstorming_pos < task_pos, "Status prompts should appear before card info");
    }
}
