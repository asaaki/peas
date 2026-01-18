//! Input validation for pea data.

use crate::error::{PeasError, Result};

/// Maximum allowed length for a pea title.
pub const MAX_TITLE_LENGTH: usize = 200;

/// Maximum allowed length for a pea body.
pub const MAX_BODY_LENGTH: usize = 50_000;

/// Maximum allowed length for a pea ID.
pub const MAX_ID_LENGTH: usize = 50;

/// Characters forbidden in IDs to prevent path traversal.
const FORBIDDEN_ID_CHARS: &[char] = &['/', '\\', '\0'];

/// Validates a pea title.
pub fn validate_title(title: &str) -> Result<()> {
    if title.is_empty() {
        return Err(PeasError::Validation("Title cannot be empty".to_string()));
    }
    if title.len() > MAX_TITLE_LENGTH {
        return Err(PeasError::Validation(format!(
            "Title exceeds maximum length of {} characters",
            MAX_TITLE_LENGTH
        )));
    }
    Ok(())
}

/// Validates a pea body.
pub fn validate_body(body: &str) -> Result<()> {
    if body.len() > MAX_BODY_LENGTH {
        return Err(PeasError::Validation(format!(
            "Body exceeds maximum length of {} characters",
            MAX_BODY_LENGTH
        )));
    }
    Ok(())
}

/// Validates a pea ID to prevent path traversal attacks.
pub fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(PeasError::Validation("ID cannot be empty".to_string()));
    }
    if id.len() > MAX_ID_LENGTH {
        return Err(PeasError::Validation(format!(
            "ID exceeds maximum length of {} characters",
            MAX_ID_LENGTH
        )));
    }
    if id.contains("..") {
        return Err(PeasError::Validation(
            "ID cannot contain '..' (path traversal)".to_string(),
        ));
    }
    for c in FORBIDDEN_ID_CHARS {
        if id.contains(*c) {
            return Err(PeasError::Validation(format!("ID cannot contain '{}'", c)));
        }
    }
    Ok(())
}

/// Validates a tag name.
pub fn validate_tag(tag: &str) -> Result<()> {
    if tag.is_empty() {
        return Err(PeasError::Validation("Tag cannot be empty".to_string()));
    }
    if tag.len() > 50 {
        return Err(PeasError::Validation(
            "Tag exceeds maximum length of 50 characters".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_title_empty() {
        assert!(validate_title("").is_err());
    }

    #[test]
    fn test_validate_title_valid() {
        assert!(validate_title("A valid title").is_ok());
    }

    #[test]
    fn test_validate_title_too_long() {
        let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
        assert!(validate_title(&long_title).is_err());
    }

    #[test]
    fn test_validate_id_path_traversal() {
        assert!(validate_id("../../../etc/passwd").is_err());
        assert!(validate_id("peas-1234").is_ok());
    }

    #[test]
    fn test_validate_id_forbidden_chars() {
        assert!(validate_id("peas/1234").is_err());
        assert!(validate_id("peas\\1234").is_err());
    }
}
