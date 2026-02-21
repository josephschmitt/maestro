use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FileChangeStatus {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub status: FileChangeStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DiffLineType {
    Added,
    Removed,
    Context,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
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

pub fn get_changed_files(worktree_path: &str, base_branch: &str) -> Result<Vec<ChangedFile>, String> {
    let merge_base = find_merge_base(worktree_path, base_branch)?;

    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("diff")
        .arg("--name-status")
        .arg(&merge_base)
        .output()
        .map_err(|e| format!("Failed to run git diff: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git diff --name-status failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_name_status(&stdout)
}

pub fn get_file_diff(worktree_path: &str, base_branch: &str, file_path: &str) -> Result<FileDiff, String> {
    let merge_base = find_merge_base(worktree_path, base_branch)?;

    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("diff")
        .arg(&merge_base)
        .arg("--")
        .arg(file_path)
        .output()
        .map_err(|e| format!("Failed to run git diff: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git diff failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_unified_diff(&stdout, file_path)
}

fn find_merge_base(worktree_path: &str, base_branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("merge-base")
        .arg(base_branch)
        .arg("HEAD")
        .output()
        .map_err(|e| format!("Failed to run git merge-base: {e}"))?;

    if !output.status.success() {
        return Ok(base_branch.to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_name_status(output: &str) -> Result<Vec<ChangedFile>, String> {
    let mut files = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() != 2 {
            continue;
        }
        let status = match parts[0].chars().next() {
            Some('A') => FileChangeStatus::Added,
            Some('M') => FileChangeStatus::Modified,
            Some('D') => FileChangeStatus::Deleted,
            _ => FileChangeStatus::Modified,
        };
        files.push(ChangedFile {
            path: parts[1].to_string(),
            status,
        });
    }
    Ok(files)
}

fn parse_unified_diff(diff_text: &str, file_path: &str) -> Result<FileDiff, String> {
    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line: u32 = 0;
    let mut new_line: u32 = 0;

    for line in diff_text.lines() {
        if line.starts_with("@@") {
            if let Some(hunk) = current_hunk.take() {
                hunks.push(hunk);
            }
            if let Some((os, oc, ns, nc, header)) = parse_hunk_header(line) {
                old_line = os;
                new_line = ns;
                current_hunk = Some(DiffHunk {
                    old_start: os,
                    old_count: oc,
                    new_start: ns,
                    new_count: nc,
                    header: header.to_string(),
                    lines: Vec::new(),
                });
            }
        } else if let Some(ref mut hunk) = current_hunk {
            if line.starts_with('+') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    content: line[1..].to_string(),
                    old_line: None,
                    new_line: Some(new_line),
                });
                new_line += 1;
            } else if line.starts_with('-') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Removed,
                    content: line[1..].to_string(),
                    old_line: Some(old_line),
                    new_line: None,
                });
                old_line += 1;
            } else if line.starts_with(' ') {
                let content = line[1..].to_string();
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content,
                    old_line: Some(old_line),
                    new_line: Some(new_line),
                });
                old_line += 1;
                new_line += 1;
            }
        }
    }

    if let Some(hunk) = current_hunk {
        hunks.push(hunk);
    }

    Ok(FileDiff {
        path: file_path.to_string(),
        hunks,
    })
}

fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32, &str)> {
    // Format: @@ -old_start,old_count +new_start,new_count @@ optional header
    let rest = line.strip_prefix("@@ ")?;
    let end_idx = rest.find(" @@")?;
    let range_part = &rest[..end_idx];
    let header = rest[end_idx + 3..].trim_start();

    let parts: Vec<&str> = range_part.split(' ').collect();
    if parts.len() != 2 {
        return None;
    }

    let (old_start, old_count) = parse_range(parts[0].strip_prefix('-')?)?;
    let (new_start, new_count) = parse_range(parts[1].strip_prefix('+')?)?;

    Some((old_start, old_count, new_start, new_count, header))
}

fn parse_range(s: &str) -> Option<(u32, u32)> {
    if let Some((start, count)) = s.split_once(',') {
        Some((start.parse().ok()?, count.parse().ok()?))
    } else {
        Some((s.parse().ok()?, 1))
    }
}

pub fn push_branch(worktree_path: &str, branch_name: &str) -> Result<(), String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg(branch_name)
        .output()
        .map_err(|e| format!("Failed to run git push: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git push failed: {stderr}"));
    }

    Ok(())
}

pub fn create_pull_request(
    worktree_path: &str,
    title: &str,
    body: &str,
) -> Result<String, String> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("create")
        .arg("--title")
        .arg(title)
        .arg("--body")
        .arg(body)
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to run gh pr create: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh pr create failed: {stderr}"));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_status() {
        let input = "A\tsrc/new-file.ts\nM\tsrc/existing.ts\nD\tsrc/old-file.ts\n";
        let files = parse_name_status(input).unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].path, "src/new-file.ts");
        assert_eq!(files[0].status, FileChangeStatus::Added);
        assert_eq!(files[1].path, "src/existing.ts");
        assert_eq!(files[1].status, FileChangeStatus::Modified);
        assert_eq!(files[2].path, "src/old-file.ts");
        assert_eq!(files[2].status, FileChangeStatus::Deleted);
    }

    #[test]
    fn test_parse_name_status_empty() {
        let files = parse_name_status("").unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_unified_diff() {
        let diff = r#"diff --git a/src/foo.ts b/src/foo.ts
index abc1234..def5678 100644
--- a/src/foo.ts
+++ b/src/foo.ts
@@ -1,5 +1,6 @@ function foo() {
 line 1
-old line 2
+new line 2
+added line
 line 3
 line 4
 line 5
"#;
        let result = parse_unified_diff(diff, "src/foo.ts").unwrap();
        assert_eq!(result.path, "src/foo.ts");
        assert_eq!(result.hunks.len(), 1);

        let hunk = &result.hunks[0];
        assert_eq!(hunk.old_start, 1);
        assert_eq!(hunk.old_count, 5);
        assert_eq!(hunk.new_start, 1);
        assert_eq!(hunk.new_count, 6);
        // 1 context + 1 removed + 2 added + 3 context = 7 lines
        assert_eq!(hunk.lines.len(), 7);

        assert_eq!(hunk.lines[0].line_type, DiffLineType::Context);
        assert_eq!(hunk.lines[0].content, "line 1");
        assert_eq!(hunk.lines[0].old_line, Some(1));
        assert_eq!(hunk.lines[0].new_line, Some(1));

        assert_eq!(hunk.lines[1].line_type, DiffLineType::Removed);
        assert_eq!(hunk.lines[1].content, "old line 2");
        assert_eq!(hunk.lines[1].old_line, Some(2));
        assert_eq!(hunk.lines[1].new_line, None);

        assert_eq!(hunk.lines[2].line_type, DiffLineType::Added);
        assert_eq!(hunk.lines[2].content, "new line 2");
        assert_eq!(hunk.lines[2].old_line, None);
        assert_eq!(hunk.lines[2].new_line, Some(2));

        assert_eq!(hunk.lines[3].line_type, DiffLineType::Added);
        assert_eq!(hunk.lines[3].content, "added line");
        assert_eq!(hunk.lines[3].old_line, None);
        assert_eq!(hunk.lines[3].new_line, Some(3));

        assert_eq!(hunk.lines[4].line_type, DiffLineType::Context);
        assert_eq!(hunk.lines[4].content, "line 3");
        assert_eq!(hunk.lines[5].line_type, DiffLineType::Context);
        assert_eq!(hunk.lines[5].content, "line 4");
        assert_eq!(hunk.lines[6].line_type, DiffLineType::Context);
        assert_eq!(hunk.lines[6].content, "line 5");
    }

    #[test]
    fn test_parse_hunk_header() {
        let (os, oc, ns, nc, header) =
            parse_hunk_header("@@ -10,7 +12,9 @@ fn main() {").unwrap();
        assert_eq!(os, 10);
        assert_eq!(oc, 7);
        assert_eq!(ns, 12);
        assert_eq!(nc, 9);
        assert_eq!(header, "fn main() {");
    }

    #[test]
    fn test_parse_hunk_header_no_count() {
        let (os, oc, ns, nc, _) = parse_hunk_header("@@ -1 +1 @@").unwrap();
        assert_eq!(os, 1);
        assert_eq!(oc, 1);
        assert_eq!(ns, 1);
        assert_eq!(nc, 1);
    }

    #[test]
    fn test_parse_range() {
        assert_eq!(parse_range("10,5"), Some((10, 5)));
        assert_eq!(parse_range("1"), Some((1, 1)));
    }

    #[test]
    fn test_parse_unified_diff_multiple_hunks() {
        let diff = r#"diff --git a/file.rs b/file.rs
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,3 @@
 line 1
-old
+new
 line 3
@@ -10,3 +10,4 @@
 line 10
 line 11
+inserted
 line 12
"#;
        let result = parse_unified_diff(diff, "file.rs").unwrap();
        assert_eq!(result.hunks.len(), 2);
        assert_eq!(result.hunks[0].old_start, 1);
        assert_eq!(result.hunks[1].old_start, 10);
        assert_eq!(result.hunks[1].lines.len(), 4);
    }

    #[test]
    fn test_parse_unified_diff_empty() {
        let result = parse_unified_diff("", "empty.ts").unwrap();
        assert!(result.hunks.is_empty());
    }
}
