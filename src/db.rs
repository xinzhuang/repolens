use rusqlite::{params, Connection, Result as SqlResult};
use crate::models::Repo;

pub fn create_connection(path: &str) -> SqlResult<Connection> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS repos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            parent_path TEXT NOT NULL,
            languages TEXT NOT NULL DEFAULT '[]',
            frameworks TEXT NOT NULL DEFAULT '[]',
            branch TEXT NOT NULL DEFAULT '',
            last_commit_hash TEXT NOT NULL DEFAULT '',
            last_commit_date TEXT,
            last_commit_msg TEXT NOT NULL DEFAULT '',
            file_count INTEGER NOT NULL DEFAULT 0,
            size_bytes INTEGER NOT NULL DEFAULT 0,
            readme_hash TEXT NOT NULL DEFAULT '',
            readme_summary TEXT NOT NULL DEFAULT '',
            scanned_at TEXT NOT NULL DEFAULT (datetime('now')),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS scan_roots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS repos_fts USING fts5(
            name, path, readme_summary, frameworks, languages,
            content=repos,
            content_rowid=id
        );

        CREATE TRIGGER IF NOT EXISTS repos_ai AFTER INSERT ON repos BEGIN
            INSERT INTO repos_fts(rowid, name, path, readme_summary, frameworks, languages)
            VALUES (new.id, new.name, new.path, new.readme_summary, new.frameworks, new.languages);
        END;

        CREATE TRIGGER IF NOT EXISTS repos_ad AFTER DELETE ON repos BEGIN
            INSERT INTO repos_fts(repos_fts, rowid, name, path, readme_summary, frameworks, languages)
            VALUES ('delete', old.id, old.name, old.path, old.readme_summary, old.frameworks, old.languages);
        END;

        CREATE TRIGGER IF NOT EXISTS repos_au AFTER UPDATE ON repos BEGIN
            INSERT INTO repos_fts(repos_fts, rowid, name, path, readme_summary, frameworks, languages)
            VALUES ('delete', old.id, old.name, old.path, old.readme_summary, old.frameworks, old.languages);
            INSERT INTO repos_fts(rowid, name, path, readme_summary, frameworks, languages)
            VALUES (new.id, new.name, new.path, new.readme_summary, new.frameworks, new.languages);
        END;"
    )?;
    Ok(())
}

pub fn insert_repo(conn: &Connection, repo: &Repo) -> SqlResult<i64> {
    conn.execute(
        "INSERT OR REPLACE INTO repos (name, path, parent_path, languages, frameworks, branch,
            last_commit_hash, last_commit_date, last_commit_msg, file_count, size_bytes,
            readme_hash, readme_summary, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'))",
        params![
            repo.name,
            repo.path,
            repo.parent_path,
            serde_json::to_string(&repo.languages).unwrap_or_default(),
            serde_json::to_string(&repo.frameworks).unwrap_or_default(),
            repo.branch,
            repo.last_commit_hash,
            repo.last_commit_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            repo.last_commit_msg,
            repo.file_count,
            repo.size_bytes,
            repo.readme_hash,
            repo.readme_summary,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_repo_by_name(conn: &Connection, name: &str) -> SqlResult<Option<Repo>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, parent_path, languages, frameworks, branch,
                last_commit_hash, last_commit_date, last_commit_msg, file_count, size_bytes,
                readme_hash, readme_summary, scanned_at, created_at, updated_at
         FROM repos WHERE name = ?1 LIMIT 1"
    )?;
    let mut rows = stmt.query(params![name])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_repo(row)?)),
        None => Ok(None),
    }
}

pub fn get_repo_by_path(conn: &Connection, path: &str) -> SqlResult<Option<Repo>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, parent_path, languages, frameworks, branch,
                last_commit_hash, last_commit_date, last_commit_msg, file_count, size_bytes,
                readme_hash, readme_summary, scanned_at, created_at, updated_at
         FROM repos WHERE path = ?1 LIMIT 1"
    )?;
    let mut rows = stmt.query(params![path])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_repo(row)?)),
        None => Ok(None),
    }
}

pub fn list_repos(conn: &Connection, limit: i64, sort: &str) -> SqlResult<Vec<Repo>> {
    let order = match sort {
        "name" => "name ASC",
        "size" => "size_bytes DESC",
        _ => "updated_at DESC",
    };
    let sql = format!(
        "SELECT id, name, path, parent_path, languages, frameworks, branch,
                last_commit_hash, last_commit_date, last_commit_msg, file_count, size_bytes,
                readme_hash, readme_summary, scanned_at, created_at, updated_at
         FROM repos ORDER BY {} LIMIT ?1",
        order
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![limit], |row| row_to_repo(row))?;
    rows.collect()
}

pub fn row_to_repo(row: &rusqlite::Row) -> SqlResult<Repo> {
    Ok(Repo {
        id: Some(row.get(0)?),
        name: row.get(1)?,
        path: row.get(2)?,
        parent_path: row.get(3)?,
        languages: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
        frameworks: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
        branch: row.get(6)?,
        last_commit_hash: row.get(7)?,
        last_commit_date: row.get::<_, Option<String>>(8)?
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
        last_commit_msg: row.get(9)?,
        file_count: row.get(10)?,
        size_bytes: row.get(11)?,
        readme_hash: row.get(12)?,
        readme_summary: row.get(13)?,
        scanned_at: row.get::<_, Option<String>>(14)?
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
        created_at: row.get::<_, Option<String>>(15)?
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
        updated_at: row.get::<_, Option<String>>(16)?
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
    })
}

pub fn delete_all_repos(conn: &Connection) -> SqlResult<usize> {
    Ok(conn.execute("DELETE FROM repos", [])?)
}

pub fn count_repos(conn: &Connection) -> SqlResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM repos", [], |row| row.get(0))
}

pub fn add_scan_root(conn: &Connection, path: &str) -> SqlResult<()> {
    conn.execute("INSERT OR IGNORE INTO scan_roots (path) VALUES (?1)", params![path])?;
    Ok(())
}

pub fn remove_scan_root(conn: &Connection, path: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM scan_roots WHERE path = ?1", params![path])?;
    Ok(())
}

pub fn get_scan_roots(conn: &Connection) -> SqlResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM scan_roots")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn sample_repo(name: &str, path: &str) -> Repo {
        Repo {
            id: None,
            name: name.to_string(),
            path: path.to_string(),
            parent_path: format!("{}/..", path),
            languages: vec!["Rust".to_string()],
            frameworks: vec!["clap".to_string()],
            branch: "main".to_string(),
            last_commit_hash: "abc123".to_string(),
            last_commit_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
            last_commit_msg: "initial commit".to_string(),
            file_count: 10,
            size_bytes: 1024,
            readme_hash: "hash123".to_string(),
            readme_summary: "A test project".to_string(),
            scanned_at: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_create_db_and_init_schema() {
        let conn = test_conn();
        assert_eq!(count_repos(&conn).unwrap(), 0);
    }

    #[test]
    fn test_insert_and_query_repo() {
        let conn = test_conn();
        let repo = sample_repo("myrepo", "/tmp/myrepo");
        let id = insert_repo(&conn, &repo).unwrap();
        assert!(id > 0);

        let found = get_repo_by_name(&conn, "myrepo").unwrap().unwrap();
        assert_eq!(found.name, "myrepo");
        assert_eq!(found.path, "/tmp/myrepo");
        assert_eq!(found.languages, vec!["Rust"]);
        assert_eq!(found.file_count, 10);
    }

    #[test]
    fn test_insert_duplicate_path_replaces() {
        let conn = test_conn();
        let mut repo = sample_repo("myrepo", "/tmp/myrepo");
        insert_repo(&conn, &repo).unwrap();
        repo.file_count = 20;
        insert_repo(&conn, &repo).unwrap();
        assert_eq!(count_repos(&conn).unwrap(), 1);
        let found = get_repo_by_name(&conn, "myrepo").unwrap().unwrap();
        assert_eq!(found.file_count, 20);
    }

    #[test]
    fn test_get_repo_not_found() {
        let conn = test_conn();
        assert!(get_repo_by_name(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_list_repos() {
        let conn = test_conn();
        insert_repo(&conn, &sample_repo("repo-a", "/a")).unwrap();
        insert_repo(&conn, &sample_repo("repo-b", "/b")).unwrap();
        let repos = list_repos(&conn, 100, "name").unwrap();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].name, "repo-a");
    }

    #[test]
    fn test_scan_roots_crud() {
        let conn = test_conn();
        add_scan_root(&conn, "/home/user/projects").unwrap();
        add_scan_root(&conn, "/opt/repos").unwrap();
        let roots = get_scan_roots(&conn).unwrap();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&"/home/user/projects".to_string()));

        remove_scan_root(&conn, "/home/user/projects").unwrap();
        let roots = get_scan_roots(&conn).unwrap();
        assert_eq!(roots.len(), 1);
    }

    #[test]
    fn test_delete_all_repos() {
        let conn = test_conn();
        insert_repo(&conn, &sample_repo("a", "/a")).unwrap();
        insert_repo(&conn, &sample_repo("b", "/b")).unwrap();
        delete_all_repos(&conn).unwrap();
        assert_eq!(count_repos(&conn).unwrap(), 0);
    }
}
