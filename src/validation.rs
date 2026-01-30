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

/// Validates that a parent exists (if specified).
/// Pass a closure that checks if an ID exists in the repository.
pub fn validate_parent_exists<F>(parent: &Option<String>, exists_fn: F) -> Result<()>
where
    F: Fn(&str) -> bool,
{
    if let Some(parent_id) = parent
        && !exists_fn(parent_id)
    {
        return Err(PeasError::Validation(format!(
            "Parent pea '{}' does not exist",
            parent_id
        )));
    }
    Ok(())
}

/// Validates that a pea doesn't reference itself as parent.
pub fn validate_no_self_parent(id: &str, parent: &Option<String>) -> Result<()> {
    if let Some(parent_id) = parent
        && id == parent_id
    {
        return Err(PeasError::Validation(
            "A pea cannot be its own parent".to_string(),
        ));
    }
    Ok(())
}

/// Validates that blocking relationships don't contain the pea's own ID.
pub fn validate_no_self_blocking(id: &str, blocking: &[String]) -> Result<()> {
    if blocking.contains(&id.to_string()) {
        return Err(PeasError::Validation(
            "A pea cannot block itself".to_string(),
        ));
    }
    Ok(())
}

/// Validates that all blocking IDs exist.
pub fn validate_blocking_exist<F>(blocking: &[String], exists_fn: F) -> Result<()>
where
    F: Fn(&str) -> bool,
{
    for blocked_id in blocking {
        if !exists_fn(blocked_id) {
            return Err(PeasError::Validation(format!(
                "Blocked pea '{}' does not exist",
                blocked_id
            )));
        }
    }
    Ok(())
}

/// Checks for circular parent-child relationship by walking up the parent chain.
/// Pass a closure that retrieves a pea's parent ID.
pub fn validate_no_circular_parent<F>(
    id: &str,
    new_parent: &Option<String>,
    get_parent_fn: F,
) -> Result<()>
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(parent_id) = new_parent {
        // Walk up the parent chain to check if we'd create a cycle
        let mut current = parent_id.clone();
        let mut visited = std::collections::HashSet::new();
        visited.insert(id.to_string());

        loop {
            if current == id {
                return Err(PeasError::Validation(format!(
                    "Setting '{}' as parent would create a circular relationship",
                    parent_id
                )));
            }

            visited.insert(current.clone());

            match get_parent_fn(&current) {
                Some(next_parent) => {
                    if visited.contains(&next_parent) {
                        // Cycle detected in existing data (shouldn't happen but be safe)
                        return Err(PeasError::Validation(format!(
                            "Circular parent relationship detected in existing data involving '{}'",
                            current
                        )));
                    }
                    current = next_parent;
                }
                None => break, // Reached the root
            }
        }
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

    #[test]
    fn test_validate_no_self_parent() {
        assert!(validate_no_self_parent("peas-123", &Some("peas-123".to_string())).is_err());
        assert!(validate_no_self_parent("peas-123", &Some("peas-456".to_string())).is_ok());
        assert!(validate_no_self_parent("peas-123", &None).is_ok());
    }

    #[test]
    fn test_validate_no_self_blocking() {
        assert!(validate_no_self_blocking("peas-123", &["peas-123".to_string()]).is_err());
        assert!(validate_no_self_blocking("peas-123", &["peas-456".to_string()]).is_ok());
        assert!(validate_no_self_blocking("peas-123", &[]).is_ok());
    }

    #[test]
    fn test_validate_parent_exists() {
        let exists_fn = |id: &str| id == "peas-999";

        assert!(validate_parent_exists(&Some("peas-999".to_string()), exists_fn).is_ok());
        assert!(validate_parent_exists(&Some("peas-404".to_string()), exists_fn).is_err());
        assert!(validate_parent_exists(&None, exists_fn).is_ok());
    }

    #[test]
    fn test_validate_blocking_exist() {
        let exists_fn = |id: &str| id == "peas-111" || id == "peas-222";

        assert!(validate_blocking_exist(&["peas-111".to_string()], exists_fn).is_ok());
        assert!(
            validate_blocking_exist(&["peas-111".to_string(), "peas-222".to_string()], exists_fn)
                .is_ok()
        );
        assert!(validate_blocking_exist(&["peas-404".to_string()], exists_fn).is_err());
    }

    #[test]
    fn test_validate_no_circular_parent() {
        // Setup: peas-1 -> peas-2 -> peas-3
        let get_parent = |id: &str| match id {
            "peas-2" => Some("peas-1".to_string()),
            "peas-3" => Some("peas-2".to_string()),
            _ => None,
        };

        // OK: peas-4 -> peas-3 (no cycle)
        assert!(
            validate_no_circular_parent("peas-4", &Some("peas-3".to_string()), get_parent).is_ok()
        );

        // ERROR: peas-1 -> peas-3 would create cycle (3 -> 2 -> 1 -> 3)
        assert!(
            validate_no_circular_parent("peas-1", &Some("peas-3".to_string()), get_parent).is_err()
        );

        // ERROR: Direct self-reference
        assert!(
            validate_no_circular_parent("peas-1", &Some("peas-1".to_string()), get_parent).is_err()
        );
    }
}
