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
    /// Supports multiple undo levels by maintaining a stack
    pub fn record(&self, op: UndoOperation) -> Result<()> {
        let mut stack = self.get_stack()?;

        // Limit stack size to prevent unbounded growth (keep last 50 operations)
        const MAX_UNDO_LEVELS: usize = 50;
        if stack.len() >= MAX_UNDO_LEVELS {
            stack.remove(0); // Remove oldest operation
        }

        stack.push(op);
        self.save_stack(&stack)?;
        Ok(())
    }

    /// Get the entire undo stack
    fn get_stack(&self) -> Result<Vec<UndoOperation>> {
        if !self.undo_file.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&self.undo_file)?;
        let stack: Vec<UndoOperation> = serde_json::from_str(&content)?;
        Ok(stack)
    }

    /// Save the undo stack to disk
    fn save_stack(&self, stack: &[UndoOperation]) -> Result<()> {
        let content = serde_json::to_string_pretty(&stack)?;
        std::fs::write(&self.undo_file, content)?;
        Ok(())
    }

    /// Get the last recorded operation
    pub fn last_operation(&self) -> Result<Option<UndoOperation>> {
        let stack = self.get_stack()?;
        Ok(stack.last().cloned())
    }

    /// Get the number of operations that can be undone
    pub fn undo_count(&self) -> usize {
        self.get_stack().map(|s| s.len()).unwrap_or(0)
    }

    /// Get descriptions of all operations in the undo stack
    pub fn undo_stack_descriptions(&self) -> Vec<String> {
        self.get_stack()
            .unwrap_or_default()
            .iter()
            .map(|op| op.description())
            .collect()
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
        let mut stack = self.get_stack()?;

        let op = stack
            .pop()
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

        // Save the updated stack (with the operation removed)
        if stack.is_empty() {
            self.clear()?;
        } else {
            self.save_stack(&stack)?;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_multi_level_undo() {
        let temp_dir = TempDir::new().unwrap();
        let undo_manager = UndoManager::new(temp_dir.path());

        // Record multiple operations
        let file1 = temp_dir.path().join("test1.txt");
        let file2 = temp_dir.path().join("test2.txt");
        let file3 = temp_dir.path().join("test3.txt");

        std::fs::write(&file1, "content1").unwrap();
        std::fs::write(&file2, "content2").unwrap();
        std::fs::write(&file3, "content3").unwrap();

        undo_manager
            .record(UndoOperation::Create {
                id: "id1".to_string(),
                file_path: file1.clone(),
            })
            .unwrap();

        undo_manager
            .record(UndoOperation::Create {
                id: "id2".to_string(),
                file_path: file2.clone(),
            })
            .unwrap();

        undo_manager
            .record(UndoOperation::Create {
                id: "id3".to_string(),
                file_path: file3.clone(),
            })
            .unwrap();

        // Should have 3 operations
        assert_eq!(undo_manager.undo_count(), 3);

        // Undo third operation
        let result = undo_manager.undo().unwrap();
        assert!(result.contains("id3"));
        assert!(!file3.exists());
        assert_eq!(undo_manager.undo_count(), 2);

        // Undo second operation
        let result = undo_manager.undo().unwrap();
        assert!(result.contains("id2"));
        assert!(!file2.exists());
        assert_eq!(undo_manager.undo_count(), 1);

        // Undo first operation
        let result = undo_manager.undo().unwrap();
        assert!(result.contains("id1"));
        assert!(!file1.exists());
        assert_eq!(undo_manager.undo_count(), 0);

        // No more undos available
        assert!(undo_manager.undo().is_err());
    }

    #[test]
    fn test_undo_stack_descriptions() {
        let temp_dir = TempDir::new().unwrap();
        let undo_manager = UndoManager::new(temp_dir.path());

        let file1 = temp_dir.path().join("test1.txt");
        let file2 = temp_dir.path().join("test2.txt");

        std::fs::write(&file1, "content1").unwrap();
        std::fs::write(&file2, "content2").unwrap();

        undo_manager
            .record(UndoOperation::Create {
                id: "peas-abc".to_string(),
                file_path: file1,
            })
            .unwrap();

        undo_manager
            .record(UndoOperation::Update {
                id: "peas-def".to_string(),
                file_path: file2,
                previous_content: "old content".to_string(),
            })
            .unwrap();

        let descriptions = undo_manager.undo_stack_descriptions();
        assert_eq!(descriptions.len(), 2);
        assert_eq!(descriptions[0], "Create peas-abc");
        assert_eq!(descriptions[1], "Update peas-def");
    }

    #[test]
    fn test_undo_stack_limit() {
        let temp_dir = TempDir::new().unwrap();
        let undo_manager = UndoManager::new(temp_dir.path());

        // Record 51 operations (exceeds the 50 limit)
        for i in 0..51 {
            let file = temp_dir.path().join(format!("test{}.txt", i));
            std::fs::write(&file, format!("content{}", i)).unwrap();
            undo_manager
                .record(UndoOperation::Create {
                    id: format!("id{}", i),
                    file_path: file,
                })
                .unwrap();
        }

        // Should only have 50 (oldest removed)
        assert_eq!(undo_manager.undo_count(), 50);

        // Oldest operation (id0) should be gone
        let descriptions = undo_manager.undo_stack_descriptions();
        assert!(!descriptions[0].contains("id0"));
        assert!(descriptions[0].contains("id1")); // First one should be id1
    }

    #[test]
    fn test_undo_update_operation() {
        let temp_dir = TempDir::new().unwrap();
        let undo_manager = UndoManager::new(temp_dir.path());

        let file = temp_dir.path().join("test.txt");
        std::fs::write(&file, "original content").unwrap();

        // Record update with previous content
        undo_manager
            .record(UndoOperation::Update {
                id: "test-id".to_string(),
                file_path: file.clone(),
                previous_content: "original content".to_string(),
            })
            .unwrap();

        // Modify file
        std::fs::write(&file, "new content").unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "new content");

        // Undo should restore original content
        undo_manager.undo().unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "original content");
    }

    #[test]
    fn test_empty_undo_stack() {
        let temp_dir = TempDir::new().unwrap();
        let undo_manager = UndoManager::new(temp_dir.path());

        assert_eq!(undo_manager.undo_count(), 0);
        assert_eq!(undo_manager.undo_stack_descriptions().len(), 0);
        assert!(undo_manager.last_operation().unwrap().is_none());
        assert!(undo_manager.undo().is_err());
    }
}
