use crate::error::{PeasError, Result};
use crate::storage::FrontmatterFormat;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// URL to the JSON Schema for peas configuration files
pub const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/asaaki/peas/refs/heads/main/schemas/peas.json";

/// Canonical data directory name
pub const DATA_DIR: &str = ".peas";

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
    /// Deprecated: data directory is now always `.peas/`
    /// This field is ignored but kept for backwards compatibility.
    #[serde(default, skip_serializing)]
    pub path: Option<String>,

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
            path: None,
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
        let (config_path, is_legacy) = Self::find_config_file(start_path)?;
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

        // Print deprecation warnings
        if is_legacy {
            eprintln!(
                "{}: Config file location `{}` is deprecated. Please move to `{}/config.toml`",
                "warning".yellow().bold(),
                config_path.display(),
                DATA_DIR
            );
        }
        if config.peas.path.is_some() {
            eprintln!(
                "{}: The `peas.path` config option is deprecated and ignored. Data is always stored in `{}/`",
                "warning".yellow().bold(),
                DATA_DIR
            );
        }

        // Project root is parent of .peas/ for new location, or parent of config file for legacy
        let project_root = if is_legacy {
            config_path
                .parent()
                .ok_or_else(|| {
                    PeasError::Config("Config file has no parent directory".to_string())
                })?
                .to_path_buf()
        } else {
            // Config is at .peas/config.toml, so project root is grandparent
            config_path
                .parent() // .peas/
                .and_then(|p| p.parent()) // project root
                .ok_or_else(|| {
                    PeasError::Config("Config file has no parent directory".to_string())
                })?
                .to_path_buf()
        };
        Ok((config, project_root))
    }

    /// Find config file, returns (path, is_legacy)
    pub fn find_config_file(start_path: &Path) -> Result<(PathBuf, bool)> {
        let mut current = start_path.to_path_buf();
        loop {
            // Try new canonical location first: .peas/config.{toml,yml,yaml,json}
            let peas_dir = current.join(DATA_DIR);
            if peas_dir.is_dir() {
                for filename in ["config.toml", "config.yml", "config.yaml", "config.json"] {
                    let config_path = peas_dir.join(filename);
                    if config_path.exists() {
                        return Ok((config_path, false));
                    }
                }
            }

            // Fall back to legacy locations: .peas.{toml,yml,yaml,json}
            for filename in [".peas.toml", ".peas.yml", ".peas.yaml", ".peas.json"] {
                let config_path = current.join(filename);
                if config_path.exists() {
                    return Ok((config_path, true));
                }
            }

            if !current.pop() {
                return Err(PeasError::NotInitialized);
            }
        }
    }

    pub fn data_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(DATA_DIR)
    }

    pub fn archive_path(&self, project_root: &Path) -> PathBuf {
        self.data_path(project_root).join("archive")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        // Determine format based on file extension, default to TOML
        let content = if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "toml" => {
                    let toml_content = toml::to_string_pretty(self)?;
                    format!("#:schema {}\n\n{}", SCHEMA_URL, toml_content)
                }
                "json" => {
                    // Add $schema property to JSON output
                    let mut json_value = serde_json::to_value(self)?;
                    if let serde_json::Value::Object(ref mut map) = json_value {
                        map.insert(
                            "$schema".to_string(),
                            serde_json::Value::String(SCHEMA_URL.to_string()),
                        );
                    }
                    serde_json::to_string_pretty(&json_value)?
                }
                "yml" | "yaml" => {
                    let yaml_content = serde_yaml::to_string(self)?;
                    format!(
                        "# yaml-language-server: $schema={}\n\n{}",
                        SCHEMA_URL, yaml_content
                    )
                }
                _ => {
                    let toml_content = toml::to_string_pretty(self)?;
                    format!("#:schema {}\n\n{}", SCHEMA_URL, toml_content)
                }
            }
        } else {
            let toml_content = toml::to_string_pretty(self)?;
            format!("#:schema {}\n\n{}", SCHEMA_URL, toml_content)
        };
        std::fs::write(path, content)?;
        Ok(())
    }
}
