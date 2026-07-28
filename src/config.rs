use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub api_url: String,
    pub token: String,
    pub conference_id: String,
    pub conference_title: String,
    #[serde(default)]
    pub name: Option<String>,
}

pub fn config_path() -> Result<PathBuf> {
    default_path()
}

fn default_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("KONF_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    let base = dirs::config_dir().context("Could not determine config directory")?;
    Ok(base.join("konf").join("config.toml"))
}

pub fn load() -> Result<Config> {
    load_from(&default_path()?)
}

pub fn save(config: &Config) -> Result<()> {
    save_to(config, &default_path()?)
}

pub fn delete() -> Result<bool> {
    delete_at(&default_path()?)
}

pub fn exists() -> bool {
    default_path().is_ok_and(|p| p.exists())
}

pub fn load_from(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Could not read config at {}", path.display()))?;
    toml::from_str(&content).context("Invalid config file format")
}

pub fn save_to(config: &Config, path: &Path) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .with_context(|| format!("Could not create config directory {}", dir.display()))?;
    }
    let content = toml::to_string_pretty(config).context("Could not serialize config")?;

    // Write atomically: temp file → fsync → rename
    // This prevents corruption if the process is interrupted mid-write.
    let dir = path
        .parent()
        .context("Config path has no parent directory")?;
    let tmp_path = dir.join(".config.toml.tmp");

    {
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&tmp_path)
                .with_context(|| format!("Could not write config to {}", tmp_path.display()))?;
            file.write_all(content.as_bytes())
                .with_context(|| format!("Could not write config to {}", tmp_path.display()))?;
            file.sync_all()?;
        }
        #[cfg(not(unix))]
        {
            fs::write(&tmp_path, &content)
                .with_context(|| format!("Could not write config to {}", tmp_path.display()))?;
        }
    }

    fs::rename(&tmp_path, path)
        .with_context(|| format!("Could not finalize config at {}", path.display()))?;

    Ok(())
}

pub fn delete_at(path: &Path) -> Result<bool> {
    if path.exists() {
        fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config() -> Config {
        Config {
            api_url: "https://2026.cloudnativedays.no".to_string(),
            token: "test-token-abc123".to_string(),
            conference_id: "2026.cloudnativedays.no".to_string(),
            conference_title: "2026.cloudnativedays.no".to_string(),
            name: Some("Alice".to_string()),
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let cfg = test_config();

        save_to(&cfg, &path).unwrap();
        let loaded = load_from(&path).unwrap();

        assert_eq!(cfg, loaded);
    }

    #[test]
    fn load_nonexistent_file_errors() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nope.toml");

        let result = load_from(&path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Could not read config"),
        );
    }

    #[test]
    fn load_invalid_toml_errors() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad.toml");
        fs::write(&path, "this is not [valid toml").unwrap();

        let result = load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn load_missing_fields_errors() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("partial.toml");
        fs::write(&path, "api_url = \"https://example.com\"\n").unwrap();

        let result = load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn delete_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "").unwrap();

        assert!(delete_at(&path).unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn delete_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nope.toml");

        assert!(!delete_at(&path).unwrap());
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("config.toml");
        let cfg = test_config();

        save_to(&cfg, &path).unwrap();
        assert!(path.exists());

        let loaded = load_from(&path).unwrap();
        assert_eq!(cfg, loaded);
    }

    #[test]
    fn save_overwrites_existing_config() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let cfg1 = test_config();
        save_to(&cfg1, &path).unwrap();

        let cfg2 = Config {
            api_url: "https://other.example.com".to_string(),
            token: "different-token".to_string(),
            conference_id: "other-conf".to_string(),
            conference_title: "Other Conference".to_string(),
            name: None,
        };
        save_to(&cfg2, &path).unwrap();

        let loaded = load_from(&path).unwrap();
        assert_eq!(cfg2, loaded);
        assert_ne!(cfg1, loaded);
    }
}
