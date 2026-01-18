use crate::error::{PeasError, Result};
use crate::model::Pea;

const FRONTMATTER_DELIMITER: &str = "---";

pub fn parse_markdown(content: &str) -> Result<Pea> {
    let content = content.trim();

    if !content.starts_with(FRONTMATTER_DELIMITER) {
        return Err(PeasError::Parse(
            "Missing YAML frontmatter delimiter".to_string(),
        ));
    }

    let after_first = &content[FRONTMATTER_DELIMITER.len()..];
    let end_index = after_first
        .find(FRONTMATTER_DELIMITER)
        .ok_or_else(|| PeasError::Parse("Missing closing frontmatter delimiter".to_string()))?;

    let yaml_content = &after_first[..end_index].trim();
    let body_start = FRONTMATTER_DELIMITER.len() + end_index + FRONTMATTER_DELIMITER.len();
    let body = content[body_start..].trim().to_string();

    let mut pea: Pea = serde_yaml::from_str(yaml_content)?;
    pea.body = body;

    Ok(pea)
}

pub fn render_markdown(pea: &Pea) -> Result<String> {
    let yaml = serde_yaml::to_string(pea)?;
    let yaml = yaml.trim();

    let mut output = String::new();
    output.push_str(FRONTMATTER_DELIMITER);
    output.push('\n');
    output.push_str(yaml);
    output.push('\n');
    output.push_str(FRONTMATTER_DELIMITER);
    output.push('\n');

    if !pea.body.is_empty() {
        output.push('\n');
        output.push_str(&pea.body);
        output.push('\n');
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PeaStatus, PeaType};

    #[test]
    fn test_parse_markdown() {
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
    fn test_render_markdown() {
        let pea = Pea::new(
            "peas-xyz9".to_string(),
            "My Task".to_string(),
            PeaType::Task,
        )
        .with_body("Task description here.".to_string());

        let rendered = render_markdown(&pea).unwrap();
        assert!(rendered.contains("id: peas-xyz9"));
        assert!(rendered.contains("title: My Task"));
        assert!(rendered.contains("Task description here."));
    }

    #[test]
    fn test_roundtrip() {
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
}
