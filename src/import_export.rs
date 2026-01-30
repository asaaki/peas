//! Import and export functionality for beans format compatibility.

use crate::error::{PeasError, Result};
use crate::model::{Pea, PeaPriority, PeaStatus, PeaType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Beans frontmatter structure (YAML format)
#[derive(Debug, Deserialize)]
struct BeansFrontmatter {
    title: String,
    status: String,
    #[serde(rename = "type")]
    pea_type: String,
    #[serde(default = "default_priority")]
    priority: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    blocking: Vec<String>,
}

fn default_priority() -> String {
    "normal".to_string()
}

/// Beans export frontmatter structure
#[derive(Debug, Serialize)]
struct BeansExportFrontmatter {
    title: String,
    status: String,
    #[serde(rename = "type")]
    pea_type: String,
    priority: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    blocking: Vec<String>,
}

/// Parse a beans markdown file and convert to Pea
pub fn parse_beans_file(content: &str, filename: &str) -> Result<Pea> {
    let content = content.trim();

    // Beans uses YAML frontmatter with --- delimiters
    if !content.starts_with("---") {
        return Err(PeasError::Parse(
            "Beans file must start with YAML frontmatter (---)".to_string(),
        ));
    }

    // Find the closing delimiter
    let rest = &content[3..];
    let end_idx = rest
        .find("\n---")
        .ok_or_else(|| PeasError::Parse("Missing closing frontmatter delimiter".to_string()))?;

    let frontmatter_str = &rest[..end_idx].trim();
    let body = rest[end_idx + 4..].trim();

    // Extract ID from first line comment (# peas-xxxx)
    let id = extract_beans_id(frontmatter_str, filename)?;

    // Remove the ID comment line for YAML parsing
    let yaml_content: String = frontmatter_str
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    let fm: BeansFrontmatter =
        serde_yaml::from_str(&yaml_content).map_err(|e| PeasError::Parse(e.to_string()))?;

    let pea_type = fm.pea_type.parse::<PeaType>().unwrap_or_default();
    let status = fm.status.parse::<PeaStatus>().unwrap_or_default();
    let priority = fm.priority.parse::<PeaPriority>().unwrap_or_default();

    let mut pea = Pea::new(id, fm.title, pea_type)
        .with_status(status)
        .with_priority(priority)
        .with_body(body.to_string());

    pea.created = fm.created_at;
    pea.updated = fm.updated_at;

    if let Some(parent) = fm.parent {
        pea.parent = Some(parent);
    }

    if !fm.tags.is_empty() {
        pea.tags = fm.tags;
    }

    if !fm.blocking.is_empty() {
        pea.blocking = fm.blocking;
    }

    Ok(pea)
}

/// Extract ID from beans frontmatter (comment line or filename)
fn extract_beans_id(frontmatter: &str, filename: &str) -> Result<String> {
    // Try to find ID in comment: # peas-xxxx or # beans-xxxx
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix('#') {
            let id = stripped.trim();
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }
    }

    // Fall back to extracting from filename: peas-xxxx--title.md or beans-xxxx--title.md
    let stem = filename.trim_end_matches(".md");
    if let Some(idx) = stem.find("--") {
        return Ok(stem[..idx].to_string());
    }

    Err(PeasError::Parse(format!(
        "Could not extract ID from file: {}",
        filename
    )))
}

/// Import all beans files from a directory
pub fn import_beans_directory(path: &Path) -> Result<Vec<Pea>> {
    if !path.exists() {
        return Err(PeasError::Storage(format!(
            "Directory does not exist: {}",
            path.display()
        )));
    }

    let mut peas = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() && file_path.extension().is_some_and(|e| e == "md") {
            let filename = file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let content = std::fs::read_to_string(&file_path)?;

            // Skip files that are already in peas/TOML format
            if content.trim().starts_with("+++") {
                continue;
            }

            match parse_beans_file(&content, &filename) {
                Ok(pea) => peas.push(pea),
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", file_path.display(), e);
                }
            }
        }
    }

    Ok(peas)
}

/// Export a Pea to beans format (YAML frontmatter)
pub fn export_to_beans(pea: &Pea) -> Result<String> {
    let fm = BeansExportFrontmatter {
        title: pea.title.clone(),
        status: pea.status.to_string(),
        pea_type: pea.pea_type.to_string(),
        priority: pea.priority.to_string(),
        created_at: pea.created,
        updated_at: pea.updated,
        parent: pea.parent.clone(),
        tags: pea.tags.clone(),
        blocking: pea.blocking.clone(),
    };

    let yaml = serde_yaml::to_string(&fm).map_err(|e| PeasError::Parse(e.to_string()))?;

    let mut output = String::new();
    output.push_str("---\n");
    output.push_str(&format!("# {}\n", pea.id));
    output.push_str(&yaml);
    output.push_str("---\n");

    if !pea.body.is_empty() {
        output.push('\n');
        output.push_str(&pea.body);
        output.push('\n');
    }

    Ok(output)
}

/// Generate beans-style filename
pub fn beans_filename(pea: &Pea) -> String {
    let slug = slug::slugify(&pea.title);
    let slug = if slug.len() > 50 {
        slug[..50].to_string()
    } else {
        slug
    };
    format!("{}--{}.md", pea.id, slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_beans_file() {
        let content = r#"---
# peas-test1
title: Test Bean
status: todo
type: task
priority: normal
created_at: 2026-01-18T12:00:00Z
updated_at: 2026-01-18T12:00:00Z
---

This is the body content."#;

        let pea = parse_beans_file(content, "peas-test1--test-bean.md").unwrap();
        assert_eq!(pea.id, "peas-test1");
        assert_eq!(pea.title, "Test Bean");
        assert_eq!(pea.status, PeaStatus::Todo);
        assert_eq!(pea.pea_type, PeaType::Task);
        assert_eq!(pea.body, "This is the body content.");
    }

    #[test]
    fn test_parse_beans_with_parent() {
        let content = r#"---
# peas-child
title: Child Task
status: in-progress
type: task
priority: high
created_at: 2026-01-18T12:00:00Z
updated_at: 2026-01-18T12:00:00Z
parent: peas-parent
---
"#;

        let pea = parse_beans_file(content, "peas-child--child-task.md").unwrap();
        assert_eq!(pea.parent, Some("peas-parent".to_string()));
    }

    #[test]
    fn test_export_to_beans() {
        let pea = Pea::new(
            "peas-export".to_string(),
            "Export Test".to_string(),
            PeaType::Task,
        );
        let output = export_to_beans(&pea).unwrap();

        assert!(output.starts_with("---\n# peas-export\n"));
        assert!(output.contains("title: Export Test"));
        assert!(output.contains("status: todo"));
    }
}
