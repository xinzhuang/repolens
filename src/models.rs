use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
    pub parent_path: String,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub branch: String,
    pub last_commit_hash: String,
    pub last_commit_date: Option<NaiveDateTime>,
    pub last_commit_msg: String,
    pub file_count: i64,
    pub size_bytes: i64,
    pub readme_hash: String,
    pub readme_summary: String,
    pub scanned_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub total_found: usize,
    pub new: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub repos: Vec<Repo>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub scan_roots: Vec<String>,
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

fn default_db_path() -> String {
    let base = dirs_home().unwrap_or_else(|| "/tmp".to_string());
    format!("{}/.local/share/repolens/repolens.db", base)
}

fn dirs_home() -> Option<String> {
    std::env::var("HOME").ok()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scan_roots: Vec::new(),
            db_path: default_db_path(),
        }
    }
}
