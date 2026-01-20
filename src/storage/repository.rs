use super::markdown::{
    FrontmatterFormat, detect_format, parse_markdown, render_markdown_with_format,
};
use crate::{
    config::PeasConfig,
    error::{PeasError, Result},
    model::{Pea, PeaType},
    validation,
};
use slug::slugify;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub struct PeaRepository {
    data_path: PathBuf,
    archive_path: PathBuf,
    prefix: String,
    frontmatter_format: FrontmatterFormat,
}

impl PeaRepository {
    pub fn new(config: &PeasConfig, project_root: &Path) -> Self {
        Self {
            data_path: config.data_path(project_root),
            archive_path: config.archive_path(project_root),
            prefix: config.peas.prefix.clone(),
            frontmatter_format: config.peas.frontmatter_format(),
        }
    }

    pub fn generate_id(&self) -> String {
        const ALPHABET: &[char] = &[
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
            'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z',
        ];
        let random = nanoid::nanoid!(5, ALPHABET);
        format!("{}{}", self.prefix, random)
    }

    pub fn generate_filename(&self, id: &str, title: &str) -> String {
        let slug = slugify(title);
        let slug = if slug.len() > 50 {
            slug[..50].to_string()
        } else {
            slug
        };
        format!("{}--{}.md", id, slug)
    }

    pub fn create(&self, pea: &Pea) -> Result<PathBuf> {
        // Validate input
        validation::validate_id(&pea.id)?;
        validation::validate_title(&pea.title)?;
        validation::validate_body(&pea.body)?;
        for tag in &pea.tags {
            validation::validate_tag(tag)?;
        }

        // Validate relationships
        validation::validate_no_self_parent(&pea.id, &pea.parent)?;
        validation::validate_no_self_blocking(&pea.id, &pea.blocking)?;
        validation::validate_parent_exists(&pea.parent, |id| self.exists(id))?;
        validation::validate_blocking_exist(&pea.blocking, |id| self.exists(id))?;
        validation::validate_no_circular_parent(&pea.id, &pea.parent, |id| {
            self.get(id).ok().and_then(|p| p.parent)
        })?;

        std::fs::create_dir_all(&self.data_path)?;

        let filename = self.generate_filename(&pea.id, &pea.title);
        let file_path = self.data_path.join(&filename);

        if file_path.exists() {
            return Err(PeasError::Storage(format!(
                "File already exists: {}",
                file_path.display()
            )));
        }

        let content = render_markdown_with_format(pea, self.frontmatter_format)?;

        // Atomic write: write to temp file, then rename
        self.atomic_write(&file_path, &content)?;

        Ok(file_path)
    }

    pub fn get(&self, id: &str) -> Result<Pea> {
        let file_path = self.find_file_by_id(id)?;
        let content = std::fs::read_to_string(&file_path)?;
        parse_markdown(&content)
    }

    /// Check if a pea exists by ID
    pub fn exists(&self, id: &str) -> bool {
        self.find_file_by_id(id).is_ok()
    }

    pub fn update(&self, pea: &mut Pea) -> Result<PathBuf> {
        // Validate input
        validation::validate_title(&pea.title)?;
        validation::validate_body(&pea.body)?;
        for tag in &pea.tags {
            validation::validate_tag(tag)?;
        }

        // Validate relationships
        validation::validate_no_self_parent(&pea.id, &pea.parent)?;
        validation::validate_no_self_blocking(&pea.id, &pea.blocking)?;
        validation::validate_parent_exists(&pea.parent, |id| self.exists(id))?;
        validation::validate_blocking_exist(&pea.blocking, |id| self.exists(id))?;
        validation::validate_no_circular_parent(&pea.id, &pea.parent, |id| {
            self.get(id).ok().and_then(|p| p.parent)
        })?;

        let old_path = self.find_file_by_id(&pea.id)?;

        // Concurrent edit detection: check if file was modified since we loaded it
        // This prevents one TUI instance from clobbering another's changes
        // IMPORTANT: This check must happen BEFORE we call touch(), so we still have
        // the original timestamp that was loaded from disk
        let current_pea = self.get(&pea.id)?;
        if current_pea.updated != pea.updated {
            return Err(PeasError::Storage(format!(
                "Concurrent modification detected for pea '{}'. The file was modified by another process.\nYour version was updated at: {}\nCurrent version was updated at: {}\nPlease reload and try again.",
                pea.id, pea.updated, current_pea.updated
            )));
        }

        // Now that we've verified no concurrent edits, update the timestamp
        pea.touch();

        let new_filename = self.generate_filename(&pea.id, &pea.title);
        let new_path = self.data_path.join(&new_filename);

        // Preserve original frontmatter format
        let original_content = std::fs::read_to_string(&old_path)?;
        let format = detect_format(&original_content).unwrap_or(self.frontmatter_format);
        let content = render_markdown_with_format(pea, format)?;

        // Atomic write: write to new file first, then remove old
        self.atomic_write(&new_path, &content)?;

        // Only remove old file if it's different from new (title changed)
        if old_path != new_path {
            std::fs::remove_file(&old_path)?;
        }

        Ok(new_path)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let file_path = self.find_file_by_id(id)?;
        std::fs::remove_file(&file_path)?;
        Ok(())
    }

    pub fn archive(&self, id: &str) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.archive_path)?;

        let old_path = self.find_file_by_id(id)?;
        let filename = old_path
            .file_name()
            .ok_or_else(|| PeasError::Storage("Path has no filename".to_string()))?
            .to_string_lossy()
            .to_string();
        let new_path = self.archive_path.join(&filename);

        std::fs::rename(&old_path, &new_path)?;
        Ok(new_path)
    }

    pub fn list(&self) -> Result<Vec<Pea>> {
        self.list_in_path(&self.data_path)
    }

    pub fn list_archived(&self) -> Result<Vec<Pea>> {
        if !self.archive_path.exists() {
            return Ok(Vec::new());
        }
        self.list_in_path(&self.archive_path)
    }

    fn list_in_path(&self, path: &Path) -> Result<Vec<Pea>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut peas = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                let Some(filename) = path.file_name() else {
                    continue;
                };
                let filename = filename.to_string_lossy();
                if filename.starts_with(&self.prefix) {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => match parse_markdown(&content) {
                            Ok(pea) => peas.push(pea),
                            Err(e) => {
                                eprintln!("Warning: Failed to parse {}: {}", path.display(), e)
                            }
                        },
                        Err(e) => eprintln!("Warning: Failed to read {}: {}", path.display(), e),
                    }
                }
            }
        }

        peas.sort_by(|a, b| a.created.cmp(&b.created));
        Ok(peas)
    }

    pub fn find_file_by_id(&self, id: &str) -> Result<PathBuf> {
        let search_id = if id.starts_with(&self.prefix) {
            id.to_string()
        } else {
            format!("{}{}", self.prefix, id)
        };

        if self.data_path.exists() {
            for entry in std::fs::read_dir(&self.data_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    let Some(filename) = path.file_name() else {
                        continue;
                    };
                    let filename = filename.to_string_lossy();
                    if filename.starts_with(&search_id) {
                        return Ok(path);
                    }
                }
            }
        }

        Err(PeasError::NotFound(id.to_string()))
    }

    pub fn find_by_type(&self, pea_type: PeaType) -> Result<Vec<Pea>> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|p| p.pea_type == pea_type)
            .collect())
    }

    pub fn find_children(&self, parent_id: &str) -> Result<Vec<Pea>> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|p| p.parent.as_deref() == Some(parent_id))
            .collect())
    }

    /// Atomically write content to a file using temp file + rename
    /// This ensures we never have a partially written file or lose data on crash
    fn atomic_write(&self, target_path: &Path, content: &str) -> Result<()> {
        // Get the directory for the temp file (same as target for atomic rename)
        let target_dir = target_path
            .parent()
            .ok_or_else(|| PeasError::Storage("Target path has no parent directory".to_string()))?;

        // Create temp file in same directory as target (required for atomic rename)
        let mut temp_file = NamedTempFile::new_in(target_dir)
            .map_err(|e| PeasError::Storage(format!("Failed to create temp file: {}", e)))?;

        // Write content to temp file
        use std::io::Write;
        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| PeasError::Storage(format!("Failed to write to temp file: {}", e)))?;

        // Sync to disk to ensure durability
        temp_file
            .as_file()
            .sync_all()
            .map_err(|e| PeasError::Storage(format!("Failed to sync temp file: {}", e)))?;

        // Atomically rename temp file to target (overwrites if exists)
        // This is atomic on Unix and Windows (when in same directory)
        temp_file
            .persist(target_path)
            .map_err(|e| PeasError::Storage(format!("Failed to persist temp file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PeaStatus, PeaType};
    use tempfile::TempDir;

    fn setup_test_repo() -> (PeaRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = PeasConfig {
            peas: crate::config::PeasSettings {
                path: ".peas".to_string(),
                prefix: "test-".to_string(),
                id_length: 5,
                default_status: "todo".to_string(),
                default_type: "task".to_string(),
                frontmatter: "toml".to_string(),
            },
            tui: crate::config::TuiSettings::default(),
        };
        let repo = PeaRepository::new(&config, temp_dir.path());
        (repo, temp_dir)
    }

    #[test]
    fn test_concurrent_edit_detection_rejects_stale_update() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create a pea
        let mut pea = Pea::new(
            "test-12345".to_string(),
            "Original Title".to_string(),
            PeaType::Task,
        );
        pea.body = "Original body".to_string();
        repo.create(&pea).unwrap();

        // Load the pea (simulating first TUI instance)
        let mut pea1 = repo.get("test-12345").unwrap();

        // Load the same pea (simulating second TUI instance)
        let mut pea2 = repo.get("test-12345").unwrap();

        // First instance modifies and saves
        pea1.title = "Modified by Instance 1".to_string();
        // NOTE: No touch() call - update() handles it internally now
        repo.update(&mut pea1).unwrap();

        // Second instance tries to save with stale timestamp
        pea2.title = "Modified by Instance 2".to_string();
        // NOTE: No touch() call - update() would handle it, but we expect failure first

        // This should fail with concurrent modification error
        let result = repo.update(&mut pea2);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Concurrent modification detected"));
        assert!(err_msg.contains("test-12345"));
        assert!(err_msg.contains("reload and try again"));
    }

    #[test]
    fn test_concurrent_edit_detection_allows_reload() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create a pea
        let mut pea = Pea::new(
            "test-67890".to_string(),
            "Original Title".to_string(),
            PeaType::Task,
        );
        pea.body = "Original body".to_string();
        repo.create(&pea).unwrap();

        // Load the pea (simulating first TUI instance)
        let mut pea1 = repo.get("test-67890").unwrap();

        // Load the same pea (simulating second TUI instance)
        let mut pea2 = repo.get("test-67890").unwrap();

        // First instance modifies and saves
        pea1.title = "Modified by Instance 1".to_string();
        // NOTE: No touch() call - update() handles it internally now
        repo.update(&mut pea1).unwrap();

        // Second instance detects conflict and reloads
        pea2 = repo.get("test-67890").unwrap();

        // Now modify and save with fresh timestamp - should succeed
        pea2.title = "Modified by Instance 2 after reload".to_string();
        // NOTE: No touch() call - update() handles it internally now

        let result = repo.update(&mut pea2);
        assert!(result.is_ok());

        // Verify the final state
        let final_pea = repo.get("test-67890").unwrap();
        assert_eq!(final_pea.title, "Modified by Instance 2 after reload");
    }

    #[test]
    fn test_no_false_positive_on_same_timestamp() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create a pea
        let mut pea = Pea::new(
            "test-abcde".to_string(),
            "Original Title".to_string(),
            PeaType::Task,
        );
        pea.body = "Original body".to_string();
        repo.create(&pea).unwrap();

        // Load and modify
        let mut pea = repo.get("test-abcde").unwrap();
        pea.status = PeaStatus::InProgress;
        // NOTE: No touch() call - update() handles it internally now

        // First update should succeed
        let result = repo.update(&mut pea);
        assert!(result.is_ok());

        // Reload and modify again
        let mut pea = repo.get("test-abcde").unwrap();
        pea.status = PeaStatus::Completed;
        // NOTE: No touch() call - update() handles it internally now

        // Second update should also succeed
        let result = repo.update(&mut pea);
        assert!(result.is_ok());
    }
}
