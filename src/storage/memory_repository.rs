use super::markdown::{FrontmatterFormat, parse_markdown_memory, render_markdown_memory};
use crate::{
    config::PeasConfig,
    error::{PeasError, Result},
    model::Memory,
    validation,
};
use std::path::{Path, PathBuf};

/// Maximum allowed memory content size: 100 KB
pub const MAX_MEMORY_CONTENT_SIZE: usize = 100_000;

/// Maximum number of memories allowed
pub const MAX_MEMORY_COUNT: usize = 500;

pub struct MemoryRepository {
    memory_path: PathBuf,
    frontmatter_format: FrontmatterFormat,
}

impl MemoryRepository {
    pub fn new(config: &PeasConfig, project_root: &Path) -> Self {
        let memory_path = config.data_path(project_root).join("memory");
        Self {
            memory_path,
            frontmatter_format: config.peas.frontmatter_format(),
        }
    }

    fn validate_key(&self, key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(PeasError::Validation("Key cannot be empty".to_string()));
        }

        // Validate that key is safe for use as a filename
        if key.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
            return Err(PeasError::Validation(
                "Key contains invalid characters for filenames".to_string(),
            ));
        }

        // Prevent directory traversal
        if key.contains("..") {
            return Err(PeasError::Validation("Key cannot contain '..'".to_string()));
        }

        Ok(())
    }

    fn generate_filename(&self, key: &str) -> String {
        format!("{}.md", key)
    }

    fn get_file_path(&self, key: &str) -> PathBuf {
        let filename = self.generate_filename(key);
        self.memory_path.join(filename)
    }

    pub fn create(&self, memory: &Memory) -> Result<PathBuf> {
        // Validate input
        self.validate_key(&memory.key)?;
        validation::validate_body(&memory.content)?;
        if memory.content.len() > MAX_MEMORY_CONTENT_SIZE {
            return Err(PeasError::Validation(format!(
                "Memory content exceeds maximum size of {} bytes",
                MAX_MEMORY_CONTENT_SIZE
            )));
        }
        for tag in &memory.tags {
            validation::validate_tag(tag)?;
        }

        std::fs::create_dir_all(&self.memory_path)?;

        // Check memory count limit
        if self.memory_path.exists() {
            let count = std::fs::read_dir(&self.memory_path)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().is_file()
                        && e.path().extension().map(|ext| ext == "md").unwrap_or(false)
                })
                .count();
            if count >= MAX_MEMORY_COUNT {
                return Err(PeasError::Validation(format!(
                    "Maximum memory count ({}) reached — delete old memories first",
                    MAX_MEMORY_COUNT
                )));
            }
        }

        let file_path = self.get_file_path(&memory.key);

        if file_path.exists() {
            return Err(PeasError::Storage(format!(
                "Memory with key '{}' already exists",
                memory.key
            )));
        }

        let content = render_markdown_memory(memory, self.frontmatter_format)?;
        std::fs::write(&file_path, content)?;

        Ok(file_path)
    }

    pub fn get(&self, key: &str) -> Result<Memory> {
        self.validate_key(key)?;
        let file_path = self.get_file_path(key);

        if !file_path.exists() {
            return Err(PeasError::NotFound(format!("Memory key: {}", key)));
        }

        let content = std::fs::read_to_string(&file_path)?;
        parse_markdown_memory(&content)
    }

    pub fn update(&self, memory: &Memory) -> Result<PathBuf> {
        // Validate input
        self.validate_key(&memory.key)?;
        validation::validate_body(&memory.content)?;
        if memory.content.len() > MAX_MEMORY_CONTENT_SIZE {
            return Err(PeasError::Validation(format!(
                "Memory content exceeds maximum size of {} bytes",
                MAX_MEMORY_CONTENT_SIZE
            )));
        }
        for tag in &memory.tags {
            validation::validate_tag(tag)?;
        }

        let file_path = self.get_file_path(&memory.key);

        if !file_path.exists() {
            return Err(PeasError::NotFound(format!("Memory key: {}", memory.key)));
        }

        let content = render_markdown_memory(memory, self.frontmatter_format)?;
        std::fs::write(&file_path, content)?;

        Ok(file_path)
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        self.validate_key(key)?;
        let file_path = self.get_file_path(key);

        if !file_path.exists() {
            return Err(PeasError::NotFound(format!("Memory key: {}", key)));
        }

        std::fs::remove_file(&file_path)?;
        Ok(())
    }

    pub fn list(&self, tag_filter: Option<&str>) -> Result<Vec<Memory>> {
        if !self.memory_path.exists() {
            return Ok(Vec::new());
        }

        let mut memories = Vec::new();
        for entry in std::fs::read_dir(&self.memory_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match parse_markdown_memory(&content) {
                        Ok(memory) => {
                            // Apply tag filter if specified
                            if let Some(tag) = tag_filter {
                                if memory.tags.contains(&tag.to_string()) {
                                    memories.push(memory);
                                }
                            } else {
                                memories.push(memory);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse {}: {}", path.display(), e)
                        }
                    },
                    Err(e) => eprintln!("Warning: Failed to read {}: {}", path.display(), e),
                }
            }
        }

        // Sort by updated timestamp (newest first)
        memories.sort_by(|a, b| b.updated.cmp(&a.updated));
        Ok(memories)
    }

    /// Return memory usage statistics: count and total size in bytes.
    pub fn stats(&self) -> Result<(usize, u64)> {
        if !self.memory_path.exists() {
            return Ok((0, 0));
        }

        let mut count = 0usize;
        let mut total_bytes = 0u64;

        for entry in std::fs::read_dir(&self.memory_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                count += 1;
                if let Ok(meta) = std::fs::metadata(&path) {
                    total_bytes += meta.len();
                }
            }
        }

        Ok((count, total_bytes))
    }

    pub fn search(&self, query: &str) -> Result<Vec<Memory>> {
        if !self.memory_path.exists() {
            return Ok(Vec::new());
        }

        let query_lower = query.to_lowercase();
        let mut memories = Vec::new();

        for entry in std::fs::read_dir(&self.memory_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match parse_markdown_memory(&content) {
                        Ok(memory) => {
                            // Search in key, content, and tags
                            let key_match = memory.key.to_lowercase().contains(&query_lower);
                            let content_match =
                                memory.content.to_lowercase().contains(&query_lower);
                            let tag_match = memory
                                .tags
                                .iter()
                                .any(|t| t.to_lowercase().contains(&query_lower));

                            if key_match || content_match || tag_match {
                                memories.push(memory);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse {}: {}", path.display(), e)
                        }
                    },
                    Err(e) => eprintln!("Warning: Failed to read {}: {}", path.display(), e),
                }
            }
        }

        // Sort by updated timestamp (newest first)
        memories.sort_by(|a, b| b.updated.cmp(&a.updated));
        Ok(memories)
    }
}
