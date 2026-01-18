use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeasError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Pea not found: {0}")]
    NotFound(String),

    #[error("Invalid pea ID: {0}")]
    InvalidId(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Project not initialized. Run 'peas init' first.")]
    NotInitialized,

    #[error("Project already initialized at {0}")]
    AlreadyInitialized(String),
}

pub type Result<T> = std::result::Result<T, PeasError>;
