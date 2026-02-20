use std::path::{Path, PathBuf};
use std::process::Command;

use crate::fs::artifacts::name_to_slug;

pub fn worktree_dir(base_path: &Path, project_id: &str) -> PathBuf {
    base_path.join("projects").join(project_id).join("worktrees")
}

pub fn worktree_path(
    base_path: &Path,
    project_id: &str,
    card_id: &str,
    branch_slug: &str,
) -> PathBuf {
    let card_short = &card_id[..8.min(card_id.len())];
    worktree_dir(base_path, project_id).join(format!("{card_short}-{branch_slug}"))
}

pub fn generate_branch_name(card_id: &str, title: &str) -> String {
    let card_short = &card_id[..8.min(card_id.len())];
    let slug = name_to_slug(title);
    let slug_truncated = truncate_slug(&slug, 40);
    format!("maestro/{card_short}-{slug_truncated}")
}

fn truncate_slug(slug: &str, max_len: usize) -> &str {
    if slug.len() <= max_len {
        return slug;
    }
    let truncated = &slug[..max_len];
    truncated.trim_end_matches('-')
}

pub fn create_worktree(
    repo_path: &str,
    worktree_path: &Path,
    branch_name: &str,
) -> Result<(), String> {
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create worktree parent directory: {e}"))?;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("worktree")
        .arg("add")
        .arg(worktree_path.to_string_lossy().as_ref())
        .arg("-b")
        .arg(branch_name)
        .output()
        .map_err(|e| format!("Failed to run git worktree add: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git worktree add failed: {stderr}"));
    }

    Ok(())
}

pub fn worktree_exists(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

pub fn branch_slug_from_title(title: &str) -> String {
    let slug = name_to_slug(title);
    truncate_slug(&slug, 40).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_branch_name() {
        let name = generate_branch_name("a1b2c3d4-5678-abcd-efgh-ijklmnopqrst", "Add Auth");
        assert_eq!(name, "maestro/a1b2c3d4-add-auth");
    }

    #[test]
    fn test_generate_branch_name_long_title() {
        let name = generate_branch_name(
            "a1b2c3d4-5678-abcd-efgh-ijklmnopqrst",
            "This is a very long title that should be truncated at forty characters",
        );
        assert!(name.starts_with("maestro/a1b2c3d4-"));
        let slug_part = name.strip_prefix("maestro/a1b2c3d4-").unwrap();
        assert!(slug_part.len() <= 40);
        assert!(!slug_part.ends_with('-'));
    }

    #[test]
    fn test_generate_branch_name_special_chars() {
        let name = generate_branch_name("abcdef12-xxxx", "Fix bug: handle 404 errors!!!");
        assert_eq!(name, "maestro/abcdef12-fix-bug-handle-404-errors");
    }

    #[test]
    fn test_truncate_slug() {
        assert_eq!(truncate_slug("hello-world", 20), "hello-world");
        assert_eq!(truncate_slug("abcde-fghij", 5), "abcde");
        assert_eq!(truncate_slug("abcd-", 4), "abcd");
    }

    #[test]
    fn test_worktree_path() {
        let base = Path::new("/tmp/maestro");
        let path = worktree_path(base, "proj-1", "a1b2c3d4-full-uuid", "add-auth");
        assert_eq!(
            path,
            PathBuf::from("/tmp/maestro/projects/proj-1/worktrees/a1b2c3d4-add-auth")
        );
    }

    #[test]
    fn test_worktree_exists_false_for_nonexistent() {
        assert!(!worktree_exists(Path::new("/tmp/definitely-not-a-real-worktree")));
    }

    #[test]
    fn test_worktree_exists_true_for_directory() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(worktree_exists(tmp.path()));
    }

    #[test]
    fn test_branch_slug_from_title() {
        assert_eq!(branch_slug_from_title("Add Auth"), "add-auth");
        assert_eq!(branch_slug_from_title("Fix Bug #42"), "fix-bug-42");
    }
}
