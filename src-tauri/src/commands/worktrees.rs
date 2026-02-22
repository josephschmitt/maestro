use tauri::State;

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;
use crate::fs::worktrees;

pub fn generate_branch_name_inner(card_id: &str, title: &str) -> String {
    worktrees::generate_branch_name(card_id, title)
}

#[tauri::command]
pub fn generate_branch_name(card_id: String, title: String) -> String {
    generate_branch_name_inner(&card_id, &title)
}

pub fn create_worktree_inner(
    config: &ConfigState,
    project_id: &str,
    card_id: &str,
    repo_path: &str,
    branch_name: &str,
) -> Result<String, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let slug = worktrees::branch_slug_from_title(&branch_name.replace("maestro/", ""));
    let wt_path = worktrees::worktree_path(&base_path, project_id, card_id, &slug);

    if worktrees::worktree_exists(&wt_path) {
        return Ok(wt_path.to_string_lossy().to_string());
    }

    worktrees::create_worktree(repo_path, &wt_path, branch_name)?;

    Ok(wt_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn create_worktree(
    config: State<'_, ConfigState>,
    project_id: String,
    card_id: String,
    repo_path: String,
    branch_name: String,
) -> Result<String, String> {
    create_worktree_inner(&config, &project_id, &card_id, &repo_path, &branch_name)
}

pub fn check_worktree_exists_inner(
    config: &ConfigState,
    project_id: &str,
    card_id: &str,
    branch_slug: &str,
) -> Result<Option<String>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let wt_path = worktrees::worktree_path(&base_path, project_id, card_id, branch_slug);

    if worktrees::worktree_exists(&wt_path) {
        Ok(Some(wt_path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn check_worktree_exists(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
    branch_slug: String,
) -> Result<Option<String>, String> {
    check_worktree_exists_inner(&config, &project_id, &card_id, &branch_slug)
}

pub fn get_card_worktree_inner(
    config: &ConfigState,
    project_id: &str,
    card_id: &str,
) -> Result<Option<WorktreeInfo>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, project_id)?;

    db.with_conn(|conn| {
        let result: Result<(String, String), _> = conn.query_row(
            "SELECT worktree_path, branch_name FROM agent_workspaces \
             WHERE card_id = ?1 AND worktree_path IS NOT NULL \
             ORDER BY attached_at DESC LIMIT 1",
            rusqlite::params![card_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match result {
            Ok((path, branch)) => Ok(Some(WorktreeInfo { path, branch })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to query worktree: {e}")),
        }
    })
}

#[tauri::command]
pub fn get_card_worktree(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
) -> Result<Option<WorktreeInfo>, String> {
    get_card_worktree_inner(&config, &project_id, &card_id)
}

pub fn get_claude_worktree_path_inner(repo_path: &str, card_id: &str, title: &str) -> String {
    let worktree_name = worktrees::worktree_name_from_card(card_id, title);
    worktrees::claude_worktree_path(repo_path, &worktree_name)
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
pub fn get_claude_worktree_path(repo_path: String, card_id: String, title: String) -> String {
    get_claude_worktree_path_inner(&repo_path, &card_id, &title)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
}
