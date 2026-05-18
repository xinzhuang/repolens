use crate::models::{Repo, ScanResult, SearchResult};

pub fn format_repos_table(repos: &[Repo]) -> String {
    if repos.is_empty() {
        return "No repositories found.".to_string();
    }

    let name_w = repos.iter().map(|r| r.name.len()).max().unwrap_or(10).max(4);
    let lang_w = repos.iter().map(|r| r.languages.join(", ").len()).max().unwrap_or(10).max(4);
    let branch_w = repos.iter().map(|r| r.branch.len()).max().unwrap_or(6).max(6);

    let mut out = String::new();
    out.push_str(&format!(
        "{:<name_w$}  {:<lang_w$}  {:<branch_w$}  {}\n",
        "NAME", "LANG", "BRANCH", "PATH", name_w = name_w, lang_w = lang_w, branch_w = branch_w
    ));
    out.push_str(&format!(
        "{}  {}  {}  {}\n",
        "-".repeat(name_w), "-".repeat(lang_w), "-".repeat(branch_w), "-".repeat(40)
    ));

    for repo in repos {
        let langs = repo.languages.join(", ");
        out.push_str(&format!(
            "{:<name_w$}  {:<lang_w$}  {:<branch_w$}  {}\n",
            repo.name, langs, repo.branch, repo.path,
            name_w = name_w, lang_w = lang_w, branch_w = branch_w
        ));
    }
    out
}

pub fn format_repo_detail(repo: &Repo) -> String {
    let langs = repo.languages.join(", ");
    let frameworks = repo.frameworks.join(", ");
    let date = repo.last_commit_date
        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default();

    format!(
        "Name:            {}\n\
         Path:            {}\n\
         Languages:       {}\n\
         Frameworks:      {}\n\
         Branch:          {}\n\
         Last Commit:     {}\n\
         Commit Message:  {}\n\
         File Count:      {}\n\
         Size:            {}\n\
         {}\
         {}",
        repo.name,
        repo.path,
        if langs.is_empty() { "N/A".into() } else { langs },
        if frameworks.is_empty() { "N/A".into() } else { frameworks },
        repo.branch,
        date,
        repo.last_commit_msg,
        repo.file_count,
        format_size(repo.size_bytes),
        if repo.readme_summary.is_empty() { String::new() } else {
            format!("README Summary:  {}\n", &repo.readme_summary[..repo.readme_summary.len().min(200)])
        },
        if repo.id.is_some() { format!("ID:              {}\n", repo.id.unwrap()) } else { String::new() },
    )
}

pub fn format_scan_result(result: &ScanResult, quiet: bool) -> String {
    if quiet {
        return String::new();
    }
    format!(
        "Scan complete: {} repos found ({} new, {} updated, {} unchanged){}",
        result.total_found, result.new, result.updated, result.unchanged,
        if result.errors.is_empty() { String::new() } else {
            format!("\nErrors:\n{}", result.errors.iter().map(|e| format!("  - {}", e)).collect::<Vec<_>>().join("\n"))
        }
    )
}

pub fn format_search_result_json(result: &SearchResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_default()
}

pub fn format_repo_json(repo: &Repo) -> String {
    serde_json::to_string_pretty(repo).unwrap_or_default()
}

pub fn format_repos_json(repos: &[Repo]) -> String {
    serde_json::to_string_pretty(repos).unwrap_or_default()
}

fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = 1024 * KB;
    const GB: i64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_repo(name: &str) -> Repo {
        Repo {
            id: Some(1),
            name: name.to_string(),
            path: format!("/tmp/{}", name),
            parent_path: "/tmp".to_string(),
            languages: vec!["Rust".to_string()],
            frameworks: vec!["clap".to_string()],
            branch: "main".to_string(),
            last_commit_hash: "abc123".to_string(),
            last_commit_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
            last_commit_msg: "test commit".to_string(),
            file_count: 42,
            size_bytes: 1024 * 512,
            readme_hash: "hash".to_string(),
            readme_summary: "A test project".to_string(),
            scanned_at: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_format_repos_table() {
        let repos = vec![sample_repo("myrepo")];
        let table = format_repos_table(&repos);
        assert!(table.contains("myrepo"));
        assert!(table.contains("Rust"));
        assert!(table.contains("/tmp/myrepo"));
    }

    #[test]
    fn test_format_repos_table_empty() {
        let table = format_repos_table(&[]);
        assert!(table.contains("No repositories"));
    }

    #[test]
    fn test_format_repo_detail() {
        let repo = sample_repo("myrepo");
        let detail = format_repo_detail(&repo);
        assert!(detail.contains("myrepo"));
        assert!(detail.contains("Rust"));
        assert!(detail.contains("clap"));
        assert!(detail.contains("42"));
    }

    #[test]
    fn test_format_repo_json() {
        let repo = sample_repo("myrepo");
        let json = format_repo_json(&repo);
        assert!(json.contains("myrepo"));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "myrepo");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024 * 3), "3.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 2), "2.0 GB");
    }
}
