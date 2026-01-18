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
        std::fs::write(&file_path, content)?;

        Ok(file_path)
    }

    pub fn get(&self, id: &str) -> Result<Pea> {
        let file_path = self.find_file_by_id(id)?;
        let content = std::fs::read_to_string(&file_path)?;
        parse_markdown(&content)
    }

    pub fn update(&self, pea: &Pea) -> Result<PathBuf> {
        // Validate input
        validation::validate_title(&pea.title)?;
        validation::validate_body(&pea.body)?;
        for tag in &pea.tags {
            validation::validate_tag(tag)?;
        }

        let old_path = self.find_file_by_id(&pea.id)?;
        let new_filename = self.generate_filename(&pea.id, &pea.title);
        let new_path = self.data_path.join(&new_filename);

        // Preserve original frontmatter format
        let original_content = std::fs::read_to_string(&old_path)?;
        let format = detect_format(&original_content).unwrap_or(self.frontmatter_format);
        let content = render_markdown_with_format(pea, format)?;

        if old_path != new_path {
            std::fs::remove_file(&old_path)?;
        }

        std::fs::write(&new_path, content)?;
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
}
