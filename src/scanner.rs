use std::path::{Path, PathBuf};
use crate::models::Repo;

pub fn discover_repos(roots: &[String]) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    for root in roots {
        if Path::new(root).exists() {
            find_git_dirs(Path::new(root), &mut repos);
        }
    }
    repos
}

fn find_git_dirs(root: &Path, repos: &mut Vec<PathBuf>) {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_dir() && path.file_name().is_some_and(|n| n == ".git") {
            if let Some(repo_path) = path.parent() {
                repos.push(repo_path.to_path_buf());
            }
        }
    }
}

pub fn extract_repo_info(repo_path: &Path) -> Result<Repo, String> {
    let git_dir = repo_path.join(".git");
    let repo = git2::Repository::open(&git_dir).map_err(|e| format!("Git error: {}", e))?;

    let name = repo_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let parent_path = repo_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let branch = repo.head()
        .ok()
        .and_then(|r| r.shorthand().map(|s| s.to_string()))
        .unwrap_or_default();

    let (last_commit_hash, last_commit_date, last_commit_msg) = repo.head()
        .ok()
        .and_then(|head| head.target())
        .and_then(|oid| repo.find_commit(oid).ok())
        .map(|commit| {
            let hash = format!("{}", commit.id());
            let msg = commit.message().unwrap_or_default().lines().next().unwrap_or_default().to_string();
            let timestamp = commit.time();
            let dt = chrono::DateTime::from_timestamp(timestamp.seconds(), 0)
                .map(|dt| dt.naive_utc())
                .unwrap_or_else(|| chrono::NaiveDateTime::default());
            (hash, dt, msg)
        })
        .unwrap_or_default();

    let (file_count, size_bytes) = count_files_and_size(repo_path);

    Ok(Repo {
        id: None,
        name,
        path: repo_path.to_string_lossy().to_string(),
        parent_path,
        languages: Vec::new(),
        frameworks: Vec::new(),
        branch,
        last_commit_hash,
        last_commit_date: Some(last_commit_date),
        last_commit_msg,
        file_count,
        size_bytes,
        readme_hash: String::new(),
        readme_summary: String::new(),
        scanned_at: None,
        created_at: None,
        updated_at: None,
    })
}

fn count_files_and_size(path: &Path) -> (i64, i64) {
    let mut count = 0i64;
    let mut size = 0i64;
    let walker = ignore::WalkBuilder::new(path)
        .hidden(true)
        .git_ignore(true)
        .build();

    for entry in walker.flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            count += 1;
            size += entry.metadata().map(|m| m.len()).unwrap_or(0) as i64;
        }
    }
    (count, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_repos_under_myrepos() {
        let repos = discover_repos(&["/home/frank/Project/myrepos".to_string()]);
        assert!(!repos.is_empty(), "Should find at least one repo under myrepos");
        let repolens = repos.iter().find(|r| r.file_name().is_some_and(|n| n == "repolens"));
        assert!(repolens.is_some(), "Should find repolens repo");
    }

    #[test]
    fn test_extract_repo_info_repolens() {
        let path = Path::new("/home/frank/Project/myrepos/repolens");
        let repo = extract_repo_info(path).unwrap();
        assert_eq!(repo.name, "repolens");
        assert_eq!(repo.path, "/home/frank/Project/myrepos/repolens");
        assert!(!repo.branch.is_empty());
        assert!(repo.file_count >= 0);
    }

    #[test]
    fn test_extract_nonexistent_repo() {
        let result = extract_repo_info(Path::new("/tmp/no_such_repo"));
        assert!(result.is_err());
    }
}
