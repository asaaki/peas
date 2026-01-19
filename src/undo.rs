use crate::error::{PeasError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Types of operations that can be undone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoOperation {
    /// Created a new pea - undo by deleting
    Create { id: String, file_path: PathBuf },
    /// Updated a pea - undo by restoring previous content
    Update {
        id: String,
        file_path: PathBuf,
        previous_content: String,
    },
    /// Deleted a pea - undo by restoring the file
    Delete {
        id: String,
        file_path: PathBuf,
        previous_content: String,
    },
    /// Archived a pea - undo by moving back
    Archive {
        id: String,
        original_path: PathBuf,
        archive_path: PathBuf,
    },
}

impl UndoOperation {
    pub fn description(&self) -> String {
        match self {
            UndoOperation::Create { id, .. } => format!("Create {}", id),
            UndoOperation::Update { id, .. } => format!("Update {}", id),
            UndoOperation::Delete { id, .. } => format!("Delete {}", id),
            UndoOperation::Archive { id, .. } => format!("Archive {}", id),
        }
    }

    pub fn id(&self) -> &str {
        match self {
            UndoOperation::Create { id, .. } => id,
            UndoOperation::Update { id, .. } => id,
            UndoOperation::Delete { id, .. } => id,
            UndoOperation::Archive { id, .. } => id,
        }
    }
}

/// Manages undo state for peas operations
pub struct UndoManager {
    undo_file: PathBuf,
}

impl UndoManager {
    pub fn new(data_path: &Path) -> Self {
        Self {
            undo_file: data_path.join(".undo"),
        }
    }

    /// Record an operation for potential undo
    pub fn record(&self, op: UndoOperation) -> Result<()> {
        let content = serde_json::to_string_pretty(&op)?;
        std::fs::write(&self.undo_file, content)?;
        Ok(())
    }

    /// Get the last recorded operation
    pub fn last_operation(&self) -> Result<Option<UndoOperation>> {
        if !self.undo_file.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&self.undo_file)?;
        let op: UndoOperation = serde_json::from_str(&content)?;
        Ok(Some(op))
    }

    /// Clear the undo state
    pub fn clear(&self) -> Result<()> {
        if self.undo_file.exists() {
            std::fs::remove_file(&self.undo_file)?;
        }
        Ok(())
    }

    /// Execute undo of the last operation
    pub fn undo(&self) -> Result<String> {
        let op = self
            .last_operation()?
            .ok_or_else(|| PeasError::Storage("Nothing to undo".to_string()))?;

        let description = op.description();

        match op {
            UndoOperation::Create { file_path, .. } => {
                // Undo create by deleting the file
                if file_path.exists() {
                    std::fs::remove_file(&file_path)?;
                }
            }
            UndoOperation::Update {
                file_path,
                previous_content,
                ..
            } => {
                // Undo update by restoring previous content
                std::fs::write(&file_path, previous_content)?;
            }
            UndoOperation::Delete {
                file_path,
                previous_content,
                ..
            } => {
                // Undo delete by recreating the file
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&file_path, previous_content)?;
            }
            UndoOperation::Archive {
                original_path,
                archive_path,
                ..
            } => {
                // Undo archive by moving back
                if archive_path.exists() {
                    std::fs::rename(&archive_path, &original_path)?;
                }
            }
        }

        // Clear the undo state after successful undo
        self.clear()?;

        Ok(format!("Undone: {}", description))
    }
}

/// Helper to record a create operation
pub fn record_create(undo_manager: &UndoManager, id: &str, file_path: &Path) -> Result<()> {
    undo_manager.record(UndoOperation::Create {
        id: id.to_string(),
        file_path: file_path.to_path_buf(),
    })
}

/// Helper to record an update operation (call before the update)
pub fn record_update(undo_manager: &UndoManager, id: &str, file_path: &Path) -> Result<()> {
    let previous_content = std::fs::read_to_string(file_path)?;
    undo_manager.record(UndoOperation::Update {
        id: id.to_string(),
        file_path: file_path.to_path_buf(),
        previous_content,
    })
}

/// Helper to record a delete operation (call before the delete)
pub fn record_delete(undo_manager: &UndoManager, id: &str, file_path: &Path) -> Result<()> {
    let previous_content = std::fs::read_to_string(file_path)?;
    undo_manager.record(UndoOperation::Delete {
        id: id.to_string(),
        file_path: file_path.to_path_buf(),
        previous_content,
    })
}

/// Helper to record an archive operation
pub fn record_archive(
    undo_manager: &UndoManager,
    id: &str,
    original_path: &Path,
    archive_path: &Path,
) -> Result<()> {
    undo_manager.record(UndoOperation::Archive {
        id: id.to_string(),
        original_path: original_path.to_path_buf(),
        archive_path: archive_path.to_path_buf(),
    })
}
