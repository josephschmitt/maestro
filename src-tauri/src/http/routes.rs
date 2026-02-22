use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde_json::Value;

use super::error::{extract_arg, extract_optional_arg, AppError};
use super::server::AppState;
use crate::executor::MaestroEvent;

use crate::commands::{
    agent::{
        get_workspace_inner, launch_agent_inner, list_running_workspaces_inner,
        list_workspaces_inner, resume_agent_inner, send_agent_input_inner, stop_agent_inner,
        stop_all_agents_inner,
    },
    artifacts::{
        create_artifact_inner, delete_artifact_inner, list_artifacts_inner, read_artifact_inner,
        update_artifact_inner,
    },
    cards::{
        create_card_inner, delete_card_inner, get_card_inner, list_cards_inner,
        list_sub_cards_inner, move_card_inner, reorder_cards_inner, update_card_inner,
    },
    config::{get_global_config_inner, resolve_config_inner, set_last_project_inner},
    conversations::{
        count_conversation_messages_inner, create_conversation_inner, create_message_inner,
        list_conversations_inner, list_messages_inner,
    },
    directories::{
        add_linked_directory_inner, list_linked_directories_inner, remove_linked_directory_inner,
    },
    projects::{
        create_project_inner, delete_project_inner, get_project_inner, list_projects_inner,
        update_project_inner,
    },
    questions::{
        count_unresolved_questions_inner, create_question_inner, delete_question_inner,
        list_questions_inner, resolve_question_inner, unresolve_question_inner,
    },
    statuses::{
        create_status_inner, delete_status_inner, list_statuses_inner, reorder_statuses_inner,
        update_status_inner,
    },
    worktrees::{
        check_worktree_exists_inner, create_worktree_inner, generate_branch_name_inner,
        get_card_worktree_inner, get_claude_worktree_path_inner,
    },
};

pub fn api_routes() -> Router<AppState> {
    Router::new().route("/{command}", post(handle_command))
}

async fn handle_command(
    State(state): State<AppState>,
    Path(command): Path<String>,
    Json(args): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let result = dispatch_command(&state, &command, &args).await?;
    Ok(Json(result))
}

async fn dispatch_command(
    state: &AppState,
    command: &str,
    args: &Value,
) -> Result<Value, AppError> {
    match command {
        // Config commands
        "get_global_config" => dispatch_config_get_global(state, args),
        "set_last_project" => dispatch_config_set_last_project(state, args),
        "resolve_config" => dispatch_config_resolve(state, args),

        // Project commands
        "create_project" => dispatch_projects_create(state, args),
        "get_project" => dispatch_projects_get(state, args),
        "list_projects" => dispatch_projects_list(state, args),
        "update_project" => dispatch_projects_update(state, args),
        "delete_project" => dispatch_projects_delete(state, args),

        // Status commands
        "list_statuses" => dispatch_statuses_list(state, args),
        "create_status" => dispatch_statuses_create(state, args),
        "update_status" => dispatch_statuses_update(state, args),
        "delete_status" => dispatch_statuses_delete(state, args),
        "reorder_statuses" => dispatch_statuses_reorder(state, args),

        // Card commands
        "create_card" => dispatch_cards_create(state, args),
        "get_card" => dispatch_cards_get(state, args),
        "update_card" => dispatch_cards_update(state, args),
        "delete_card" => dispatch_cards_delete(state, args),
        "list_cards" => dispatch_cards_list(state, args),
        "list_sub_cards" => dispatch_cards_list_sub(state, args),
        "move_card" => dispatch_cards_move(state, args),
        "reorder_cards" => dispatch_cards_reorder(state, args),

        // Question commands
        "create_question" => dispatch_questions_create(state, args),
        "list_questions" => dispatch_questions_list(state, args),
        "resolve_question" => dispatch_questions_resolve(state, args),
        "unresolve_question" => dispatch_questions_unresolve(state, args),
        "delete_question" => dispatch_questions_delete(state, args),
        "count_unresolved_questions" => dispatch_questions_count_unresolved(state, args),

        // Artifact commands
        "create_artifact" => dispatch_artifacts_create(state, args),
        "read_artifact" => dispatch_artifacts_read(state, args),
        "update_artifact" => dispatch_artifacts_update(state, args),
        "delete_artifact" => dispatch_artifacts_delete(state, args),
        "list_artifacts" => dispatch_artifacts_list(state, args),

        // Directory commands
        "add_linked_directory" => dispatch_directories_add(state, args),
        "remove_linked_directory" => dispatch_directories_remove(state, args),
        "list_linked_directories" => dispatch_directories_list(state, args),

        // Conversation commands
        "create_conversation" => dispatch_conversations_create(state, args),
        "list_conversations" => dispatch_conversations_list(state, args),
        "create_message" => dispatch_conversations_create_message(state, args),
        "list_messages" => dispatch_conversations_list_messages(state, args),
        "count_conversation_messages" => dispatch_conversations_count_messages(state, args),

        // Worktree commands
        "generate_branch_name" => dispatch_worktrees_generate_branch_name(state, args),
        "create_worktree" => dispatch_worktrees_create(state, args),
        "check_worktree_exists" => dispatch_worktrees_check_exists(state, args),
        "get_card_worktree" => dispatch_worktrees_get_card(state, args),
        "get_claude_worktree_path" => dispatch_worktrees_get_claude_path(state, args),

        // Agent commands (async)
        "launch_agent" => dispatch_agent_launch(state, args).await,
        "send_agent_input" => dispatch_agent_send_input(state, args).await,
        "stop_agent" => dispatch_agent_stop(state, args).await,
        "resume_agent" => dispatch_agent_resume(state, args).await,
        "stop_all_agents" => dispatch_agent_stop_all(state, args).await,
        "list_workspaces" => dispatch_agent_list_workspaces(state, args),
        "get_workspace" => dispatch_agent_get_workspace(state, args),
        "list_running_workspaces" => dispatch_agent_list_running(state, args),

        _ => Err(AppError::NotFound(format!("Unknown command: {command}"))),
    }
}

// ============================================================================
// Config dispatchers
// ============================================================================

fn dispatch_config_get_global(state: &AppState, _args: &Value) -> Result<Value, AppError> {
    let result = get_global_config_inner(&state.config)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_config_set_last_project(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    set_last_project_inner(&state.config, &project_id)?;
    state.event_bus.emit_maestro(MaestroEvent::ConfigChanged);
    Ok(serde_json::json!(null))
}

fn dispatch_config_resolve(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_agent_config: Value = extract_arg(args, "project_agent_config")?;
    let status_group: String = extract_arg(args, "status_group")?;
    let result = resolve_config_inner(&state.config, &project_agent_config, &status_group)?;
    Ok(serde_json::to_value(result).unwrap())
}

// ============================================================================
// Project dispatchers
// ============================================================================

fn dispatch_projects_create(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let name: String = extract_arg(args, "name")?;
    let result = create_project_inner(&state.config, &name)?;
    state.event_bus.emit_maestro(MaestroEvent::ProjectsChanged);
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_projects_get(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let id: String = extract_arg(args, "id")?;
    let result = get_project_inner(&state.config, &id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_projects_list(state: &AppState, _args: &Value) -> Result<Value, AppError> {
    let result = list_projects_inner(&state.config)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_projects_update(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let id: String = extract_arg(args, "id")?;
    let name: Option<String> = extract_optional_arg(args, "name")?;
    let agent_config: Option<Value> = extract_optional_arg(args, "agent_config")?;
    let base_path: Option<String> = extract_optional_arg(args, "base_path")?;
    let result = update_project_inner(&state.config, &id, name, agent_config, base_path)?;
    state.event_bus.emit_maestro(MaestroEvent::ProjectsChanged);
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_projects_delete(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let id: String = extract_arg(args, "id")?;
    delete_project_inner(&state.config, &id)?;
    state.event_bus.emit_maestro(MaestroEvent::ProjectsChanged);
    Ok(serde_json::json!(null))
}

// ============================================================================
// Status dispatchers
// ============================================================================

fn dispatch_statuses_list(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let result = list_statuses_inner(&state.config, &project_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_statuses_create(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let group: String = extract_arg(args, "group")?;
    let name: String = extract_arg(args, "name")?;
    let is_default: Option<bool> = extract_optional_arg(args, "is_default")?;
    let status_prompts: Option<Vec<String>> = extract_optional_arg(args, "status_prompts")?;
    let result =
        create_status_inner(&state.config, &project_id, &group, &name, is_default, status_prompts)?;
    state.event_bus.emit_maestro(MaestroEvent::StatusesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_statuses_update(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    let name: Option<String> = extract_optional_arg(args, "name")?;
    let is_default: Option<bool> = extract_optional_arg(args, "is_default")?;
    let status_prompts: Option<Vec<String>> = extract_optional_arg(args, "status_prompts")?;
    let result =
        update_status_inner(&state.config, &project_id, &id, name, is_default, status_prompts)?;
    state.event_bus.emit_maestro(MaestroEvent::StatusesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_statuses_delete(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    delete_status_inner(&state.config, &project_id, &id)?;
    state.event_bus.emit_maestro(MaestroEvent::StatusesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::json!(null))
}

fn dispatch_statuses_reorder(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let group: String = extract_arg(args, "group")?;
    let status_ids: Vec<String> = extract_arg(args, "status_ids")?;
    let result = reorder_statuses_inner(&state.config, &project_id, &group, &status_ids)?;
    state.event_bus.emit_maestro(MaestroEvent::StatusesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

// ============================================================================
// Card dispatchers
// ============================================================================

fn dispatch_cards_create(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let title: String = extract_arg(args, "title")?;
    let description: Option<String> = extract_optional_arg(args, "description")?;
    let labels: Option<Vec<String>> = extract_optional_arg(args, "labels")?;
    let parent_id: Option<String> = extract_optional_arg(args, "parent_id")?;
    let status_id: Option<String> = extract_optional_arg(args, "status_id")?;
    let result = create_card_inner(
        &state.config,
        &project_id,
        &title,
        description,
        labels,
        parent_id,
        status_id,
    )?;
    state.event_bus.emit_maestro(MaestroEvent::CardsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_cards_get(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    let result = get_card_inner(&state.config, &project_id, &id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_cards_update(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    let title: Option<String> = extract_optional_arg(args, "title")?;
    let description: Option<String> = extract_optional_arg(args, "description")?;
    let labels: Option<Vec<String>> = extract_optional_arg(args, "labels")?;
    let result = update_card_inner(&state.config, &project_id, &id, title, description, labels)?;
    state.event_bus.emit_maestro(MaestroEvent::CardsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_cards_delete(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    delete_card_inner(&state.config, &project_id, &id)?;
    state.event_bus.emit_maestro(MaestroEvent::CardsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::json!(null))
}

fn dispatch_cards_list(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let result = list_cards_inner(&state.config, &project_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_cards_list_sub(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let parent_id: String = extract_arg(args, "parent_id")?;
    let result = list_sub_cards_inner(&state.config, &project_id, &parent_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_cards_move(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    let target_status_id: String = extract_arg(args, "target_status_id")?;
    let target_sort_order: i32 = extract_arg(args, "target_sort_order")?;
    let result =
        move_card_inner(&state.config, &project_id, &id, &target_status_id, target_sort_order)?;
    state.event_bus.emit_maestro(MaestroEvent::CardsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_cards_reorder(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let status_id: String = extract_arg(args, "status_id")?;
    let card_ids: Vec<String> = extract_arg(args, "card_ids")?;
    let result = reorder_cards_inner(&state.config, &project_id, &status_id, &card_ids)?;
    state.event_bus.emit_maestro(MaestroEvent::CardsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

// ============================================================================
// Question dispatchers
// ============================================================================

fn dispatch_questions_create(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let question: String = extract_arg(args, "question")?;
    let source: String = extract_arg(args, "source")?;
    let result = create_question_inner(&state.config, &project_id, &card_id, &question, &source)?;
    state.event_bus.emit_maestro(MaestroEvent::QuestionsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_questions_list(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let result = list_questions_inner(&state.config, &project_id, &card_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_questions_resolve(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    let resolution: Option<String> = extract_optional_arg(args, "resolution")?;
    let resolved_by: String = extract_arg(args, "resolved_by")?;
    let result = resolve_question_inner(&state.config, &project_id, &id, resolution, &resolved_by)?;
    state.event_bus.emit_maestro(MaestroEvent::QuestionsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_questions_unresolve(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    let result = unresolve_question_inner(&state.config, &project_id, &id)?;
    state.event_bus.emit_maestro(MaestroEvent::QuestionsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_questions_delete(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    delete_question_inner(&state.config, &project_id, &id)?;
    state.event_bus.emit_maestro(MaestroEvent::QuestionsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::json!(null))
}

fn dispatch_questions_count_unresolved(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_ids: Vec<String> = extract_arg(args, "card_ids")?;
    let result = count_unresolved_questions_inner(&state.config, &project_id, &card_ids)?;
    Ok(serde_json::to_value(result).unwrap())
}

// ============================================================================
// Artifact dispatchers
// ============================================================================

fn dispatch_artifacts_create(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let name: String = extract_arg(args, "name")?;
    let content: String = extract_arg(args, "content")?;
    let created_by: String = extract_arg(args, "created_by")?;
    let result =
        create_artifact_inner(&state.config, &project_id, &card_id, &name, &content, &created_by)?;
    state.event_bus.emit_maestro(MaestroEvent::ArtifactsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_artifacts_read(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    let result = read_artifact_inner(&state.config, &project_id, &id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_artifacts_update(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    let content: String = extract_arg(args, "content")?;
    let result = update_artifact_inner(&state.config, &project_id, &id, &content)?;
    state.event_bus.emit_maestro(MaestroEvent::ArtifactsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_artifacts_delete(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    delete_artifact_inner(&state.config, &project_id, &id)?;
    state.event_bus.emit_maestro(MaestroEvent::ArtifactsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::json!(null))
}

fn dispatch_artifacts_list(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let result = list_artifacts_inner(&state.config, &project_id, &card_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

// ============================================================================
// Directory dispatchers
// ============================================================================

fn dispatch_directories_add(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let path: String = extract_arg(args, "path")?;
    let label: String = extract_arg(args, "label")?;
    let result = add_linked_directory_inner(&state.config, &project_id, &path, &label)?;
    state.event_bus.emit_maestro(MaestroEvent::DirectoriesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_directories_remove(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let id: String = extract_arg(args, "id")?;
    remove_linked_directory_inner(&state.config, &project_id, &id)?;
    state.event_bus.emit_maestro(MaestroEvent::DirectoriesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::json!(null))
}

fn dispatch_directories_list(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let result = list_linked_directories_inner(&state.config, &project_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

// ============================================================================
// Conversation dispatchers
// ============================================================================

fn dispatch_conversations_create(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let agent_type: String = extract_arg(args, "agent_type")?;
    let result = create_conversation_inner(&state.config, &project_id, &card_id, &agent_type)?;
    state.event_bus.emit_maestro(MaestroEvent::ConversationsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_conversations_list(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let result = list_conversations_inner(&state.config, &project_id, &card_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_conversations_create_message(
    state: &AppState,
    args: &Value,
) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let conversation_id: String = extract_arg(args, "conversation_id")?;
    let role: String = extract_arg(args, "role")?;
    let content: String = extract_arg(args, "content")?;
    let result =
        create_message_inner(&state.config, &project_id, &conversation_id, &role, &content)?;
    state.event_bus.emit_maestro(MaestroEvent::ConversationsChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_conversations_list_messages(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let conversation_id: String = extract_arg(args, "conversation_id")?;
    let result = list_messages_inner(&state.config, &project_id, &conversation_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_conversations_count_messages(
    state: &AppState,
    args: &Value,
) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let conversation_ids: Vec<String> = extract_arg(args, "conversation_ids")?;
    let result = count_conversation_messages_inner(&state.config, &project_id, &conversation_ids)?;
    Ok(serde_json::to_value(result).unwrap())
}

// ============================================================================
// Worktree dispatchers
// ============================================================================

fn dispatch_worktrees_generate_branch_name(
    _state: &AppState,
    args: &Value,
) -> Result<Value, AppError> {
    let card_id: String = extract_arg(args, "card_id")?;
    let title: String = extract_arg(args, "title")?;
    let result = generate_branch_name_inner(&card_id, &title);
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_worktrees_create(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let repo_path: String = extract_arg(args, "repo_path")?;
    let branch_name: String = extract_arg(args, "branch_name")?;
    let result =
        create_worktree_inner(&state.config, &project_id, &card_id, &repo_path, &branch_name)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_worktrees_check_exists(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let branch_slug: String = extract_arg(args, "branch_slug")?;
    let result = check_worktree_exists_inner(&state.config, &project_id, &card_id, &branch_slug)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_worktrees_get_card(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let result = get_card_worktree_inner(&state.config, &project_id, &card_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_worktrees_get_claude_path(_state: &AppState, args: &Value) -> Result<Value, AppError> {
    let repo_path: String = extract_arg(args, "repo_path")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let title: String = extract_arg(args, "title")?;
    let result = get_claude_worktree_path_inner(&repo_path, &card_id, &title);
    Ok(serde_json::to_value(result).unwrap())
}

// ============================================================================
// Agent dispatchers (async)
// ============================================================================

async fn dispatch_agent_launch(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let status_id: String = extract_arg(args, "status_id")?;
    let worktree_path: Option<String> = extract_optional_arg(args, "worktree_path")?;
    let branch_name: Option<String> = extract_optional_arg(args, "branch_name")?;
    let repo_path: Option<String> = extract_optional_arg(args, "repo_path")?;

    let result = launch_agent_inner(
        None,
        Some(state.event_bus.clone()),
        &state.config,
        &state.registry,
        &project_id,
        &card_id,
        &status_id,
        worktree_path,
        branch_name,
        repo_path,
    )
    .await?;
    state.event_bus.emit_maestro(MaestroEvent::WorkspacesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

async fn dispatch_agent_send_input(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let workspace_id: String = extract_arg(args, "workspace_id")?;
    let text: String = extract_arg(args, "text")?;
    send_agent_input_inner(&state.registry, &workspace_id, &text).await?;
    Ok(serde_json::json!(null))
}

async fn dispatch_agent_stop(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let workspace_id: String = extract_arg(args, "workspace_id")?;
    let result = stop_agent_inner(&state.config, &state.registry, &project_id, &workspace_id).await?;
    state.event_bus.emit_maestro(MaestroEvent::WorkspacesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

async fn dispatch_agent_resume(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let workspace_id: String = extract_arg(args, "workspace_id")?;
    let card_id: String = extract_arg(args, "card_id")?;

    let result = resume_agent_inner(
        None,
        Some(state.event_bus.clone()),
        &state.config,
        &state.registry,
        &project_id,
        &workspace_id,
        &card_id,
    )
    .await?;
    state.event_bus.emit_maestro(MaestroEvent::WorkspacesChanged {
        project_id: project_id.clone(),
    });
    Ok(serde_json::to_value(result).unwrap())
}

async fn dispatch_agent_stop_all(state: &AppState, _args: &Value) -> Result<Value, AppError> {
    stop_all_agents_inner(&state.config, &state.registry).await?;
    Ok(serde_json::json!(null))
}

fn dispatch_agent_list_workspaces(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let card_id: String = extract_arg(args, "card_id")?;
    let result = list_workspaces_inner(&state.config, &project_id, &card_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_agent_get_workspace(state: &AppState, args: &Value) -> Result<Value, AppError> {
    let project_id: String = extract_arg(args, "project_id")?;
    let workspace_id: String = extract_arg(args, "workspace_id")?;
    let result = get_workspace_inner(&state.config, &project_id, &workspace_id)?;
    Ok(serde_json::to_value(result).unwrap())
}

fn dispatch_agent_list_running(state: &AppState, _args: &Value) -> Result<Value, AppError> {
    let result = list_running_workspaces_inner(&state.config)?;
    Ok(serde_json::to_value(result).unwrap())
}
