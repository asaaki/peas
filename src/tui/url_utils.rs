use std::collections::HashSet;

/// Extract all URLs from text with smart punctuation handling
pub fn extract_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();

    // Find potential URLs with regex
    let url_pattern = regex::Regex::new(r"https?://[^\s<>]+").unwrap();

    for matched in url_pattern.find_iter(text) {
        let mut url_str = matched.as_str();

        // Trim trailing punctuation that's likely not part of the URL
        // Common cases: "Check out https://example.com." or "(see https://example.com)"
        while !url_str.is_empty() {
            let last_char = url_str.chars().last().unwrap();
            if matches!(
                last_char,
                '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '\'' | '"'
            ) {
                // Check if this is actually part of the URL or sentence punctuation
                // If removing it still gives a valid URL, it was probably sentence punctuation
                let trimmed = &url_str[..url_str.len() - last_char.len_utf8()];
                if url::Url::parse(trimmed).is_ok() {
                    url_str = trimmed;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Validate and add if it's a proper URL
        if url::Url::parse(url_str).is_ok() {
            urls.push(url_str.to_string());
        }
    }

    // Deduplicate while preserving order
    let mut seen = HashSet::new();
    urls.retain(|url| seen.insert(url.clone()));

    urls
}
