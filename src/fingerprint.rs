use std::collections::HashMap;
use std::path::Path;
use crate::models::Repo;

static EXTENSIONS: &[(&str, &str)] = &[
    (".rs", "Rust"), (".go", "Go"), (".py", "Python"), (".js", "JavaScript"),
    (".ts", "TypeScript"), (".tsx", "TypeScript"), (".jsx", "JavaScript"),
    (".java", "Java"), (".kt", "Kotlin"), (".scala", "Scala"),
    (".c", "C"), (".cpp", "C++"), (".h", "C"), (".hpp", "C++"),
    (".cs", "C#"), (".rb", "Ruby"), (".php", "PHP"), (".swift", "Swift"),
    (".m", "Objective-C"), (".sh", "Shell"), (".bash", "Shell"),
    (".lua", "Lua"), (".r", "R"), (".jl", "Julia"), (".ex", "Elixir"),
    (".exs", "Elixir"), (".erl", "Erlang"), (".hs", "Haskell"),
    (".ml", "OCaml"), (".zig", "Zig"), (".nim", "Nim"), (".dart", "Dart"),
    (".vue", "Vue"), (".svelte", "Svelte"), (".html", "HTML"), (".css", "CSS"),
    (".scss", "CSS"), (".less", "CSS"),
];

static FRAMEWORK_FILES: &[(&str, &str)] = &[
    ("package.json", "Node.js"), ("Cargo.toml", "Rust"),
    ("go.mod", "Go"), ("requirements.txt", "Python"),
    ("Pipfile", "Python"), ("pyproject.toml", "Python"),
    ("pom.xml", "Java/Maven"), ("build.gradle", "Java/Gradle"),
    ("Gemfile", "Ruby"), ("composer.json", "PHP"),
    ("Package.swift", "Swift"), ("mix.exs", "Elixir"),
    ("pubspec.yaml", "Flutter"), ("project.clj", "Clojure"),
    ("deps.edn", "Clojure"), ("dune-project", "OCaml"),
    ("cabal.file", "Haskell"), ("stack.yaml", "Haskell"),
];

pub fn detect_languages(path: &Path) -> Vec<String> {
    let mut counts: HashMap<String, i64> = HashMap::new();
    let walker = ignore::WalkBuilder::new(path)
        .hidden(true)
        .git_ignore(true)
        .build();

    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
            let ext_with_dot = format!(".{}", ext);
            for &(pattern, lang) in EXTENSIONS {
                if ext_with_dot == pattern {
                    *counts.entry(lang.to_string()).or_default() += 1;
                    break;
                }
            }
        }
    }

    let mut langs: Vec<(String, i64)> = counts.into_iter().collect();
    langs.sort_by(|a, b| b.1.cmp(&a.1));
    langs.into_iter().take(10).map(|(l, _)| l).collect()
}

pub fn detect_frameworks(path: &Path) -> Vec<String> {
    let mut frameworks = Vec::new();
    for &(file, framework) in FRAMEWORK_FILES {
        if path.join(file).exists() {
            frameworks.push(framework.to_string());
        }
    }

    // Detect specific frameworks from package.json
    let pkg_path = path.join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                let deps = val.get("dependencies");
                let dev_deps = val.get("devDependencies");
                for (key, container) in [("dep", deps), ("dev", dev_deps)] {
                    let _ = key;
                    if let Some(map) = container.and_then(|v| v.as_object()) {
                        if map.contains_key("react") { frameworks.push("React".into()); }
                        if map.contains_key("vue") { frameworks.push("Vue".into()); }
                        if map.contains_key("next") { frameworks.push("Next.js".into()); }
                        if map.contains_key("svelte") { frameworks.push("Svelte".into()); }
                        if map.contains_key("express") { frameworks.push("Express".into()); }
                        if map.contains_key("fastify") { frameworks.push("Fastify".into()); }
                        if map.contains_key("tailwindcss") { frameworks.push("Tailwind CSS".into()); }
                    }
                }
            }
        }
    }

    // Detect from Cargo.toml
    let cargo_path = path.join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            if let Ok(val) = toml::from_str::<toml::Value>(&content) {
                if let Some(deps) = val.get("dependencies").and_then(|v| v.as_table()) {
                    if deps.contains_key("tokio") { frameworks.push("Tokio".into()); }
                    if deps.contains_key("axum") { frameworks.push("Axum".into()); }
                    if deps.contains_key("actix-web") { frameworks.push("Actix".into()); }
                    if deps.contains_key("clap") { frameworks.push("clap".into()); }
                    if deps.contains_key("serde") { frameworks.push("Serde".into()); }
                }
            }
        }
    }

    frameworks.sort();
    frameworks.dedup();
    frameworks
}

pub fn extract_readme_summary(path: &Path) -> (String, String) {
    let readme_names = ["README.md", "README.rst", "README.txt", "README", "readme.md"];
    for name in &readme_names {
        let readme_path = path.join(name);
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let summary: String = content
                    .lines()
                    .filter(|l| !l.starts_with('#') || l.trim().matches('#').count() == l.trim().len())
                    .take_while(|l| !l.trim().is_empty() || summary_lines_so_far(&content) < 3)
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .chars()
                    .take(500)
                    .collect();
                let hash = sha2_hash(&content);
                return (hash, summary);
            }
        }
    }
    (String::new(), String::new())
}

fn summary_lines_so_far(content: &str) -> usize {
    content.lines().take(5).count()
}

fn sha2_hash(content: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn enrich_repo(repo: &mut Repo) {
    let path = Path::new(&repo.path);
    repo.languages = detect_languages(path);
    repo.frameworks = detect_frameworks(path);
    let (hash, summary) = extract_readme_summary(path);
    repo.readme_hash = hash;
    repo.readme_summary = summary;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_languages_repolens() {
        let path = Path::new("/home/frank/Project/myrepos/repolens");
        if !path.exists() { return; }
        let langs = detect_languages(path);
        assert!(langs.contains(&"Rust".to_string()), "Should detect Rust: {:?}", langs);
    }

    #[test]
    fn test_detect_frameworks_repolens() {
        let path = Path::new("/home/frank/Project/myrepos/repolens");
        if !path.exists() { return; }
        let frameworks = detect_frameworks(path);
        assert!(frameworks.contains(&"Rust".to_string()) || frameworks.iter().any(|f| f.contains("clap") || f.contains("Serde")),
            "Should detect Rust frameworks: {:?}", frameworks);
    }

    #[test]
    fn test_extract_readme_summary_repolens() {
        let path = Path::new("/home/frank/Project/myrepos/repolens");
        if !path.exists() { return; }
        let (hash, summary) = extract_readme_summary(path);
        assert!(!hash.is_empty(), "README hash should not be empty");
    }

    #[test]
    fn test_sha2_hash() {
        let hash = sha2_hash("hello");
        assert_eq!(hash.len(), 64);
        assert_ne!(hash, sha2_hash("world"));
    }
}
