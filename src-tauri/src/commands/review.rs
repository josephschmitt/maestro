use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::config::ConfigState;
use crate::commands::projects::open_project_db;
use crate::fs::diff;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffLine {
    pub line_type: String,
    pub content: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePrResult {
    pub url: String,
}

fn convert_changed_file(f: diff::ChangedFile) -> ChangedFile {
    let status = match f.status {
        diff::FileChangeStatus::Added => "A".to_string(),
        diff::FileChangeStatus::Modified => "M".to_string(),
        diff::FileChangeStatus::Deleted => "D".to_string(),
    };
    ChangedFile {
        path: f.path,
        status,
    }
}

fn convert_diff_line(l: diff::DiffLine) -> DiffLine {
    let line_type = match l.line_type {
        diff::DiffLineType::Added => "added".to_string(),
        diff::DiffLineType::Removed => "removed".to_string(),
        diff::DiffLineType::Context => "context".to_string(),
    };
    DiffLine {
        line_type,
        content: l.content,
        old_line: l.old_line,
        new_line: l.new_line,
    }
}

fn convert_file_diff(fd: diff::FileDiff) -> FileDiff {
    FileDiff {
        path: fd.path,
        hunks: fd
            .hunks
            .into_iter()
            .map(|h| DiffHunk {
                old_start: h.old_start,
                old_count: h.old_count,
                new_start: h.new_start,
                new_count: h.new_count,
                header: h.header,
                lines: h.lines.into_iter().map(convert_diff_line).collect(),
            })
            .collect(),
    }
}

#[tauri::command]
pub fn get_changed_files(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
) -> Result<Vec<ChangedFile>, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    let (worktree_path, branch_name) = db.with_conn(|conn| {
        conn.query_row(
            "SELECT worktree_path, branch_name FROM agent_workspaces \
             WHERE card_id = ?1 AND worktree_path IS NOT NULL \
             ORDER BY attached_at DESC LIMIT 1",
            rusqlite::params![card_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .map_err(|e| format!("No workspace found for card: {e}"))
    })?;

    let wt_path = worktree_path.ok_or_else(|| "No worktree path found".to_string())?;
    let _branch = branch_name.ok_or_else(|| "No branch name found".to_string())?;

    let base_branch = resolve_base_branch(&wt_path)?;
    let files = diff::get_changed_files(&wt_path, &base_branch)?;
    Ok(files.into_iter().map(convert_changed_file).collect())
}

#[tauri::command]
pub fn get_file_diff(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
    file_path: String,
) -> Result<FileDiff, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    let worktree_path = db.with_conn(|conn| {
        conn.query_row(
            "SELECT worktree_path FROM agent_workspaces \
             WHERE card_id = ?1 AND worktree_path IS NOT NULL \
             ORDER BY attached_at DESC LIMIT 1",
            rusqlite::params![card_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .map_err(|e| format!("No workspace found for card: {e}"))
    })?;

    let wt_path = worktree_path.ok_or_else(|| "No worktree path found".to_string())?;
    let base_branch = resolve_base_branch(&wt_path)?;
    let fd = diff::get_file_diff(&wt_path, &base_branch, &file_path)?;
    Ok(convert_file_diff(fd))
}

#[tauri::command]
pub fn send_back_card(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
    feedback: String,
    in_progress_status_id: String,
) -> Result<(), String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;
    let now = chrono::Utc::now().to_rfc3339();

    db.with_conn(|conn| {
        conn.execute(
            "UPDATE cards SET status_id = ?1, updated_at = ?2 WHERE id = ?3 AND project_id = ?4",
            rusqlite::params![in_progress_status_id, now, card_id, project_id],
        )
        .map_err(|e| format!("Failed to update card status: {e}"))?;

        conn.execute(
            "UPDATE agent_workspaces SET review_count = review_count + 1 \
             WHERE card_id = ?1 AND status NOT IN ('completed', 'failed')",
            rusqlite::params![card_id],
        )
        .map_err(|e| format!("Failed to increment review count: {e}"))?;

        let conv_id = get_or_create_review_conversation(conn, &card_id, &now)?;

        let msg_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO conversation_messages (id, conversation_id, role, content, timestamp) \
             VALUES (?1, ?2, 'user', ?3, ?4)",
            rusqlite::params![msg_id, conv_id, feedback, now],
        )
        .map_err(|e| format!("Failed to create review message: {e}"))?;

        Ok(())
    })
}

#[tauri::command]
pub fn approve_card(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
    completed_status_id: String,
) -> Result<(), String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;
    let now = chrono::Utc::now().to_rfc3339();

    db.with_conn(|conn| {
        conn.execute(
            "UPDATE cards SET status_id = ?1, updated_at = ?2 WHERE id = ?3 AND project_id = ?4",
            rusqlite::params![completed_status_id, now, card_id, project_id],
        )
        .map_err(|e| format!("Failed to update card status: {e}"))?;

        conn.execute(
            "UPDATE agent_workspaces SET status = 'completed', completed_at = ?1 \
             WHERE card_id = ?2 AND status NOT IN ('completed', 'failed')",
            rusqlite::params![now, card_id],
        )
        .map_err(|e| format!("Failed to complete workspaces: {e}"))?;

        Ok(())
    })
}

#[tauri::command]
pub fn create_pr(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
    title: String,
    body: String,
) -> Result<CreatePrResult, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    let (worktree_path, branch_name) = db.with_conn(|conn| {
        conn.query_row(
            "SELECT worktree_path, branch_name FROM agent_workspaces \
             WHERE card_id = ?1 AND worktree_path IS NOT NULL \
             ORDER BY attached_at DESC LIMIT 1",
            rusqlite::params![card_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .map_err(|e| format!("No workspace found for card: {e}"))
    })?;

    let wt_path = worktree_path.ok_or_else(|| "No worktree path found".to_string())?;
    let branch = branch_name.ok_or_else(|| "No branch name found".to_string())?;

    diff::push_branch(&wt_path, &branch)?;
    let url = diff::create_pull_request(&wt_path, &title, &body)?;

    Ok(CreatePrResult { url })
}

#[tauri::command]
pub fn get_review_count(
    config: State<ConfigState>,
    project_id: String,
    card_id: String,
) -> Result<i32, String> {
    let base_path = config.with_config(|c| Ok(c.resolve_base_path()))?;
    let db = open_project_db(&base_path, &project_id)?;

    db.with_conn(|conn| {
        let count: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(review_count), 0) FROM agent_workspaces WHERE card_id = ?1",
                rusqlite::params![card_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get review count: {e}"))?;
        Ok(count)
    })
}

fn get_or_create_review_conversation(
    conn: &rusqlite::Connection,
    card_id: &str,
    now: &str,
) -> Result<String, String> {
    let existing: Result<String, _> = conn.query_row(
        "SELECT id FROM conversations WHERE card_id = ?1 AND agent_type = 'review' \
         ORDER BY started_at DESC LIMIT 1",
        rusqlite::params![card_id],
        |row| row.get(0),
    );

    match existing {
        Ok(id) => Ok(id),
        Err(_) => {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO conversations (id, card_id, agent_type, started_at) \
                 VALUES (?1, ?2, 'review', ?3)",
                rusqlite::params![id, card_id, now],
            )
            .map_err(|e| format!("Failed to create review conversation: {e}"))?;
            Ok(id)
        }
    }
}

fn resolve_base_branch(worktree_path: &str) -> Result<String, String> {
    // Try common base branch names
    for branch in &["main", "master"] {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(worktree_path)
            .arg("rev-parse")
            .arg("--verify")
            .arg(format!("origin/{branch}"))
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                return Ok(format!("origin/{branch}"));
            }
        }
    }

    // Fall back to trying without origin/
    for branch in &["main", "master"] {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(worktree_path)
            .arg("rev-parse")
            .arg("--verify")
            .arg(*branch)
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                return Ok(branch.to_string());
            }
        }
    }

    Err("Could not determine base branch (tried main, master)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::projects::seed_default_statuses;
    use crate::db::DbConnection;

    fn setup_test_db() -> (DbConnection, String) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("db.sqlite");
        let db = DbConnection::open(&db_path).unwrap();
        let project_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        db.with_conn(|conn: &rusqlite::Connection| {
            conn.execute(
                "INSERT INTO projects (id, name, agent_config, created_at, updated_at) VALUES (?1, 'Test', '{}', ?2, ?3)",
                rusqlite::params![project_id, now, now],
            ).map_err(|e| format!("{e}"))?;
            seed_default_statuses(conn, &project_id)?;
            Ok(())
        }).unwrap();

        (db, project_id)
    }

    fn insert_card(conn: &rusqlite::Connection, project_id: &str, status_group: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let status_id: String = conn
            .query_row(
                "SELECT id FROM statuses WHERE project_id = ?1 AND \"group\" = ?2 AND is_default = 1",
                rusqlite::params![project_id, status_group],
                |row| row.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO cards (id, project_id, status_id, title, description, labels, sort_order, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'Test Card', 'Test description', '[]', 0, ?4, ?5)",
            rusqlite::params![id, project_id, status_id, now, now],
        )
        .unwrap();
        id
    }

    fn insert_workspace(conn: &rusqlite::Connection, card_id: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO agent_workspaces (id, card_id, agent_type, status, pid, worktree_path, branch_name, review_count, attached_at) \
             VALUES (?1, ?2, 'claude', 'reviewing', 12345, '/tmp/worktree', 'maestro/test-branch', 0, ?3)",
            rusqlite::params![id, card_id, now],
        )
        .unwrap();
        id
    }

    #[test]
    fn test_get_or_create_review_conversation_creates_new() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id, "Started");
            let now = chrono::Utc::now().to_rfc3339();

            let conv_id = get_or_create_review_conversation(conn, &card_id, &now)?;
            assert!(!conv_id.is_empty());

            let agent_type: String = conn
                .query_row(
                    "SELECT agent_type FROM conversations WHERE id = ?1",
                    rusqlite::params![conv_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("{e}"))?;
            assert_eq!(agent_type, "review");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_get_or_create_review_conversation_reuses_existing() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id, "Started");
            let now = chrono::Utc::now().to_rfc3339();

            let conv_id1 = get_or_create_review_conversation(conn, &card_id, &now)?;
            let conv_id2 = get_or_create_review_conversation(conn, &card_id, &now)?;
            assert_eq!(conv_id1, conv_id2);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_review_count_default_zero() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id, "Started");

            let count: i32 = conn
                .query_row(
                    "SELECT COALESCE(MAX(review_count), 0) FROM agent_workspaces WHERE card_id = ?1",
                    rusqlite::params![card_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("{e}"))?;
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_review_count_increments() {
        let (db, project_id) = setup_test_db();
        db.with_conn(|conn| {
            let card_id = insert_card(conn, &project_id, "Started");
            let ws_id = insert_workspace(conn, &card_id);

            conn.execute(
                "UPDATE agent_workspaces SET review_count = review_count + 1 WHERE id = ?1",
                rusqlite::params![ws_id],
            )
            .map_err(|e| format!("{e}"))?;

            let count: i32 = conn
                .query_row(
                    "SELECT review_count FROM agent_workspaces WHERE id = ?1",
                    rusqlite::params![ws_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("{e}"))?;
            assert_eq!(count, 1);
            Ok(())
        })
        .unwrap();
    }
}
