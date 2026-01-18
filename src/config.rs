use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{PeasError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeasConfig {
    #[serde(default)]
    pub peas: PeasSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeasSettings {
    #[serde(default = "default_path")]
    pub path: String,

    #[serde(default = "default_prefix")]
    pub prefix: String,

    #[serde(default = "default_id_length")]
    pub id_length: usize,

    #[serde(default = "default_status")]
    pub default_status: String,

    #[serde(default = "default_type")]
    pub default_type: String,
}

fn default_path() -> String {
    ".peas".to_string()
}

fn default_prefix() -> String {
    "peas-".to_string()
}

fn default_id_length() -> usize {
    4
}

fn default_status() -> String {
    "todo".to_string()
}

fn default_type() -> String {
    "task".to_string()
}

impl Default for PeasSettings {
    fn default() -> Self {
        Self {
            path: default_path(),
            prefix: default_prefix(),
            id_length: default_id_length(),
            default_status: default_status(),
            default_type: default_type(),
        }
    }
}

impl Default for PeasConfig {
    fn default() -> Self {
        Self {
            peas: PeasSettings::default(),
        }
    }
}

impl PeasConfig {
    pub fn load(start_path: &Path) -> Result<(Self, PathBuf)> {
        let config_path = Self::find_config_file(start_path)?;
        let content = std::fs::read_to_string(&config_path)?;
        let config: PeasConfig = serde_yaml::from_str(&content)?;
        let project_root = config_path.parent().unwrap().to_path_buf();
        Ok((config, project_root))
    }

    pub fn find_config_file(start_path: &Path) -> Result<PathBuf> {
        let mut current = start_path.to_path_buf();
        loop {
            let config_path = current.join(".peas.yml");
            if config_path.exists() {
                return Ok(config_path);
            }
            if !current.pop() {
                return Err(PeasError::NotInitialized);
            }
        }
    }

    pub fn data_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.peas.path)
    }

    pub fn archive_path(&self, project_root: &Path) -> PathBuf {
        self.data_path(project_root).join("archive")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
