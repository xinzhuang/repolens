mod models;
mod db;
mod config;
mod scanner;
mod fingerprint;
mod search;
mod display;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "repolens", about = "A local git repository search engine", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan configured roots, discover repos, build index
    Scan {
        /// Rebuild the entire index from scratch
        #[arg(long)]
        rebuild: bool,
        /// Add a path to scan roots before scanning
        #[arg(long)]
        add_path: Option<String>,
        /// Suppress output
        #[arg(long)]
        quiet: bool,
    },
    /// Search repos by query
    Find {
        /// Search query
        query: String,
        /// Filter by language
        #[arg(long)]
        lang: Option<String>,
        /// Filter by framework
        #[arg(long)]
        framework: Option<String>,
        /// Filter by recency (7d, 1m, 1y)
        #[arg(long)]
        recent: Option<String>,
        /// Filter by size (+N/-N bytes)
        #[arg(long)]
        size: Option<String>,
        /// Filter by path substring
        #[arg(long)]
        path: Option<String>,
        /// Max results
        #[arg(long, default_value = "20")]
        limit: i64,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List all repos
    List {
        /// Filter by language
        #[arg(long)]
        lang: Option<String>,
        /// Filter by framework
        #[arg(long)]
        framework: Option<String>,
        /// Sort order (recent, name, size)
        #[arg(long, default_value = "recent")]
        sort: String,
        /// Group results (lang, path)
        #[arg(long)]
        group_by: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i64,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show details for a repo
    Show {
        /// Repo name or path substring
        repo: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Manage configuration
    Config {
        /// Add a scan root path
        #[arg(long)]
        add_path: Option<String>,
        /// Remove a scan root path
        #[arg(long)]
        remove_path: Option<String>,
        /// Set a config key=value
        #[arg(long)]
        set: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let cfg = config::load_config();

    match cli.command {
        Commands::Config { add_path, remove_path, set } => {
            cmd_config(cfg, add_path, remove_path, set);
        }
        Commands::Scan { rebuild, add_path, quiet } => {
            cmd_scan(cfg, rebuild, add_path, quiet);
        }
        Commands::Find { query, lang, framework, recent, size, path, limit, json } => {
            cmd_find(cfg, query, lang, framework, recent, size, path, limit, json);
        }
        Commands::List { lang, framework, sort, group_by, limit, json } => {
            cmd_list(cfg, lang, framework, sort, group_by, limit, json);
        }
        Commands::Show { repo, json } => {
            cmd_show(cfg, repo, json);
        }
    }
}

fn get_db_conn(cfg: &models::Config) -> rusqlite::Connection {
    let conn = db::create_connection(&cfg.db_path)
        .expect("Failed to open database");
    db::init_schema(&conn).expect("Failed to initialize schema");
    conn
}

fn cmd_config(mut cfg: models::Config, add_path: Option<String>, remove_path: Option<String>, set: Option<String>) {
    let mut changed = false;

    if let Some(path) = add_path {
        if config::add_scan_root(&mut cfg, &path) {
            println!("Added scan root: {}", path);
            changed = true;
        } else {
            println!("Scan root already exists: {}", path);
        }
    }

    if let Some(path) = remove_path {
        if config::remove_scan_root(&mut cfg, &path) {
            println!("Removed scan root: {}", path);
            changed = true;
        } else {
            println!("Scan root not found: {}", path);
        }
    }

    if let Some(kv) = set {
        let parts: Vec<&str> = kv.splitn(2, '=').collect();
        if parts.len() == 2 {
            match parts[0] {
                "db_path" => { cfg.db_path = parts[1].to_string(); changed = true; }
                _ => eprintln!("Unknown config key: {}", parts[0]),
            }
        } else {
            eprintln!("Invalid format. Use: --set key=value");
        }
    }

    if changed {
        config::save_config(&cfg).expect("Failed to save config");
    }

    // Always show current config
    println!("Scan roots:");
    for root in &cfg.scan_roots {
        println!("  {}", root);
    }
    println!("Database: {}", cfg.db_path);
}

fn cmd_scan(mut cfg: models::Config, rebuild: bool, add_path: Option<String>, quiet: bool) {
    if let Some(path) = add_path {
        config::add_scan_root(&mut cfg, &path);
        config::save_config(&cfg).expect("Failed to save config");
    }

    if cfg.scan_roots.is_empty() {
        eprintln!("No scan roots configured. Use: repolens config --add-path <path>");
        std::process::exit(1);
    }

    let conn = get_db_conn(&cfg);

    if rebuild {
        db::delete_all_repos(&conn).expect("Failed to clear repos");
        if !quiet { println!("Cleared existing index."); }
    }

    let paths = scanner::discover_repos(&cfg.scan_roots);
    let mut result = models::ScanResult {
        total_found: paths.len(),
        new: 0,
        updated: 0,
        unchanged: 0,
        errors: Vec::new(),
    };

    for repo_path in &paths {
        match scanner::extract_repo_info(repo_path) {
            Ok(mut repo) => {
                let existing = db::get_repo_by_path(&conn, &repo.path).ok().flatten();
                fingerprint::enrich_repo(&mut repo);

                if existing.is_none() {
                    result.new += 1;
                    if !quiet { println!("  [NEW] {}", repo.name); }
                } else {
                    result.updated += 1;
                    if !quiet { println!("  [UPD] {}", repo.name); }
                }

                if let Err(e) = db::insert_repo(&conn, &repo) {
                    result.errors.push(format!("{}: {}", repo.name, e));
                }
            }
            Err(e) => {
                result.errors.push(format!("{}: {}", repo_path.display(), e));
            }
        }
    }

    // Store scan roots in DB too
    for root in &cfg.scan_roots {
        db::add_scan_root(&conn, root).ok();
    }

    println!("{}", display::format_scan_result(&result, quiet));
}

fn cmd_find(cfg: models::Config, query: String, lang: Option<String>, framework: Option<String>,
            recent: Option<String>, size: Option<String>, path: Option<String>, limit: i64, json: bool) {
    let conn = get_db_conn(&cfg);
    let params = search::SearchParams {
        query: Some(query),
        lang,
        framework,
        recent,
        size_filter: size,
        path_filter: path,
        limit,
    };

    match search::search_repos(&conn, &params) {
        Ok(repos) => {
            if json {
                println!("{}", display::format_repos_json(&repos));
            } else {
                print!("{}", display::format_repos_table(&repos));
            }
        }
        Err(e) => eprintln!("Search error: {}", e),
    }
}

fn cmd_list(cfg: models::Config, lang: Option<String>, framework: Option<String>,
            sort: String, _group_by: Option<String>, limit: i64, json: bool) {
    let conn = get_db_conn(&cfg);

    if lang.is_some() || framework.is_some() {
        let params = search::SearchParams {
            query: None,
            lang,
            framework,
            recent: None,
            size_filter: None,
            path_filter: None,
            limit,
        };
        match search::search_repos(&conn, &params) {
            Ok(repos) => {
                if json {
                    println!("{}", display::format_repos_json(&repos));
                } else {
                    print!("{}", display::format_repos_table(&repos));
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    } else {
        match db::list_repos(&conn, limit, &sort) {
            Ok(repos) => {
                if json {
                    println!("{}", display::format_repos_json(&repos));
                } else {
                    print!("{}", display::format_repos_table(&repos));
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn cmd_show(cfg: models::Config, repo_query: String, json: bool) {
    let conn = get_db_conn(&cfg);

    // Try exact name match first
    let repo = db::get_repo_by_name(&conn, &repo_query)
        .ok()
        .flatten()
        .or_else(|| {
            // Try path substring match
            let params = search::SearchParams {
                query: Some(repo_query.clone()),
                lang: None, framework: None, recent: None,
                size_filter: None, path_filter: None, limit: 1,
            };
            search::search_repos(&conn, &params).ok().and_then(|r| r.into_iter().next())
        });

    match repo {
        Some(r) => {
            if json {
                println!("{}", display::format_repo_json(&r));
            } else {
                print!("{}", display::format_repo_detail(&r));
            }
        }
        None => eprintln!("Repository '{}' not found.", repo_query),
    }
}
