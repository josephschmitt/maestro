use std::path::PathBuf;

use tauri::{AppHandle, Emitter};

use crate::commands::projects::open_project_db;
use crate::fs::artifacts::{ensure_artifact_dir, name_to_slug, write_artifact_file};
use crate::ipc::protocol::{IpcRequest, IpcResponse};

pub fn handle_request(
    app: &AppHandle,
    base_path: &PathBuf,
    project_id: &str,
    request: IpcRequest,
) -> IpcResponse {
    match request.command.as_str() {
        "question" => handle_question(app, base_path, project_id, &request),
        "resolve-question" => handle_resolve_question(app, base_path, project_id, &request),
        "add-artifact" => handle_add_artifact(app, base_path, project_id, &request),
        "set-status" => handle_set_status(app, base_path, project_id, &request),
        "log" => handle_log(app, base_path, project_id, &request),
        "get-card" => handle_get_card(base_path, project_id, &request),
        "get-artifacts" => handle_get_artifacts(base_path, project_id, &request),
        "get-parent" => handle_get_parent(base_path, project_id, &request),
        other => IpcResponse::error(format!("Unknown command: {other}")),
    }
}

fn handle_question(
    app: &AppHandle,
    base_path: &PathBuf,
    project_id: &str,
    request: &IpcRequest,
) -> IpcResponse {
    let question = match request.payload.get("question").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return IpcResponse::error("Missing 'question' in payload"),
    };

    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(e) => return IpcResponse::error(e),
    };

    db.with_conn(|conn| {
        let card_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM cards WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![request.card_id, project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to verify card: {e}"))?;

        if !card_exists {
            return Ok(IpcResponse::error(format!(
                "Card {} not found",
                request.card_id
            )));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO open_questions (id, card_id, question, source, created_at) \
             VALUES (?1, ?2, ?3, 'agent', ?4)",
            rusqlite::params![id, request.card_id, question, now],
        )
        .map_err(|e| format!("Failed to create question: {e}"))?;

        let data = serde_json::json!({
            "id": id,
            "card_id": request.card_id,
            "question": question,
            "source": "agent",
            "created_at": now,
        });

        let _ = app.emit("question-created", &data);

        Ok(IpcResponse::success(data))
    })
    .unwrap_or_else(IpcResponse::error)
}

fn handle_resolve_question(
    app: &AppHandle,
    base_path: &PathBuf,
    project_id: &str,
    request: &IpcRequest,
) -> IpcResponse {
    let question_id = match request.payload.get("id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return IpcResponse::error("Missing 'id' in payload"),
    };

    let resolution = request
        .payload
        .get("resolution")
        .and_then(|v| v.as_str());

    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(e) => return IpcResponse::error(e),
    };

    db.with_conn(|conn| {
        let now = chrono::Utc::now().to_rfc3339();

        let rows_affected = conn
            .execute(
                "UPDATE open_questions SET resolution = ?1, resolved_by = 'agent', resolved_at = ?2 WHERE id = ?3",
                rusqlite::params![resolution, now, question_id],
            )
            .map_err(|e| format!("Failed to resolve question: {e}"))?;

        if rows_affected == 0 {
            return Ok(IpcResponse::error(format!(
                "Question {question_id} not found"
            )));
        }

        let data = serde_json::json!({
            "id": question_id,
            "resolved_by": "agent",
            "resolved_at": now,
        });

        let _ = app.emit("question-resolved", &data);

        Ok(IpcResponse::success(data))
    })
    .unwrap_or_else(IpcResponse::error)
}

fn handle_add_artifact(
    app: &AppHandle,
    base_path: &PathBuf,
    project_id: &str,
    request: &IpcRequest,
) -> IpcResponse {
    let name = match request.payload.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return IpcResponse::error("Missing 'name' in payload"),
    };

    let content = match request.payload.get("content").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return IpcResponse::error("Missing 'content' in payload"),
    };

    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(e) => return IpcResponse::error(e),
    };

    db.with_conn(|conn| {
        let card_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM cards WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![request.card_id, project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to verify card: {e}"))?;

        if !card_exists {
            return Ok(IpcResponse::error(format!(
                "Card {} not found",
                request.card_id
            )));
        }

        let slug = name_to_slug(name);
        if slug.is_empty() {
            return Ok(IpcResponse::error(
                "Artifact name must contain at least one alphanumeric character",
            ));
        }

        let relative_path = format!("artifacts/{}/{slug}.md", request.card_id);

        let existing_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM artifacts WHERE card_id = ?1 AND path = ?2",
                rusqlite::params![request.card_id, relative_path],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check for duplicate: {e}"))?;

        let relative_path = if existing_count > 0 {
            let id_suffix = &uuid::Uuid::new_v4().to_string()[..8];
            format!("artifacts/{}/{slug}-{id_suffix}.md", request.card_id)
        } else {
            relative_path
        };

        ensure_artifact_dir(base_path, project_id, &request.card_id)?;

        let full_path = base_path
            .join("projects")
            .join(project_id)
            .join(&relative_path);
        write_artifact_file(&full_path, content)?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO artifacts (id, card_id, name, type, path, created_by, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'markdown', ?4, 'agent', ?5, ?6)",
            rusqlite::params![id, request.card_id, name, relative_path, now, now],
        )
        .map_err(|e| format!("Failed to create artifact: {e}"))?;

        let data = serde_json::json!({
            "id": id,
            "card_id": request.card_id,
            "name": name,
            "path": relative_path,
            "created_by": "agent",
            "created_at": now,
        });

        let _ = app.emit("artifact-added", &data);

        Ok(IpcResponse::success(data))
    })
    .unwrap_or_else(IpcResponse::error)
}

fn handle_set_status(
    app: &AppHandle,
    base_path: &PathBuf,
    project_id: &str,
    request: &IpcRequest,
) -> IpcResponse {
    let status_name = match request.payload.get("status").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return IpcResponse::error("Missing 'status' in payload"),
    };

    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(e) => return IpcResponse::error(e),
    };

    db.with_conn(|conn| {
        // Find status by name (case-insensitive match, replacing hyphens with spaces)
        let normalized = status_name.replace('-', " ");

        let status_id: String = conn
            .query_row(
                "SELECT id FROM statuses WHERE project_id = ?1 AND LOWER(name) = LOWER(?2)",
                rusqlite::params![project_id, normalized],
                |row| row.get(0),
            )
            .map_err(|_| format!("Status '{status_name}' not found"))?;

        // Get current card info for sort order calculation
        let (old_status_id, old_sort_order, parent_id): (String, i32, Option<String>) = conn
            .query_row(
                "SELECT status_id, sort_order, parent_id FROM cards WHERE id = ?1",
                rusqlite::params![request.card_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("Card not found: {e}"))?;

        let now = chrono::Utc::now().to_rfc3339();

        // Close gap in old status
        if parent_id.is_some() {
            conn.execute(
                "UPDATE cards SET sort_order = sort_order - 1 WHERE status_id = ?1 AND parent_id = ?2 AND sort_order > ?3",
                rusqlite::params![old_status_id, parent_id, old_sort_order],
            )
            .map_err(|e| format!("Failed to close gap: {e}"))?;
        } else {
            conn.execute(
                "UPDATE cards SET sort_order = sort_order - 1 WHERE status_id = ?1 AND parent_id IS NULL AND sort_order > ?2",
                rusqlite::params![old_status_id, old_sort_order],
            )
            .map_err(|e| format!("Failed to close gap: {e}"))?;
        }

        // Append to end of target status
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM cards WHERE status_id = ?1 AND COALESCE(parent_id, '') = COALESCE(?2, '')",
                rusqlite::params![status_id, parent_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get max sort order: {e}"))?;

        conn.execute(
            "UPDATE cards SET status_id = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![status_id, max_order + 1, now, request.card_id],
        )
        .map_err(|e| format!("Failed to update card status: {e}"))?;

        let data = serde_json::json!({
            "card_id": request.card_id,
            "status_id": status_id,
            "status_name": status_name,
        });

        let _ = app.emit("card-status-changed", &data);

        Ok(IpcResponse::success(data))
    })
    .unwrap_or_else(IpcResponse::error)
}

fn handle_log(
    app: &AppHandle,
    _base_path: &PathBuf,
    _project_id: &str,
    request: &IpcRequest,
) -> IpcResponse {
    let message = match request.payload.get("message").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return IpcResponse::error("Missing 'message' in payload"),
    };

    let data = serde_json::json!({
        "card_id": request.card_id,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let _ = app.emit("agent-log", &data);

    IpcResponse::success(data)
}

fn handle_get_card(
    base_path: &PathBuf,
    project_id: &str,
    request: &IpcRequest,
) -> IpcResponse {
    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(e) => return IpcResponse::error(e),
    };

    db.with_conn(|conn| {
        let result = conn.query_row(
            "SELECT c.id, c.project_id, c.parent_id, c.title, c.description, c.labels, \
             s.name AS status_name, s.\"group\" AS status_group \
             FROM cards c JOIN statuses s ON c.status_id = s.id \
             WHERE c.id = ?1 AND c.project_id = ?2",
            rusqlite::params![request.card_id, project_id],
            |row| {
                let labels_json: String = row.get(5)?;
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "project_id": row.get::<_, String>(1)?,
                    "parent_id": row.get::<_, Option<String>>(2)?,
                    "title": row.get::<_, String>(3)?,
                    "description": row.get::<_, String>(4)?,
                    "labels": serde_json::from_str::<serde_json::Value>(&labels_json).unwrap_or(serde_json::json!([])),
                    "status_name": row.get::<_, String>(6)?,
                    "status_group": row.get::<_, String>(7)?,
                }))
            },
        );

        match result {
            Ok(data) => Ok(IpcResponse::success(data)),
            Err(e) => Ok(IpcResponse::error(format!("Card not found: {e}"))),
        }
    })
    .unwrap_or_else(IpcResponse::error)
}

fn handle_get_artifacts(
    base_path: &PathBuf,
    project_id: &str,
    request: &IpcRequest,
) -> IpcResponse {
    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(e) => return IpcResponse::error(e),
    };

    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id, card_id, name, type, path, created_by, created_at, updated_at \
                 FROM artifacts WHERE card_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map(rusqlite::params![request.card_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "card_id": row.get::<_, String>(1)?,
                    "name": row.get::<_, String>(2)?,
                    "type": row.get::<_, String>(3)?,
                    "path": row.get::<_, String>(4)?,
                    "created_by": row.get::<_, String>(5)?,
                    "created_at": row.get::<_, String>(6)?,
                    "updated_at": row.get::<_, String>(7)?,
                }))
            })
            .map_err(|e| format!("Failed to query artifacts: {e}"))?;

        let artifacts: Vec<serde_json::Value> = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read artifact row: {e}"))?;

        Ok(IpcResponse::success(serde_json::json!(artifacts)))
    })
    .unwrap_or_else(IpcResponse::error)
}

fn handle_get_parent(
    base_path: &PathBuf,
    project_id: &str,
    request: &IpcRequest,
) -> IpcResponse {
    let db = match open_project_db(base_path, project_id) {
        Ok(db) => db,
        Err(e) => return IpcResponse::error(e),
    };

    db.with_conn(|conn| {
        let parent_id: Option<String> = conn
            .query_row(
                "SELECT parent_id FROM cards WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![request.card_id, project_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Card not found: {e}"))?;

        let parent_id = match parent_id {
            Some(pid) => pid,
            None => return Ok(IpcResponse::success(serde_json::json!(null))),
        };

        let result = conn.query_row(
            "SELECT c.id, c.project_id, c.parent_id, c.title, c.description, c.labels, \
             s.name AS status_name, s.\"group\" AS status_group \
             FROM cards c JOIN statuses s ON c.status_id = s.id \
             WHERE c.id = ?1",
            rusqlite::params![parent_id],
            |row| {
                let labels_json: String = row.get(5)?;
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "project_id": row.get::<_, String>(1)?,
                    "parent_id": row.get::<_, Option<String>>(2)?,
                    "title": row.get::<_, String>(3)?,
                    "description": row.get::<_, String>(4)?,
                    "labels": serde_json::from_str::<serde_json::Value>(&labels_json).unwrap_or(serde_json::json!([])),
                    "status_name": row.get::<_, String>(6)?,
                    "status_group": row.get::<_, String>(7)?,
                }))
            },
        );

        match result {
            Ok(data) => Ok(IpcResponse::success(data)),
            Err(e) => Ok(IpcResponse::error(format!("Parent card not found: {e}"))),
        }
    })
    .unwrap_or_else(IpcResponse::error)
}
