//! Markdown parsing and rendering with frontmatter support.
//!
//! Supports both YAML (---) and TOML (+++) frontmatter delimiters.

use crate::error::{PeasError, Result};
use crate::model::{Memory, Pea};

const YAML_DELIMITER: &str = "---";
const TOML_DELIMITER: &str = "+++";

/// Frontmatter format detected or to be used for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrontmatterFormat {
    #[default]
    Toml,
    Yaml,
}

impl FrontmatterFormat {
    /// Returns the delimiter string for this format.
    pub fn delimiter(&self) -> &'static str {
        match self {
            FrontmatterFormat::Yaml => YAML_DELIMITER,
            FrontmatterFormat::Toml => TOML_DELIMITER,
        }
    }
}

/// Detects the frontmatter format from content.
pub fn detect_format(content: &str) -> Option<FrontmatterFormat> {
    let content = content.trim();
    if content.starts_with(YAML_DELIMITER) {
        Some(FrontmatterFormat::Yaml)
    } else if content.starts_with(TOML_DELIMITER) {
        Some(FrontmatterFormat::Toml)
    } else {
        None
    }
}

/// Parses markdown content with auto-detected frontmatter format.
pub fn parse_markdown(content: &str) -> Result<Pea> {
    let format = detect_format(content).ok_or_else(|| {
        PeasError::Parse("Missing frontmatter delimiter (--- for YAML or +++ for TOML)".to_string())
    })?;

    parse_markdown_with_format(content, format)
}

/// Parses markdown content with a specific frontmatter format.
pub fn parse_markdown_with_format(content: &str, format: FrontmatterFormat) -> Result<Pea> {
    let content = content.trim();
    let delimiter = format.delimiter();

    if !content.starts_with(delimiter) {
        return Err(PeasError::Parse(format!(
            "Expected {} frontmatter delimiter",
            match format {
                FrontmatterFormat::Yaml => "YAML (---)",
                FrontmatterFormat::Toml => "TOML (+++)",
            }
        )));
    }

    let after_first = &content[delimiter.len()..];
    let end_index = after_first
        .find(delimiter)
        .ok_or_else(|| PeasError::Parse("Missing closing frontmatter delimiter".to_string()))?;

    let frontmatter_content = after_first[..end_index].trim();
    let body_start = delimiter.len() + end_index + delimiter.len();
    let body = content[body_start..].trim().to_string();

    let mut pea: Pea = match format {
        FrontmatterFormat::Yaml => serde_yaml::from_str(frontmatter_content)?,
        FrontmatterFormat::Toml => toml::from_str(frontmatter_content)
            .map_err(|e| PeasError::Parse(format!("TOML parse error: {}", e)))?,
    };
    pea.body = body;

    Ok(pea)
}

/// Renders a pea to markdown with YAML frontmatter (default).
pub fn render_markdown(pea: &Pea) -> Result<String> {
    render_markdown_with_format(pea, FrontmatterFormat::Yaml)
}

/// Renders a pea to markdown with the specified frontmatter format.
pub fn render_markdown_with_format(pea: &Pea, format: FrontmatterFormat) -> Result<String> {
    let delimiter = format.delimiter();

    let frontmatter = match format {
        FrontmatterFormat::Yaml => {
            let yaml = serde_yaml::to_string(pea)?;
            yaml.trim().to_string()
        }
        FrontmatterFormat::Toml => toml::to_string_pretty(pea)
            .map_err(|e| PeasError::Parse(format!("TOML serialize error: {}", e)))?,
    };

    let mut output = String::new();
    output.push_str(delimiter);
    output.push('\n');
    output.push_str(&frontmatter);
    if !frontmatter.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(delimiter);
    output.push('\n');

    if !pea.body.is_empty() {
        output.push('\n');
        output.push_str(&pea.body);
        output.push('\n');
    }

    Ok(output)
}

/// Parses markdown content for a Memory with auto-detected frontmatter format.
pub fn parse_markdown_memory(content: &str) -> Result<Memory> {
    let format = detect_format(content).ok_or_else(|| {
        PeasError::Parse("Missing frontmatter delimiter (--- for YAML or +++ for TOML)".to_string())
    })?;

    parse_markdown_memory_with_format(content, format)
}

/// Parses markdown content for a Memory with a specific frontmatter format.
pub fn parse_markdown_memory_with_format(
    content: &str,
    format: FrontmatterFormat,
) -> Result<Memory> {
    let content = content.trim();
    let delimiter = format.delimiter();

    if !content.starts_with(delimiter) {
        return Err(PeasError::Parse(format!(
            "Expected {} frontmatter delimiter",
            match format {
                FrontmatterFormat::Yaml => "YAML (---)",
                FrontmatterFormat::Toml => "TOML (+++)",
            }
        )));
    }

    let after_first = &content[delimiter.len()..];
    let end_index = after_first
        .find(delimiter)
        .ok_or_else(|| PeasError::Parse("Missing closing frontmatter delimiter".to_string()))?;

    let frontmatter_content = after_first[..end_index].trim();
    let body_start = delimiter.len() + end_index + delimiter.len();
    let body = content[body_start..].trim().to_string();

    let mut memory: Memory = match format {
        FrontmatterFormat::Yaml => serde_yaml::from_str(frontmatter_content)?,
        FrontmatterFormat::Toml => toml::from_str(frontmatter_content)
            .map_err(|e| PeasError::Parse(format!("TOML parse error: {}", e)))?,
    };
    memory.content = body;

    Ok(memory)
}

/// Renders a Memory to markdown with the specified frontmatter format.
pub fn render_markdown_memory(memory: &Memory, format: FrontmatterFormat) -> Result<String> {
    let delimiter = format.delimiter();

    let frontmatter = match format {
        FrontmatterFormat::Yaml => {
            let yaml = serde_yaml::to_string(memory)?;
            yaml.trim().to_string()
        }
        FrontmatterFormat::Toml => toml::to_string_pretty(memory)
            .map_err(|e| PeasError::Parse(format!("TOML serialize error: {}", e)))?,
    };

    let mut output = String::new();
    output.push_str(delimiter);
    output.push('\n');
    output.push_str(&frontmatter);
    if !frontmatter.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(delimiter);
    output.push('\n');

    if !memory.content.is_empty() {
        output.push('\n');
        output.push_str(&memory.content);
        output.push('\n');
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PeaStatus, PeaType};

    #[test]
    fn test_detect_format_yaml() {
        let content = "---\nid: test\n---";
        assert_eq!(detect_format(content), Some(FrontmatterFormat::Yaml));
    }

    #[test]
    fn test_detect_format_toml() {
        let content = "+++\nid = \"test\"\n+++";
        assert_eq!(detect_format(content), Some(FrontmatterFormat::Toml));
    }

    #[test]
    fn test_detect_format_none() {
        let content = "no frontmatter here";
        assert_eq!(detect_format(content), None);
    }

    #[test]
    fn test_parse_yaml_markdown() {
        let content = r#"---
id: peas-abc1
title: Test Task
type: task
status: todo
priority: normal
created: 2024-01-01T00:00:00Z
updated: 2024-01-01T00:00:00Z
---

This is the body content.
"#;

        let pea = parse_markdown(content).unwrap();
        assert_eq!(pea.id, "peas-abc1");
        assert_eq!(pea.title, "Test Task");
        assert_eq!(pea.pea_type, PeaType::Task);
        assert_eq!(pea.status, PeaStatus::Todo);
        assert_eq!(pea.body, "This is the body content.");
    }

    #[test]
    fn test_parse_toml_markdown() {
        let content = r#"+++
id = "peas-xyz9"
title = "TOML Task"
type = "bug"
status = "in-progress"
priority = "high"
created = "2024-01-01T00:00:00Z"
updated = "2024-01-01T00:00:00Z"
+++

This is a TOML frontmatter body.
"#;

        let pea = parse_markdown(content).unwrap();
        assert_eq!(pea.id, "peas-xyz9");
        assert_eq!(pea.title, "TOML Task");
        assert_eq!(pea.pea_type, PeaType::Bug);
        assert_eq!(pea.status, PeaStatus::InProgress);
        assert_eq!(pea.body, "This is a TOML frontmatter body.");
    }

    #[test]
    fn test_render_yaml_markdown() {
        let pea = Pea::new(
            "peas-xyz9".to_string(),
            "My Task".to_string(),
            PeaType::Task,
        )
        .with_body("Task description here.".to_string());

        let rendered = render_markdown(&pea).unwrap();
        assert!(rendered.starts_with("---\n"));
        assert!(rendered.contains("id: peas-xyz9"));
        assert!(rendered.contains("title: My Task"));
        assert!(rendered.contains("Task description here."));
    }

    #[test]
    fn test_render_toml_markdown() {
        let pea = Pea::new(
            "peas-toml1".to_string(),
            "TOML Rendered".to_string(),
            PeaType::Feature,
        )
        .with_body("TOML body content.".to_string());

        let rendered = render_markdown_with_format(&pea, FrontmatterFormat::Toml).unwrap();
        assert!(rendered.starts_with("+++\n"));
        assert!(rendered.contains("id = \"peas-toml1\""));
        assert!(rendered.contains("title = \"TOML Rendered\""));
        assert!(rendered.contains("TOML body content."));
    }

    #[test]
    fn test_yaml_roundtrip() {
        let original = Pea::new(
            "peas-test".to_string(),
            "Roundtrip".to_string(),
            PeaType::Epic,
        )
        .with_status(PeaStatus::InProgress)
        .with_body("Some body text".to_string());

        let rendered = render_markdown(&original).unwrap();
        let parsed = parse_markdown(&rendered).unwrap();

        assert_eq!(original.id, parsed.id);
        assert_eq!(original.title, parsed.title);
        assert_eq!(original.pea_type, parsed.pea_type);
        assert_eq!(original.status, parsed.status);
        assert_eq!(original.body, parsed.body);
    }

    #[test]
    fn test_toml_roundtrip() {
        let original = Pea::new(
            "peas-toml".to_string(),
            "TOML Roundtrip".to_string(),
            PeaType::Bug,
        )
        .with_status(PeaStatus::Completed)
        .with_body("TOML body".to_string());

        let rendered = render_markdown_with_format(&original, FrontmatterFormat::Toml).unwrap();
        let parsed = parse_markdown(&rendered).unwrap();

        assert_eq!(original.id, parsed.id);
        assert_eq!(original.title, parsed.title);
        assert_eq!(original.pea_type, parsed.pea_type);
        assert_eq!(original.status, parsed.status);
        assert_eq!(original.body, parsed.body);
    }
}
