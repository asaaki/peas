use super::markdown::{
    FrontmatterFormat, detect_format, parse_markdown, render_markdown_with_format,
};
use crate::{
    config::{IdMode, PeasConfig},
    error::{PeasError, Result},
    model::{Pea, PeaType},
    validation,
};
use slug::slugify;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// In-memory cache for pea data
#[derive(Default)]
struct PeaCache {
    /// Cached list of all peas (None = not cached)
    list: Option<Vec<Pea>>,
    /// Cached individual peas by ID for O(1) lookups
    by_id: HashMap<String, Pea>,
}

impl PeaCache {
    fn new() -> Self {
        Self::default()
    }

    fn invalidate(&mut self) {
        self.list = None;
        self.by_id.clear();
    }

    fn set_list(&mut self, peas: Vec<Pea>) {
        // Update both list and by_id map
        self.by_id = peas.iter().map(|p| (p.id.clone(), p.clone())).collect();
        self.list = Some(peas);
    }

    fn get_list(&self) -> Option<&Vec<Pea>> {
        self.list.as_ref()
    }

    fn get_by_id(&self, id: &str) -> Option<&Pea> {
        self.by_id.get(id)
    }

    fn update_pea(&mut self, pea: &Pea) {
        self.by_id.insert(pea.id.clone(), pea.clone());
        // Update in list if present, otherwise invalidate list
        if let Some(ref mut list) = self.list {
            if let Some(pos) = list.iter().position(|p| p.id == pea.id) {
                list[pos] = pea.clone();
            } else {
                // Pea not in list (new pea) - invalidate list cache
                self.list = None;
            }
        }
    }

    fn remove_pea(&mut self, id: &str) {
        self.by_id.remove(id);
        if let Some(ref mut list) = self.list {
            list.retain(|p| p.id != id);
        }
    }
}

pub struct PeaRepository {
    data_path: PathBuf,
    archive_path: PathBuf,
    prefix: String,
    id_length: usize,
    id_mode: IdMode,
    frontmatter_format: FrontmatterFormat,
    cache: RefCell<PeaCache>,
}

impl PeaRepository {
    pub fn new(config: &PeasConfig, project_root: &Path) -> Self {
        Self {
            data_path: config.data_path(project_root),
            archive_path: config.archive_path(project_root),
            prefix: config.peas.prefix.clone(),
            id_length: config.peas.id_length,
            id_mode: config.peas.id_mode,
            frontmatter_format: config.peas.frontmatter_format(),
            cache: RefCell::new(PeaCache::new()),
        }
    }

    /// Invalidate the cache (call after external file changes)
    pub fn invalidate_cache(&self) {
        self.cache.borrow_mut().invalidate();
    }

    pub fn generate_id(&self) -> Result<String> {
        let suffix = match self.id_mode {
            IdMode::Random => self.generate_random_suffix(),
            IdMode::Sequential => self.generate_sequential_suffix()?,
        };
        Ok(format!("{}{}", self.prefix, suffix))
    }

    fn generate_random_suffix(&self) -> String {
        const ALPHABET: [char; 36] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
            'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z',
        ];
        nanoid::format(nanoid::rngs::default, &ALPHABET, self.id_length)
    }

    fn generate_sequential_suffix(&self) -> Result<String> {
        let counter_path = self.data_path.join(".id");

        // Ensure data directory exists
        std::fs::create_dir_all(&self.data_path)?;

        // Read current counter or start at 0
        let current = if counter_path.exists() {
            let content = std::fs::read_to_string(&counter_path)?;
            content.trim().parse::<u64>().unwrap_or(0)
        } else {
            0
        };

        // Increment counter
        let next = current + 1;

        // Write new counter value atomically
        self.atomic_write(&counter_path, &next.to_string())?;

        // Format with leading zeros based on id_length
        Ok(format!("{:0>width$}", next, width = self.id_length))
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
        tracing::info!(id = %pea.id, title = %pea.title, "Creating pea");

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

        // Update cache with new pea
        self.cache.borrow_mut().update_pea(pea);

        Ok(file_path)
    }

    pub fn get(&self, id: &str) -> Result<Pea> {
        // Check cache first for O(1) lookup
        let cache = self.cache.borrow();
        if let Some(pea) = cache.get_by_id(id) {
            return Ok(pea.clone());
        }
        drop(cache); // Release borrow before disk read

        // Cache miss - load from disk
        let file_path = self.find_file_by_id(id)?;
        let content = std::fs::read_to_string(&file_path)?;
        let pea = parse_markdown(&content)?;

        // Update cache with loaded pea
        self.cache.borrow_mut().update_pea(&pea);

        Ok(pea)
    }

    /// Check if a pea exists by ID
    pub fn exists(&self, id: &str) -> bool {
        // Check cache first for O(1) lookup
        let cache = self.cache.borrow();
        if cache.get_by_id(id).is_some() {
            return true;
        }
        drop(cache);

        // Cache miss - check disk
        self.find_file_by_id(id).is_ok()
    }

    pub fn update(&self, pea: &mut Pea) -> Result<PathBuf> {
        tracing::info!(id = %pea.id, title = %pea.title, "Updating pea");

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

        // Update cache with modified pea
        self.cache.borrow_mut().update_pea(pea);

        Ok(new_path)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        tracing::info!(id = %id, "Deleting pea");

        let file_path = self.find_file_by_id(id)?;
        std::fs::remove_file(&file_path)?;

        // Remove from cache
        self.cache.borrow_mut().remove_pea(id);

        Ok(())
    }

    pub fn archive(&self, id: &str) -> Result<PathBuf> {
        tracing::info!(id = %id, "Archiving pea");

        std::fs::create_dir_all(&self.archive_path)?;

        let old_path = self.find_file_by_id(id)?;
        let filename = old_path
            .file_name()
            .ok_or_else(|| PeasError::Storage("Path has no filename".to_string()))?
            .to_string_lossy()
            .to_string();
        let new_path = self.archive_path.join(&filename);

        std::fs::rename(&old_path, &new_path)?;

        // Remove from cache (it's now in archive, not active list)
        self.cache.borrow_mut().remove_pea(id);

        Ok(new_path)
    }

    pub fn list(&self) -> Result<Vec<Pea>> {
        // Check cache first
        let cache = self.cache.borrow();
        if let Some(cached_list) = cache.get_list() {
            return Ok(cached_list.clone());
        }
        drop(cache); // Release borrow before disk read

        // Cache miss - load from disk
        let peas = self.list_in_path(&self.data_path)?;

        // Update cache with loaded list
        self.cache.borrow_mut().set_list(peas.clone());

        Ok(peas)
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
                                tracing::warn!(
                                    path = %path.display(),
                                    error = %e,
                                    "Failed to parse pea file"
                                )
                            }
                        },
                        Err(e) => tracing::warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to read pea file"
                        ),
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
                path: None,
                prefix: "test-".to_string(),
                id_length: 5,
                id_mode: IdMode::Random,
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
        let _pea2 = repo.get("test-67890").unwrap();

        // First instance modifies and saves
        pea1.title = "Modified by Instance 1".to_string();
        // NOTE: No touch() call - update() handles it internally now
        repo.update(&mut pea1).unwrap();

        // Second instance detects conflict and reloads
        let mut pea2 = repo.get("test-67890").unwrap();

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

    #[test]
    fn test_cache_list_caching() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create some peas
        for i in 0..3 {
            let mut pea = Pea::new(format!("test-{}", i), format!("Pea {}", i), PeaType::Task);
            pea.body = format!("Body {}", i);
            repo.create(&pea).unwrap();
        }

        // First call should read from disk and populate cache
        let list1 = repo.list().unwrap();
        assert_eq!(list1.len(), 3);

        // Second call should use cache (no disk I/O)
        let list2 = repo.list().unwrap();
        assert_eq!(list2.len(), 3);
        assert_eq!(list1, list2);

        // Verify cache is working by checking we get the same results
        for (i, pea) in list2.iter().enumerate().take(3) {
            assert_eq!(pea.title, format!("Pea {}", i));
        }
    }

    #[test]
    fn test_cache_get_by_id_o1_lookup() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create a pea
        let mut pea = Pea::new(
            "test-cache".to_string(),
            "Cached Pea".to_string(),
            PeaType::Task,
        );
        pea.body = "Test body".to_string();
        repo.create(&pea).unwrap();

        // First get() populates cache
        let pea1 = repo.get("test-cache").unwrap();
        assert_eq!(pea1.title, "Cached Pea");

        // Second get() should use cache (O(1) HashMap lookup)
        let pea2 = repo.get("test-cache").unwrap();
        assert_eq!(pea2.title, "Cached Pea");
        assert_eq!(pea1.id, pea2.id);
    }

    #[test]
    fn test_cache_exists_check() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create a pea
        let pea = Pea::new(
            "test-exists".to_string(),
            "Exists Pea".to_string(),
            PeaType::Task,
        );
        repo.create(&pea).unwrap();

        // First exists() check might hit cache from create()
        assert!(repo.exists("test-exists"));

        // Load into cache explicitly
        let _cached = repo.get("test-exists").unwrap();

        // Second exists() should use cache (O(1) lookup)
        assert!(repo.exists("test-exists"));
        assert!(!repo.exists("test-nonexistent"));
    }

    #[test]
    fn test_cache_update_maintains_consistency() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create a pea
        let mut pea = Pea::new(
            "test-update".to_string(),
            "Original Title".to_string(),
            PeaType::Task,
        );
        pea.body = "Original body".to_string();
        repo.create(&pea).unwrap();

        // Load to populate cache
        let mut pea = repo.get("test-update").unwrap();
        assert_eq!(pea.title, "Original Title");

        // Update should invalidate and update cache
        pea.title = "Updated Title".to_string();
        repo.update(&mut pea).unwrap();

        // Get should return updated version from cache
        let updated_pea = repo.get("test-update").unwrap();
        assert_eq!(updated_pea.title, "Updated Title");

        // List should also reflect the update
        let list = repo.list().unwrap();
        let found = list.iter().find(|p| p.id == "test-update").unwrap();
        assert_eq!(found.title, "Updated Title");
    }

    #[test]
    fn test_cache_delete_removes_from_cache() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create a pea
        let pea = Pea::new(
            "test-delete".to_string(),
            "To Delete".to_string(),
            PeaType::Task,
        );
        repo.create(&pea).unwrap();

        // Load to populate cache
        let _loaded = repo.get("test-delete").unwrap();
        assert!(repo.exists("test-delete"));

        // Delete should remove from cache
        repo.delete("test-delete").unwrap();

        // Should not exist in cache or on disk
        assert!(!repo.exists("test-delete"));

        // List should not include deleted pea
        let list = repo.list().unwrap();
        assert!(!list.iter().any(|p| p.id == "test-delete"));
    }

    #[test]
    fn test_cache_archive_removes_from_active_cache() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create a pea
        let pea = Pea::new(
            "test-archive".to_string(),
            "To Archive".to_string(),
            PeaType::Task,
        );
        repo.create(&pea).unwrap();

        // Load to populate cache
        let _loaded = repo.get("test-archive").unwrap();

        // Verify it's in active list
        let list_before = repo.list().unwrap();
        assert!(list_before.iter().any(|p| p.id == "test-archive"));

        // Archive should remove from active cache
        repo.archive("test-archive").unwrap();

        // Should not be in active list
        let list_after = repo.list().unwrap();
        assert!(!list_after.iter().any(|p| p.id == "test-archive"));

        // Should be in archived list
        let archived = repo.list_archived().unwrap();
        assert!(archived.iter().any(|p| p.id == "test-archive"));
    }

    #[test]
    fn test_cache_invalidate_clears_all() {
        let (repo, _temp_dir) = setup_test_repo();

        // Create some peas
        for i in 0..3 {
            let pea = Pea::new(
                format!("test-inv-{}", i),
                format!("Pea {}", i),
                PeaType::Task,
            );
            repo.create(&pea).unwrap();
        }

        // Load list to populate cache
        let list_before = repo.list().unwrap();
        assert_eq!(list_before.len(), 3);

        // Load individual peas
        for i in 0..3 {
            let _pea = repo.get(&format!("test-inv-{}", i)).unwrap();
        }

        // Invalidate cache
        repo.invalidate_cache();

        // List should reload from disk (cache miss)
        let list_after = repo.list().unwrap();
        assert_eq!(list_after.len(), 3);

        // Data should still be consistent
        assert_eq!(list_before, list_after);
    }

    #[test]
    fn test_generate_random_id() {
        let (repo, _temp_dir) = setup_test_repo();

        let id1 = repo.generate_id().unwrap();
        let id2 = repo.generate_id().unwrap();

        // Should have prefix
        assert!(id1.starts_with("test-"));
        assert!(id2.starts_with("test-"));

        // Should be 5 chars after prefix
        assert_eq!(id1.len(), 10); // "test-" (5) + random (5)

        // Random IDs should (almost certainly) be different
        assert_ne!(id1, id2);
    }

    fn setup_sequential_repo() -> (PeaRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = PeasConfig {
            peas: crate::config::PeasSettings {
                path: None,
                prefix: "peas-".to_string(),
                id_length: 5,
                id_mode: IdMode::Sequential,
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
    fn test_generate_sequential_id() {
        let (repo, _temp_dir) = setup_sequential_repo();

        let id1 = repo.generate_id().unwrap();
        let id2 = repo.generate_id().unwrap();
        let id3 = repo.generate_id().unwrap();

        assert_eq!(id1, "peas-00001");
        assert_eq!(id2, "peas-00002");
        assert_eq!(id3, "peas-00003");
    }

    #[test]
    fn test_sequential_id_persists_across_repos() {
        let temp_dir = TempDir::new().unwrap();
        let config = PeasConfig {
            peas: crate::config::PeasSettings {
                path: None,
                prefix: "peas-".to_string(),
                id_length: 5,
                id_mode: IdMode::Sequential,
                default_status: "todo".to_string(),
                default_type: "task".to_string(),
                frontmatter: "toml".to_string(),
            },
            tui: crate::config::TuiSettings::default(),
        };

        // First repo generates some IDs
        let repo1 = PeaRepository::new(&config, temp_dir.path());
        let id1 = repo1.generate_id().unwrap();
        let id2 = repo1.generate_id().unwrap();

        assert_eq!(id1, "peas-00001");
        assert_eq!(id2, "peas-00002");

        // Second repo (simulating restart) should continue from where we left off
        let repo2 = PeaRepository::new(&config, temp_dir.path());
        let id3 = repo2.generate_id().unwrap();
        let id4 = repo2.generate_id().unwrap();

        assert_eq!(id3, "peas-00003");
        assert_eq!(id4, "peas-00004");
    }

    #[test]
    fn test_sequential_id_respects_length() {
        let temp_dir = TempDir::new().unwrap();
        let config = PeasConfig {
            peas: crate::config::PeasSettings {
                path: None,
                prefix: "t-".to_string(),
                id_length: 3,
                id_mode: IdMode::Sequential,
                default_status: "todo".to_string(),
                default_type: "task".to_string(),
                frontmatter: "toml".to_string(),
            },
            tui: crate::config::TuiSettings::default(),
        };
        let repo = PeaRepository::new(&config, temp_dir.path());

        let id = repo.generate_id().unwrap();
        assert_eq!(id, "t-001");
    }
}
