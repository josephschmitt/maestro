use rusqlite::Connection;

struct Migration {
    version: i64,
    name: &'static str,
    up: MigrationFn,
}

enum MigrationFn {
    Sql(&'static str),
    Func(fn(&Connection) -> Result<(), String>),
}

fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table))
        .unwrap();
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    columns.contains(&column.to_string())
}

fn migrate_add_skills_to_statuses(conn: &Connection) -> Result<(), String> {
    if !has_column(conn, "statuses", "skills") {
        conn.execute_batch("ALTER TABLE statuses ADD COLUMN skills TEXT NOT NULL DEFAULT '[]'")
            .map_err(|e| format!("Failed to add skills column: {e}"))?;
    }

    conn.execute_batch(
        "UPDATE statuses SET skills = '[\"brainstorming\"]' WHERE name = 'Backlog';\n\
         UPDATE statuses SET skills = '[\"tdd\",\"systematic-debugging\",\"verification\"]' WHERE name = 'In Progress';\n\
         UPDATE statuses SET skills = '[\"code-review\",\"verification\"]' WHERE name = 'In Review';",
    )
    .map_err(|e| format!("Failed to backfill skills: {e}"))?;

    Ok(())
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_schema",
        up: MigrationFn::Sql(include_str!("schema.sql")),
    },
    Migration {
        version: 2,
        name: "add_skills_to_statuses",
        up: MigrationFn::Func(migrate_add_skills_to_statuses),
    },
];

pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
    )
    .map_err(|e| format!("Failed to create migrations table: {e}"))?;

    let applied: Vec<i64> = conn
        .prepare("SELECT version FROM _migrations ORDER BY version")
        .map_err(|e| format!("Failed to query migrations: {e}"))?
        .query_map([], |row| row.get(0))
        .map_err(|e| format!("Failed to read migrations: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect migrations: {e}"))?;

    for migration in MIGRATIONS {
        if applied.contains(&migration.version) {
            continue;
        }

        match &migration.up {
            MigrationFn::Sql(sql) => {
                conn.execute_batch(sql)
                    .map_err(|e| format!("Migration {} ({}) failed: {e}", migration.version, migration.name))?;
            }
            MigrationFn::Func(func) => {
                func(conn)
                    .map_err(|e| format!("Migration {} ({}) failed: {e}", migration.version, migration.name))?;
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO _migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![migration.version, migration.name, now],
        )
        .map_err(|e| format!("Failed to record migration {}: {e}", migration.version))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_run_successfully() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_all_tables_created() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
        run_migrations(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name != '_migrations' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"linked_directories".to_string()));
        assert!(tables.contains(&"statuses".to_string()));
        assert!(tables.contains(&"cards".to_string()));
        assert!(tables.contains(&"open_questions".to_string()));
        assert!(tables.contains(&"conversations".to_string()));
        assert!(tables.contains(&"conversation_messages".to_string()));
        assert!(tables.contains(&"agent_workspaces".to_string()));
        assert!(tables.contains(&"artifacts".to_string()));
    }

    #[test]
    fn test_skills_column_exists_after_migration() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
        run_migrations(&conn).unwrap();

        assert!(has_column(&conn, "statuses", "skills"));
    }
}
