use std::path::{Path, PathBuf};

pub fn artifact_dir(base_path: &Path, project_id: &str, card_id: &str) -> PathBuf {
    base_path
        .join("projects")
        .join(project_id)
        .join("artifacts")
        .join(card_id)
}

pub fn ensure_artifact_dir(base_path: &Path, project_id: &str, card_id: &str) -> Result<PathBuf, String> {
    let dir = artifact_dir(base_path, project_id, card_id);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create artifact directory: {e}"))?;
    Ok(dir)
}

pub fn write_artifact_file(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write artifact file: {e}"))
}

pub fn read_artifact_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read artifact file: {e}"))
}

pub fn delete_artifact_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|e| format!("Failed to delete artifact file: {e}"))?;
    }
    Ok(())
}

pub fn name_to_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_slug() {
        assert_eq!(name_to_slug("My Artifact"), "my-artifact");
        assert_eq!(name_to_slug("Hello World!"), "hello-world");
        assert_eq!(name_to_slug("  spaces  and---dashes  "), "spaces-and-dashes");
        assert_eq!(name_to_slug("CamelCase"), "camelcase");
        assert_eq!(name_to_slug("with_underscores"), "with-underscores");
    }

    #[test]
    fn test_artifact_dir() {
        let base = Path::new("/tmp/maestro");
        let dir = artifact_dir(base, "proj-1", "card-1");
        assert_eq!(dir, PathBuf::from("/tmp/maestro/projects/proj-1/artifacts/card-1"));
    }

    #[test]
    fn test_ensure_artifact_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = ensure_artifact_dir(tmp.path(), "proj-1", "card-1").unwrap();
        assert!(dir.exists());
        assert!(dir.is_dir());
    }

    #[test]
    fn test_write_and_read_artifact() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.md");
        write_artifact_file(&file_path, "# Hello").unwrap();
        let content = read_artifact_file(&file_path).unwrap();
        assert_eq!(content, "# Hello");
    }

    #[test]
    fn test_delete_artifact_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.md");
        write_artifact_file(&file_path, "content").unwrap();
        assert!(file_path.exists());
        delete_artifact_file(&file_path).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("nonexistent.md");
        delete_artifact_file(&file_path).unwrap();
    }
}
