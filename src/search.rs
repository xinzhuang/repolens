use rusqlite::Connection;
use crate::db::row_to_repo;
use crate::models::Repo;

pub struct SearchParams {
    pub query: Option<String>,
    pub lang: Option<String>,
    pub framework: Option<String>,
    pub recent: Option<String>,
    pub path_filter: Option<String>,
    pub size_filter: Option<String>,
    pub limit: i64,
}

pub fn search_repos(conn: &Connection, params: &SearchParams) -> Result<Vec<Repo>, String> {
    let mut conditions = Vec::new();
    let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1;

    // FTS search
    let fts_join = if let Some(ref q) = params.query {
        conditions.push(format!("r.id IN (SELECT rowid FROM repos_fts WHERE repos_fts MATCH ?{})", param_idx));
        sql_params.push(Box::new(format!("{}*", q)));
        param_idx += 1;
        true
    } else {
        false
    };

    if let Some(ref lang) = params.lang {
        conditions.push(format!("r.languages LIKE ?{}", param_idx));
        sql_params.push(Box::new(format!("%\"{}\"%", lang)));
        param_idx += 1;
    }

    if let Some(ref fw) = params.framework {
        conditions.push(format!("r.frameworks LIKE ?{}", param_idx));
        sql_params.push(Box::new(format!("%\"{}\"%", fw)));
        param_idx += 1;
    }

    if let Some(ref recent) = params.recent {
        let days = match recent.as_str() {
            "7d" => 7,
            "1m" => 30,
            "1y" => 365,
            _ => 30,
        };
        conditions.push(format!("r.updated_at >= datetime('now', '-{} days')", days));
    }

    if let Some(ref pf) = params.path_filter {
        conditions.push(format!("r.path LIKE ?{}", param_idx));
        sql_params.push(Box::new(format!("%{}%", pf)));
        param_idx += 1;
    }

    if let Some(ref sf) = params.size_filter {
        if let Some(num) = sf.strip_prefix('+') {
            if let Ok(bytes) = num.parse::<i64>() {
                conditions.push(format!("r.size_bytes >= {}", bytes));
            }
        } else if let Some(num) = sf.strip_prefix('-') {
            if let Ok(bytes) = num.parse::<i64>() {
                conditions.push(format!("r.size_bytes <= {}", bytes));
            }
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let join = if fts_join { "" } else { "" };
    let sql = format!(
        "SELECT r.id, r.name, r.path, r.parent_path, r.languages, r.frameworks, r.branch,
                r.last_commit_hash, r.last_commit_date, r.last_commit_msg, r.file_count, r.size_bytes,
                r.readme_hash, r.readme_summary, r.scanned_at, r.created_at, r.updated_at
         FROM repos r {}
         {} ORDER BY r.updated_at DESC LIMIT ?{}",
        join, where_clause, param_idx
    );

    sql_params.push(Box::new(params.limit));

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("SQL error: {}", e))?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| row_to_repo(row))
        .map_err(|e| format!("Query error: {}", e))?;

    let mut repos = Vec::new();
    for row in rows {
        match row {
            Ok(r) => repos.push(r),
            Err(e) => eprintln!("Warning: skipped repo: {}", e),
        }
    }
    Ok(repos)
}

pub fn count_search_results(conn: &Connection, params: &SearchParams) -> Result<i64, String> {
    let results = search_repos(conn, params)?;
    Ok(results.len() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::Repo;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init_schema(&conn).unwrap();
        conn
    }

    fn sample_repo(name: &str, path: &str, langs: Vec<&str>, fws: Vec<&str>) -> Repo {
        Repo {
            id: None,
            name: name.to_string(),
            path: path.to_string(),
            parent_path: format!("{}/..", path),
            languages: langs.into_iter().map(|s| s.to_string()).collect(),
            frameworks: fws.into_iter().map(|s| s.to_string()).collect(),
            branch: "main".to_string(),
            last_commit_hash: "abc".to_string(),
            last_commit_date: None,
            last_commit_msg: "test".to_string(),
            file_count: 10,
            size_bytes: 1024,
            readme_hash: String::new(),
            readme_summary: "A test project for search".to_string(),
            scanned_at: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_search_by_query() {
        let conn = setup();
        db::insert_repo(&conn, &sample_repo("repolens", "/tmp/repolens", vec!["Rust"], vec!["clap"])).unwrap();
        db::insert_repo(&conn, &sample_repo("other-project", "/tmp/other", vec!["Python"], vec!["Django"])).unwrap();

        let results = search_repos(&conn, &SearchParams {
            query: Some("repolens".into()),
            lang: None, framework: None, recent: None, path_filter: None, size_filter: None,
            limit: 10,
        }).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "repolens");
    }

    #[test]
    fn test_search_by_lang() {
        let conn = setup();
        db::insert_repo(&conn, &sample_repo("r1", "/tmp/r1", vec!["Rust"], vec![])).unwrap();
        db::insert_repo(&conn, &sample_repo("r2", "/tmp/r2", vec!["Python"], vec![])).unwrap();

        let results = search_repos(&conn, &SearchParams {
            query: None,
            lang: Some("Rust".into()),
            framework: None, recent: None, path_filter: None, size_filter: None,
            limit: 10,
        }).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "r1");
    }

    #[test]
    fn test_search_no_filter() {
        let conn = setup();
        db::insert_repo(&conn, &sample_repo("r1", "/tmp/r1", vec![], vec![])).unwrap();
        db::insert_repo(&conn, &sample_repo("r2", "/tmp/r2", vec![], vec![])).unwrap();

        let results = search_repos(&conn, &SearchParams {
            query: None, lang: None, framework: None, recent: None, path_filter: None, size_filter: None,
            limit: 10,
        }).unwrap();
        assert_eq!(results.len(), 2);
    }
}
