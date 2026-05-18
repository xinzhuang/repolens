use std::fs;
use std::path::PathBuf;
use crate::models::Config;

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(format!("{}/.config/repolens", home))
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn load_config() -> Config {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    let content = toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(config_path(), content).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

pub fn add_scan_root(config: &mut Config, path: &str) -> bool {
    let abs = std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string());
    if config.scan_roots.contains(&abs) {
        return false;
    }
    config.scan_roots.push(abs);
    true
}

pub fn remove_scan_root(config: &mut Config, path: &str) -> bool {
    let abs = std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string());
    let before = config.scan_roots.len();
    config.scan_roots.retain(|r| r != &abs);
    config.scan_roots.len() != before
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.scan_roots.is_empty());
        assert!(config.db_path.contains("repolens"));
    }

    #[test]
    fn test_config_roundtrip() {
        let dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let test_dir = format!("{}/.config/repolens_test_{}", dir, std::process::id());
        fs::create_dir_all(&test_dir).ok();
        let test_path = PathBuf::from(format!("{}/config.toml", test_dir));

        let mut config = Config::default();
        config.scan_roots.push("/tmp/test".to_string());
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&test_path, &content).unwrap();

        let loaded: Config = toml::from_str(&fs::read_to_string(&test_path).unwrap()).unwrap();
        assert_eq!(loaded.scan_roots, vec!["/tmp/test"]);

        fs::remove_dir_all(&test_dir).ok();
    }
}
