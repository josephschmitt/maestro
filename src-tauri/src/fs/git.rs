use std::path::Path;

pub fn is_git_repo(path: &str) -> bool {
    Path::new(path).join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_repo_detects_repo() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        assert!(is_git_repo(dir.path().to_str().unwrap()));
    }

    #[test]
    fn test_is_git_repo_returns_false_for_non_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!is_git_repo(dir.path().to_str().unwrap()));
    }

    #[test]
    fn test_is_git_repo_returns_false_for_nonexistent_path() {
        assert!(!is_git_repo("/tmp/definitely-does-not-exist-12345"));
    }
}
