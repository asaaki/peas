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
}
