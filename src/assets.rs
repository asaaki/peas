use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Asset manager for handling ticket attachments
pub struct AssetManager {
    assets_path: PathBuf,
}

impl AssetManager {
    /// Create a new AssetManager with the given assets directory
    pub fn new(project_root: &Path) -> Self {
        Self {
            assets_path: project_root.join(".peas").join("assets"),
        }
    }

    /// Get the assets directory path
    pub fn assets_path(&self) -> &Path {
        &self.assets_path
    }

    /// Get the directory path for a specific ticket's assets
    pub fn ticket_assets_path(&self, ticket_id: &str) -> PathBuf {
        self.assets_path.join(ticket_id)
    }

    /// Initialize assets directory structure
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.assets_path).context("Failed to create assets directory")?;
        Ok(())
    }

    /// Add an asset file to a ticket
    pub fn add_asset(&self, ticket_id: &str, source_path: &Path) -> Result<String> {
        // Create ticket's asset directory if it doesn't exist
        let ticket_dir = self.ticket_assets_path(ticket_id);
        fs::create_dir_all(&ticket_dir).context("Failed to create ticket asset directory")?;

        // Get the filename from the source path
        let filename = source_path
            .file_name()
            .context("Invalid source file path")?
            .to_str()
            .context("Invalid filename")?;

        // Copy the file to the ticket's asset directory
        let dest_path = ticket_dir.join(filename);

        // If file already exists, create a unique name
        let dest_path = if dest_path.exists() {
            self.get_unique_filename(&ticket_dir, filename)?
        } else {
            dest_path
        };

        fs::copy(source_path, &dest_path).context("Failed to copy asset file")?;

        // Return the filename (relative to ticket directory)
        let filename = dest_path
            .file_name()
            .context("Failed to get filename from destination path")?
            .to_str()
            .context("Filename contains invalid UTF-8")?
            .to_string();

        Ok(filename)
    }

    /// List all assets for a ticket
    pub fn list_assets(&self, ticket_id: &str) -> Result<Vec<AssetInfo>> {
        let ticket_dir = self.ticket_assets_path(ticket_id);

        if !ticket_dir.exists() {
            return Ok(Vec::new());
        }

        let mut assets = Vec::new();
        for entry in fs::read_dir(&ticket_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = fs::metadata(&path)?;

                // Skip files with invalid names (shouldn't happen but be safe)
                let filename = match path.file_name().and_then(|n| n.to_str()) {
                    Some(name) => name.to_string(),
                    None => {
                        eprintln!("Warning: Skipping asset with invalid filename: {:?}", path);
                        continue;
                    }
                };

                assets.push(AssetInfo {
                    filename,
                    size: metadata.len(),
                    path: path.clone(),
                });
            }
        }

        assets.sort_by(|a, b| a.filename.cmp(&b.filename));
        Ok(assets)
    }

    /// Remove an asset from a ticket
    pub fn remove_asset(&self, ticket_id: &str, asset_name: &str) -> Result<()> {
        let asset_path = self.ticket_assets_path(ticket_id).join(asset_name);

        if !asset_path.exists() {
            anyhow::bail!("Asset not found: {}", asset_name);
        }

        fs::remove_file(&asset_path).context("Failed to remove asset file")?;

        // Remove ticket directory if empty
        let ticket_dir = self.ticket_assets_path(ticket_id);
        if let Ok(mut entries) = fs::read_dir(&ticket_dir) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(&ticket_dir);
            }
        }

        Ok(())
    }

    /// Remove all assets for a ticket and the ticket's asset directory
    /// Returns the number of assets deleted
    pub fn cleanup_ticket_assets(&self, ticket_id: &str) -> Result<usize> {
        let ticket_dir = self.ticket_assets_path(ticket_id);

        if !ticket_dir.exists() {
            return Ok(0);
        }

        // Count assets before deletion
        let mut count = 0;
        for entry in fs::read_dir(&ticket_dir)? {
            let entry = entry?;
            if entry.path().is_file() {
                count += 1;
            }
        }

        // Remove the entire directory and all its contents
        fs::remove_dir_all(&ticket_dir).context("Failed to remove ticket assets directory")?;

        Ok(count)
    }

    /// Check if a ticket has any assets
    pub fn has_assets(&self, ticket_id: &str) -> bool {
        let ticket_dir = self.ticket_assets_path(ticket_id);

        if !ticket_dir.exists() {
            return false;
        }

        // Check if directory has any files
        if let Ok(mut entries) = fs::read_dir(&ticket_dir) {
            entries.next().is_some()
        } else {
            false
        }
    }

    /// Get the full path to an asset
    pub fn get_asset_path(&self, ticket_id: &str, asset_name: &str) -> PathBuf {
        self.ticket_assets_path(ticket_id).join(asset_name)
    }

    /// Check if an asset exists
    pub fn asset_exists(&self, ticket_id: &str, asset_name: &str) -> bool {
        self.get_asset_path(ticket_id, asset_name).exists()
    }

    /// Generate a unique filename if a file with the same name already exists
    fn get_unique_filename(&self, dir: &Path, filename: &str) -> Result<PathBuf> {
        let path = Path::new(filename);
        let stem = path
            .file_stem()
            .context("Invalid filename")?
            .to_str()
            .context("Invalid filename")?;
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        for i in 1..1000 {
            let new_name = if extension.is_empty() {
                format!("{}-{}", stem, i)
            } else {
                format!("{}-{}.{}", stem, i, extension)
            };

            let new_path = dir.join(&new_name);
            if !new_path.exists() {
                return Ok(new_path);
            }
        }

        anyhow::bail!("Could not generate unique filename for: {}", filename)
    }
}

/// Information about an asset file
#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub filename: String,
    pub size: u64,
    pub path: PathBuf,
}

impl AssetInfo {
    /// Get a human-readable file size string
    pub fn size_string(&self) -> String {
        format_file_size(self.size)
    }

    /// Get the file extension if present
    pub fn extension(&self) -> Option<String> {
        self.path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
    }

    /// Get a simple file type description based on extension
    pub fn file_type(&self) -> &'static str {
        match self.extension().as_deref() {
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") | Some("webp") => {
                "Image"
            }
            Some("pdf") => "PDF",
            Some("txt") | Some("md") | Some("markdown") => "Text",
            Some("json") | Some("yaml") | Some("yml") | Some("toml") => "Data",
            Some("zip") | Some("tar") | Some("gz") | Some("7z") => "Archive",
            Some("mp4") | Some("avi") | Some("mov") | Some("webm") => "Video",
            Some("mp3") | Some("wav") | Some("ogg") => "Audio",
            Some("html") | Some("css") | Some("js") | Some("ts") => "Web",
            Some("rs") | Some("py") | Some("java") | Some("cpp") | Some("c") => "Code",
            Some("doc") | Some("docx") | Some("odt") => "Document",
            Some("xls") | Some("xlsx") | Some("ods") => "Spreadsheet",
            Some("ppt") | Some("pptx") | Some("odp") => "Presentation",
            _ => "File",
        }
    }
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if size == 0 {
        return "0 B".to_string();
    }

    let mut size = size as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_asset_file_type() {
        let asset = AssetInfo {
            filename: "screenshot.png".to_string(),
            size: 1024,
            path: PathBuf::from("screenshot.png"),
        };
        assert_eq!(asset.file_type(), "Image");

        let asset = AssetInfo {
            filename: "spec.pdf".to_string(),
            size: 2048,
            path: PathBuf::from("spec.pdf"),
        };
        assert_eq!(asset.file_type(), "PDF");
    }

    #[test]
    fn test_has_assets() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let manager = AssetManager::new(temp_dir.path());

        // Initially no assets
        assert!(!manager.has_assets("test-123"));

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        // Add asset
        manager.add_asset("test-123", &test_file).unwrap();

        // Now should have assets
        assert!(manager.has_assets("test-123"));
    }

    #[test]
    fn test_cleanup_ticket_assets() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let manager = AssetManager::new(temp_dir.path());

        // Create test files
        let test_file1 = temp_dir.path().join("file1.txt");
        let test_file2 = temp_dir.path().join("file2.txt");
        std::fs::write(&test_file1, "content1").unwrap();
        std::fs::write(&test_file2, "content2").unwrap();

        // Add assets
        manager.add_asset("test-456", &test_file1).unwrap();
        manager.add_asset("test-456", &test_file2).unwrap();

        // Verify assets exist
        assert!(manager.has_assets("test-456"));
        let assets = manager.list_assets("test-456").unwrap();
        assert_eq!(assets.len(), 2);

        // Cleanup assets
        let count = manager.cleanup_ticket_assets("test-456").unwrap();
        assert_eq!(count, 2);

        // Verify assets are gone
        assert!(!manager.has_assets("test-456"));
        assert!(!manager.ticket_assets_path("test-456").exists());
    }

    #[test]
    fn test_cleanup_nonexistent_ticket() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let manager = AssetManager::new(temp_dir.path());

        // Cleanup nonexistent ticket should return 0
        let count = manager.cleanup_ticket_assets("nonexistent").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_cleanup_empty_directory() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let manager = AssetManager::new(temp_dir.path());

        // Create empty directory
        let ticket_dir = manager.ticket_assets_path("test-789");
        std::fs::create_dir_all(&ticket_dir).unwrap();

        // Cleanup should work even with empty directory
        let count = manager.cleanup_ticket_assets("test-789").unwrap();
        assert_eq!(count, 0);
        assert!(!ticket_dir.exists());
    }
}
