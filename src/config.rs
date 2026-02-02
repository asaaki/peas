use crate::error::{PeasError, Result};
use crate::storage::FrontmatterFormat;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// ID generation mode for tickets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdMode {
    /// Random alphanumeric ID using nanoid (default)
    #[default]
    Random,
    /// Sequential numeric ID (00001, 00002, etc.)
    Sequential,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeasConfig {
    #[serde(default)]
    pub peas: PeasSettings,

    #[serde(default)]
    pub tui: TuiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeasSettings {
    #[serde(default = "default_path")]
    pub path: String,

    #[serde(default = "default_prefix")]
    pub prefix: String,

    #[serde(default = "default_id_length")]
    pub id_length: usize,

    #[serde(default)]
    pub id_mode: IdMode,

    #[serde(default = "default_status")]
    pub default_status: String,

    #[serde(default = "default_type")]
    pub default_type: String,

    #[serde(default = "default_frontmatter")]
    pub frontmatter: String,
}

fn default_path() -> String {
    ".peas".to_string()
}

fn default_prefix() -> String {
    "peas-".to_string()
}

fn default_id_length() -> usize {
    5
}

fn default_status() -> String {
    "todo".to_string()
}

fn default_type() -> String {
    "task".to_string()
}

fn default_frontmatter() -> String {
    "toml".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiSettings {
    #[serde(default = "default_use_type_emojis")]
    pub use_type_emojis: bool,
}

fn default_use_type_emojis() -> bool {
    false
}

impl Default for TuiSettings {
    fn default() -> Self {
        Self {
            use_type_emojis: default_use_type_emojis(),
        }
    }
}

impl Default for PeasSettings {
    fn default() -> Self {
        Self {
            path: default_path(),
            prefix: default_prefix(),
            id_length: default_id_length(),
            id_mode: IdMode::default(),
            default_status: default_status(),
            default_type: default_type(),
            frontmatter: default_frontmatter(),
        }
    }
}

impl PeasSettings {
    pub fn frontmatter_format(&self) -> FrontmatterFormat {
        match self.frontmatter.as_str() {
            "toml" => FrontmatterFormat::Toml,
            _ => FrontmatterFormat::Yaml,
        }
    }
}

impl PeasConfig {
    pub fn load(start_path: &Path) -> Result<(Self, PathBuf)> {
        let config_path = Self::find_config_file(start_path)?;
        let content = std::fs::read_to_string(&config_path)?;

        // Determine format based on file extension
        let config: PeasConfig = if config_path.extension().and_then(|s| s.to_str()) == Some("toml")
        {
            toml::from_str(&content)?
        } else if config_path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)?
        } else {
            // YAML for .yml/.yaml or unknown
            serde_yaml::from_str(&content)?
        };

        let project_root = config_path
            .parent()
            .ok_or_else(|| PeasError::Config("Config file has no parent directory".to_string()))?
            .to_path_buf();
        Ok((config, project_root))
    }

    pub fn find_config_file(start_path: &Path) -> Result<PathBuf> {
        let mut current = start_path.to_path_buf();
        loop {
            // Try TOML first (preferred), then YAML, then JSON
            for filename in [".peas.toml", ".peas.yml", ".peas.yaml", ".peas.json"] {
                let config_path = current.join(filename);
                if config_path.exists() {
                    return Ok(config_path);
                }
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
        // Determine format based on file extension, default to TOML
        let content = if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "toml" => toml::to_string_pretty(self)?,
                "json" => serde_json::to_string_pretty(self)?,
                "yml" | "yaml" => serde_yaml::to_string(self)?,
                _ => toml::to_string_pretty(self)?, // Default to TOML
            }
        } else {
            toml::to_string_pretty(self)? // Default to TOML
        };
        std::fs::write(path, content)?;
        Ok(())
    }
}
