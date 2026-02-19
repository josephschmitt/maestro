use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use super::migrations::run_migrations;

pub struct DbConnection {
    conn: Mutex<Connection>,
}

impl DbConnection {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {e}"))?;
        }

        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open database at {}: {e}", path.display()))?;

        conn.execute_batch("PRAGMA foreign_keys = ON")
            .map_err(|e| format!("Failed to enable foreign keys: {e}"))?;

        conn.execute_batch("PRAGMA journal_mode = WAL")
            .map_err(|e| format!("Failed to set WAL mode: {e}"))?;

        run_migrations(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn with_conn<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> Result<T, String>,
    {
        let conn = self.conn.lock().map_err(|e| format!("Database lock poisoned: {e}"))?;
        f(&conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_creates_db() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.sqlite");
        let db = DbConnection::open(&path).unwrap();

        db.with_conn(|conn| {
            let fk: i64 = conn
                .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            assert_eq!(fk, 1);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_open_in_nested_dir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("test.sqlite");
        DbConnection::open(&path).unwrap();
        assert!(path.exists());
    }
}
