use crate::model::{Memory, Pea};
use regex::Regex;

/// Search query with optional field-specific and regex support
#[derive(Debug, Clone)]
pub enum SearchQuery {
    /// Simple substring search (case-insensitive)
    Simple(String),
    /// Regex search
    Regex(Regex),
    /// Field-specific search
    Field {
        field: SearchField,
        pattern: Box<SearchQuery>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchField {
    Title,
    Body,
    Tag,
    Id,
    Status,
    Priority,
    Type,
}

impl SearchQuery {
    /// Parse a search query string
    /// Supports:
    /// - Simple: "bug" -> searches all fields
    /// - Field-specific: "title:bug" -> searches title only
    /// - Regex: "regex:bug.*fix" -> regex search
    /// - Combined: "title:regex:bug.*" -> regex in title field
    pub fn parse(query: &str) -> Result<Self, String> {
        if query.is_empty() {
            return Err("Empty query".to_string());
        }

        // Check for field-specific search
        if let Some((field_str, pattern)) = query.split_once(':') {
            // Try to parse field
            if let Ok(field) = field_str.parse::<SearchField>() {
                let sub_query = Self::parse(pattern)?;
                return Ok(SearchQuery::Field {
                    field,
                    pattern: Box::new(sub_query),
                });
            }

            // Check for regex: prefix
            if field_str == "regex" {
                let regex = Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))?;
                return Ok(SearchQuery::Regex(regex));
            }
        }

        // Default to simple substring search
        Ok(SearchQuery::Simple(query.to_string()))
    }

    /// Match against a Pea
    pub fn matches_pea(&self, pea: &Pea) -> bool {
        match self {
            SearchQuery::Simple(pattern) => {
                let pattern_lower = pattern.to_lowercase();
                pea.title.to_lowercase().contains(&pattern_lower)
                    || pea.body.to_lowercase().contains(&pattern_lower)
                    || pea.id.to_lowercase().contains(&pattern_lower)
                    || pea
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&pattern_lower))
            }
            SearchQuery::Regex(regex) => {
                regex.is_match(&pea.title)
                    || regex.is_match(&pea.body)
                    || regex.is_match(&pea.id)
                    || pea.tags.iter().any(|tag| regex.is_match(tag))
            }
            SearchQuery::Field { field, pattern } => match field {
                SearchField::Title => match pattern.as_ref() {
                    SearchQuery::Simple(p) => pea.title.to_lowercase().contains(&p.to_lowercase()),
                    SearchQuery::Regex(r) => r.is_match(&pea.title),
                    _ => false,
                },
                SearchField::Body => match pattern.as_ref() {
                    SearchQuery::Simple(p) => pea.body.to_lowercase().contains(&p.to_lowercase()),
                    SearchQuery::Regex(r) => r.is_match(&pea.body),
                    _ => false,
                },
                SearchField::Tag => match pattern.as_ref() {
                    SearchQuery::Simple(p) => pea
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&p.to_lowercase())),
                    SearchQuery::Regex(r) => pea.tags.iter().any(|tag| r.is_match(tag)),
                    _ => false,
                },
                SearchField::Id => match pattern.as_ref() {
                    SearchQuery::Simple(p) => pea.id.to_lowercase().contains(&p.to_lowercase()),
                    SearchQuery::Regex(r) => r.is_match(&pea.id),
                    _ => false,
                },
                SearchField::Status => {
                    let status_str = pea.status.to_string();
                    match pattern.as_ref() {
                        SearchQuery::Simple(p) => {
                            status_str.to_lowercase().contains(&p.to_lowercase())
                        }
                        SearchQuery::Regex(r) => r.is_match(&status_str),
                        _ => false,
                    }
                }
                SearchField::Priority => {
                    let priority_str = pea.priority.to_string();
                    match pattern.as_ref() {
                        SearchQuery::Simple(p) => {
                            priority_str.to_lowercase().contains(&p.to_lowercase())
                        }
                        SearchQuery::Regex(r) => r.is_match(&priority_str),
                        _ => false,
                    }
                }
                SearchField::Type => {
                    let type_str = pea.pea_type.to_string();
                    match pattern.as_ref() {
                        SearchQuery::Simple(p) => {
                            type_str.to_lowercase().contains(&p.to_lowercase())
                        }
                        SearchQuery::Regex(r) => r.is_match(&type_str),
                        _ => false,
                    }
                }
            },
        }
    }

    /// Match against a Memory
    pub fn matches_memory(&self, memory: &Memory) -> bool {
        match self {
            SearchQuery::Simple(pattern) => {
                let pattern_lower = pattern.to_lowercase();
                memory.key.to_lowercase().contains(&pattern_lower)
                    || memory.content.to_lowercase().contains(&pattern_lower)
                    || memory
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&pattern_lower))
            }
            SearchQuery::Regex(regex) => {
                regex.is_match(&memory.key)
                    || regex.is_match(&memory.content)
                    || memory.tags.iter().any(|tag| regex.is_match(tag))
            }
            SearchQuery::Field { field, pattern } => match field {
                // For Memory, we only support a subset of fields
                SearchField::Tag => match pattern.as_ref() {
                    SearchQuery::Simple(p) => memory
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&p.to_lowercase())),
                    SearchQuery::Regex(r) => memory.tags.iter().any(|tag| r.is_match(tag)),
                    _ => false,
                },
                _ => false, // Other fields don't apply to Memory
            },
        }
    }
}

impl std::str::FromStr for SearchField {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "title" => Ok(SearchField::Title),
            "body" => Ok(SearchField::Body),
            "tag" | "tags" => Ok(SearchField::Tag),
            "id" => Ok(SearchField::Id),
            "status" => Ok(SearchField::Status),
            "priority" => Ok(SearchField::Priority),
            "type" => Ok(SearchField::Type),
            _ => Err(format!("Unknown field: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PeaPriority, PeaStatus, PeaType};

    fn create_test_pea() -> Pea {
        let mut pea = Pea::new(
            "test-123".to_string(),
            "Fix critical bug in parser".to_string(),
            PeaType::Bug,
        );
        pea.body =
            "The parser crashes on malformed input.\nNeed to add error handling.".to_string();
        pea.tags = vec!["bug".to_string(), "parser".to_string()];
        pea.status = PeaStatus::InProgress;
        pea.priority = PeaPriority::Critical;
        pea
    }

    #[test]
    fn test_simple_search() {
        let pea = create_test_pea();

        let query = SearchQuery::parse("bug").unwrap();
        assert!(query.matches_pea(&pea));

        let query = SearchQuery::parse("parser").unwrap();
        assert!(query.matches_pea(&pea));

        let query = SearchQuery::parse("nonexistent").unwrap();
        assert!(!query.matches_pea(&pea));
    }

    #[test]
    fn test_field_specific_search() {
        let pea = create_test_pea();

        // Title search
        let query = SearchQuery::parse("title:critical").unwrap();
        assert!(query.matches_pea(&pea));

        let query = SearchQuery::parse("title:parser").unwrap();
        assert!(query.matches_pea(&pea));

        // Body search
        let query = SearchQuery::parse("body:crashes").unwrap();
        assert!(query.matches_pea(&pea));

        let query = SearchQuery::parse("body:critical").unwrap();
        assert!(!query.matches_pea(&pea)); // "critical" is in title, not body

        // Tag search
        let query = SearchQuery::parse("tag:parser").unwrap();
        assert!(query.matches_pea(&pea));

        let query = SearchQuery::parse("tag:urgent").unwrap();
        assert!(!query.matches_pea(&pea));
    }

    #[test]
    fn test_regex_search() {
        let pea = create_test_pea();

        // Match "bug" or "fix"
        let query = SearchQuery::parse("regex:(bug|fix)").unwrap();
        assert!(query.matches_pea(&pea));

        // Match words starting with "par"
        let query = SearchQuery::parse("regex:par\\w+").unwrap();
        assert!(query.matches_pea(&pea));

        // Invalid regex
        let result = SearchQuery::parse("regex:[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_combined_field_and_regex() {
        let pea = create_test_pea();

        // Regex in title field
        let query = SearchQuery::parse("title:regex:.*critical.*").unwrap();
        assert!(query.matches_pea(&pea));

        // Regex in body field
        let query = SearchQuery::parse("body:regex:crash\\w+").unwrap();
        assert!(query.matches_pea(&pea));
    }

    #[test]
    fn test_search_status_priority_type() {
        let pea = create_test_pea();

        // Status search
        let query = SearchQuery::parse("status:progress").unwrap();
        assert!(query.matches_pea(&pea));

        // Priority search
        let query = SearchQuery::parse("priority:critical").unwrap();
        assert!(query.matches_pea(&pea));

        // Type search
        let query = SearchQuery::parse("type:bug").unwrap();
        assert!(query.matches_pea(&pea));
    }

    #[test]
    fn test_case_insensitive_simple_search() {
        let pea = create_test_pea();

        let query = SearchQuery::parse("CRITICAL").unwrap();
        assert!(query.matches_pea(&pea));

        let query = SearchQuery::parse("BUG").unwrap();
        assert!(query.matches_pea(&pea));
    }

    #[test]
    fn test_case_insensitive_field_search() {
        let pea = create_test_pea();

        let query = SearchQuery::parse("title:CRITICAL").unwrap();
        assert!(query.matches_pea(&pea));

        let query = SearchQuery::parse("TITLE:critical").unwrap();
        assert!(query.matches_pea(&pea));
    }
}
