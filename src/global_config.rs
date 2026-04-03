use directories::ProjectDirs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct UpdatesConfig {
    pub enabled: bool,
}

impl Default for UpdatesConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct GlobalPeasConfig {
    pub updates: UpdatesConfig,
}

impl GlobalPeasConfig {
    pub fn load() -> Self {
        let Some(proj_dirs) = ProjectDirs::from("", "", "peas") else {
            return Self::default();
        };
        let config_path = proj_dirs.config_dir().join("config.toml");

        if !config_path.exists() {
            return Self::default();
        }

        let content = match std::fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        match toml::from_str::<GlobalPeasConfig>(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!(
                    "warning: failed to parse global peas config ({}): {}",
                    config_path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    /// Returns the path to the global config file, regardless of whether it exists.
    pub fn config_path() -> Option<std::path::PathBuf> {
        ProjectDirs::from("", "", "peas").map(|d| d.config_dir().join("config.toml"))
    }

    /// Load from a specific TOML string (for testing).
    /// Parse from a TOML string, returning defaults on failure.
    pub fn parse(content: &str) -> Self {
        toml::from_str::<GlobalPeasConfig>(content).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_has_updates_enabled() {
        let config = GlobalPeasConfig::default();
        assert!(config.updates.enabled);
    }

    #[test]
    fn test_load_returns_default_when_no_file() {
        // load() should never panic, even if no config file exists
        let config = GlobalPeasConfig::load();
        assert!(config.updates.enabled);
    }

    #[test]
    fn test_from_str_valid_disabled() {
        let config = GlobalPeasConfig::parse("[updates]\nenabled = false\n");
        assert!(!config.updates.enabled);
    }

    #[test]
    fn test_from_str_valid_enabled() {
        let config = GlobalPeasConfig::parse("[updates]\nenabled = true\n");
        assert!(config.updates.enabled);
    }

    #[test]
    fn test_from_str_empty_returns_default() {
        let config = GlobalPeasConfig::parse("");
        assert!(config.updates.enabled);
    }

    #[test]
    fn test_from_str_malformed_returns_default() {
        let config = GlobalPeasConfig::parse("{{{{not valid toml}}}}");
        assert!(config.updates.enabled);
    }

    #[test]
    fn test_from_str_missing_updates_section() {
        let config = GlobalPeasConfig::parse("[something_else]\nkey = \"value\"\n");
        assert!(config.updates.enabled);
    }

    #[test]
    fn test_config_path_returns_some() {
        // Should return a path on most systems
        let path = GlobalPeasConfig::config_path();
        // We can't assert Some on all CI environments, but at least check it doesn't panic
        if let Some(p) = path {
            assert!(p.to_str().unwrap().contains("peas"));
        }
    }
}
