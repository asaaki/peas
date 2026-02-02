use crate::config::{DATA_DIR, PeasConfig, SCHEMA_URL};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashSet;
use std::path::Path;

/// Legacy config file names
const LEGACY_CONFIG_FILES: &[&str] = &[".peas.toml", ".peas.yml", ".peas.yaml", ".peas.json"];

#[derive(Default)]
struct DiagnosticResults {
    passed: usize,
    warnings: usize,
    errors: usize,
}

impl DiagnosticResults {
    fn pass(&mut self, message: &str) {
        self.passed += 1;
        println!("  {} {}", "✓".green(), message);
    }

    fn warn(&mut self, message: &str) {
        self.warnings += 1;
        println!("  {} {}", "!".yellow(), message);
    }

    fn error(&mut self, message: &str) {
        self.errors += 1;
        println!("  {} {}", "✗".red(), message);
    }

    fn suggestion(&self, message: &str) {
        println!("    {} {}", "→".cyan(), message);
    }
}

pub fn handle_doctor(fix: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mut results = DiagnosticResults::default();

    println!("{}", "peas doctor".bold());
    println!("{}", "═".repeat(60));
    println!();

    // Check 1: Config location
    check_config_location(&cwd, &mut results, fix)?;

    // Check 2: Data directory
    check_data_directory(&cwd, &mut results)?;

    // Check 3: Config content
    check_config_content(&cwd, &mut results, fix)?;

    // Check 4: Ticket format validation
    check_ticket_format(&cwd, &mut results)?;

    // Check 5: Ticket integrity
    check_ticket_integrity(&cwd, &mut results)?;

    // Check 6: Mixed ID styles
    check_mixed_id_styles(&cwd, &mut results)?;

    // Check 7: Sequential ID counter (if applicable)
    check_sequential_counter(&cwd, &mut results, fix)?;

    // Summary
    println!();
    println!("{}", "═".repeat(60));
    print_summary(&results);

    Ok(())
}

fn check_config_location(cwd: &Path, results: &mut DiagnosticResults, fix: bool) -> Result<()> {
    println!("{}", "Config Location".bold());

    let data_dir = cwd.join(DATA_DIR);
    let new_config = data_dir.join("config.toml");

    // Check for new location
    if new_config.exists() {
        results.pass("Config at canonical location (.peas/config.toml)");

        // Check for leftover legacy configs
        let legacy_files: Vec<_> = LEGACY_CONFIG_FILES
            .iter()
            .map(|f| cwd.join(f))
            .filter(|p| p.exists())
            .collect();

        if !legacy_files.is_empty() {
            results.warn("Legacy config files still present");
            for path in &legacy_files {
                println!("      - {}", path.file_name().unwrap().to_string_lossy());
            }
            if fix {
                for path in legacy_files {
                    std::fs::remove_file(&path)?;
                    println!(
                        "      {} Removed {}",
                        "✓".green(),
                        path.file_name().unwrap().to_string_lossy()
                    );
                }
            } else {
                results.suggestion("Run `peas doctor --fix` to clean up");
            }
        }
    } else {
        // Check for legacy configs
        let legacy_found = LEGACY_CONFIG_FILES
            .iter()
            .map(|f| cwd.join(f))
            .find(|p| p.exists());

        if let Some(legacy_path) = legacy_found {
            results.warn(&format!(
                "Using legacy config location: {}",
                legacy_path.file_name().unwrap().to_string_lossy()
            ));
            if fix {
                // Run migration inline
                println!("      Migrating config...");
                crate::cli::handlers::handle_migrate(false)?;
                println!("      {} Migration complete", "✓".green());
            } else {
                results.suggestion("Run `peas doctor --fix` to migrate");
            }
        } else {
            results.error("No config file found");
            results.suggestion("Run `peas init` to create a new project");
        }
    }

    println!();
    Ok(())
}

fn check_data_directory(cwd: &Path, results: &mut DiagnosticResults) -> Result<()> {
    println!("{}", "Data Directory".bold());

    let data_dir = cwd.join(DATA_DIR);

    if !data_dir.exists() {
        results.error(".peas/ directory does not exist");
        results.suggestion("Run `peas init` to initialize the project");
        println!();
        return Ok(());
    }

    if !data_dir.is_dir() {
        results.error(".peas exists but is not a directory");
        println!();
        return Ok(());
    }

    results.pass(".peas/ directory exists");

    // Check read/write permissions
    let test_file = data_dir.join(".doctor-test");
    match std::fs::write(&test_file, "test") {
        Ok(_) => {
            std::fs::remove_file(&test_file)?;
            results.pass("Directory is writable");
        }
        Err(_) => {
            results.error("Directory is not writable");
        }
    }

    // Check archive directory
    let archive_dir = data_dir.join("archive");
    if archive_dir.exists() {
        results.pass("Archive directory exists");
    }

    println!();
    Ok(())
}

fn check_config_content(cwd: &Path, results: &mut DiagnosticResults, fix: bool) -> Result<()> {
    println!("{}", "Config Content".bold());

    let data_dir = cwd.join(DATA_DIR);
    let config_path = data_dir.join("config.toml");

    if !config_path.exists() {
        // Try legacy locations
        let legacy = LEGACY_CONFIG_FILES
            .iter()
            .map(|f| cwd.join(f))
            .find(|p| p.exists());

        if legacy.is_none() {
            results.warn("No config file to check");
            println!();
            return Ok(());
        }
    }

    let config_path = if config_path.exists() {
        config_path
    } else {
        LEGACY_CONFIG_FILES
            .iter()
            .map(|f| cwd.join(f))
            .find(|p| p.exists())
            .unwrap()
    };

    let content = std::fs::read_to_string(&config_path)?;

    // Check for schema directive
    let has_schema = content.contains("#:schema")
        || content.contains("yaml-language-server:")
        || content.contains("\"$schema\"");

    if has_schema {
        // Check if it's the latest schema URL
        if content.contains(SCHEMA_URL) {
            results.pass("Schema directive present with latest URL");
        } else {
            results.warn("Schema directive present but may be outdated");
            results.suggestion(&format!("Update to: {}", SCHEMA_URL));
        }
    } else {
        results.warn("No schema directive for LSP support");
        if fix && config_path.extension().and_then(|e| e.to_str()) == Some("toml") {
            let new_content = format!("#:schema {}\n\n{}", SCHEMA_URL, content);
            std::fs::write(&config_path, new_content)?;
            println!("      {} Added schema directive", "✓".green());
        } else {
            results.suggestion("Run `peas migrate` or `peas doctor --fix` to add it");
        }
    }

    // Check for deprecated path option
    if content.contains("path =") || content.contains("path:") {
        results.warn("Deprecated 'path' option found (ignored)");
        results.suggestion("Remove it manually or run `peas migrate`");
    } else {
        results.pass("No deprecated options");
    }

    // Try parsing the config
    match toml::from_str::<PeasConfig>(&content) {
        Ok(_) => results.pass("Config parses successfully"),
        Err(e) => {
            results.error(&format!("Config parse error: {}", e));
        }
    }

    println!();
    Ok(())
}

fn check_ticket_format(cwd: &Path, results: &mut DiagnosticResults) -> Result<()> {
    println!("{}", "Ticket Format Validation".bold());

    let data_dir = cwd.join(DATA_DIR);
    if !data_dir.exists() {
        results.warn("No data directory to check");
        println!();
        return Ok(());
    }

    let mut total_tickets = 0;
    let mut format_issues: Vec<(String, String)> = Vec::new();

    for entry in std::fs::read_dir(&data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
            total_tickets += 1;
            let filename = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");
            let content = std::fs::read_to_string(&path)?;

            // Check for frontmatter delimiters
            if !content.starts_with("+++") && !content.starts_with("---") {
                format_issues.push((
                    filename.to_string(),
                    "Missing frontmatter delimiters".to_string(),
                ));
                continue;
            }

            // Extract frontmatter for raw validation
            let delimiter = if content.starts_with("+++") {
                "+++"
            } else {
                "---"
            };
            let parts: Vec<&str> = content.splitn(3, delimiter).collect();
            if parts.len() < 3 {
                format_issues.push((
                    filename.to_string(),
                    "Malformed frontmatter structure".to_string(),
                ));
                continue;
            }

            let frontmatter = parts[1].trim();

            // Check for malformed array fields (comma-separated strings instead of arrays)
            // This catches: blocking = ["a,b,c"] instead of blocking = ["a", "b", "c"]
            for field in ["blocking", "tags", "assets"] {
                let pattern = format!("{} = [\"", field);
                if let Some(start) = frontmatter.find(&pattern) {
                    let after_bracket = start + pattern.len();
                    if let Some(end) = frontmatter[after_bracket..].find("\"]") {
                        let value = &frontmatter[after_bracket..after_bracket + end];
                        // If the value contains commas but no quotes, it's likely malformed
                        if value.contains(',') && !value.contains("\", \"") {
                            format_issues.push((
                                filename.to_string(),
                                format!(
                                    "Malformed {} array: contains comma-separated string instead of array elements",
                                    field
                                ),
                            ));
                        }
                    }
                }
            }

            // Try to parse and check for additional issues
            match crate::storage::parse_markdown(&content) {
                Ok(pea) => {
                    // Check ID format - should start with a prefix and have reasonable length
                    if pea.id.is_empty() {
                        format_issues.push((filename.to_string(), "Empty ticket ID".to_string()));
                    } else if !pea.id.contains('-') {
                        format_issues.push((
                            filename.to_string(),
                            format!("ID '{}' missing prefix separator", pea.id),
                        ));
                    }

                    // Check title
                    if pea.title.is_empty() {
                        format_issues
                            .push((filename.to_string(), "Empty ticket title".to_string()));
                    }

                    // Check parent format if present
                    if let Some(ref parent) = pea.parent {
                        if parent.is_empty() {
                            format_issues
                                .push((filename.to_string(), "Empty parent reference".to_string()));
                        } else if !parent.contains('-') {
                            format_issues.push((
                                filename.to_string(),
                                format!("Parent '{}' has invalid ID format", parent),
                            ));
                        }
                    }

                    // Check blocking references format
                    for blocked in &pea.blocking {
                        if blocked.is_empty() {
                            format_issues.push((
                                filename.to_string(),
                                "Empty blocking reference".to_string(),
                            ));
                        } else if !blocked.contains('-') && blocked.contains(',') {
                            // Likely a comma-separated string that wasn't caught above
                            format_issues.push((
                                filename.to_string(),
                                format!(
                                    "Blocking '{}' appears to be comma-separated (should be array)",
                                    blocked
                                ),
                            ));
                        }
                    }
                }
                Err(e) => {
                    format_issues.push((filename.to_string(), format!("Parse error: {}", e)));
                }
            }
        }
    }

    if total_tickets == 0 {
        results.pass("No tickets to validate");
        println!();
        return Ok(());
    }

    if format_issues.is_empty() {
        results.pass(&format!("All {} tickets are well-formed", total_tickets));
    } else {
        results.error(&format!(
            "{} format issues in {} tickets",
            format_issues.len(),
            total_tickets
        ));
        for (filename, issue) in &format_issues {
            println!("      - {}: {}", filename, issue);
        }
    }

    println!();
    Ok(())
}

fn check_ticket_integrity(cwd: &Path, results: &mut DiagnosticResults) -> Result<()> {
    println!("{}", "Ticket Integrity".bold());

    let data_dir = cwd.join(DATA_DIR);
    if !data_dir.exists() {
        results.warn("No data directory to check");
        println!();
        return Ok(());
    }

    // Collect all ticket IDs
    let mut ticket_ids: HashSet<String> = HashSet::new();
    let mut tickets_with_parents: Vec<(String, String)> = Vec::new();
    let mut tickets_with_blocking: Vec<(String, Vec<String>)> = Vec::new();
    let mut parse_errors = 0;
    let mut total_tickets = 0;

    for entry in std::fs::read_dir(&data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
            total_tickets += 1;
            let content = std::fs::read_to_string(&path)?;

            match crate::storage::parse_markdown(&content) {
                Ok(pea) => {
                    // Check for duplicate IDs
                    if !ticket_ids.insert(pea.id.clone()) {
                        results.error(&format!("Duplicate ID: {}", pea.id));
                    }

                    // Collect parent references
                    if let Some(ref parent) = pea.parent {
                        tickets_with_parents.push((pea.id.clone(), parent.clone()));
                    }

                    // Collect blocking references
                    if !pea.blocking.is_empty() {
                        tickets_with_blocking.push((pea.id.clone(), pea.blocking.clone()));
                    }
                }
                Err(_) => {
                    parse_errors += 1;
                }
            }
        }
    }

    if total_tickets == 0 {
        results.pass("No tickets to check");
        println!();
        return Ok(());
    }

    results.pass(&format!("{} tickets found", total_tickets));

    if parse_errors > 0 {
        results.error(&format!("{} tickets failed to parse", parse_errors));
    }

    // Check parent references
    let mut orphaned_parents = 0;
    for (id, parent) in &tickets_with_parents {
        if !ticket_ids.contains(parent) {
            if orphaned_parents == 0 {
                results.warn("Orphaned parent references found:");
            }
            orphaned_parents += 1;
            println!("      - {} references missing parent {}", id, parent);
        }
    }
    if orphaned_parents == 0 && !tickets_with_parents.is_empty() {
        results.pass("All parent references valid");
    }

    // Check blocking references
    let mut orphaned_blocking = 0;
    for (id, blocking) in &tickets_with_blocking {
        for blocked_id in blocking {
            if !ticket_ids.contains(blocked_id) {
                if orphaned_blocking == 0 {
                    results.warn("Orphaned blocking references found:");
                }
                orphaned_blocking += 1;
                println!("      - {} blocks missing ticket {}", id, blocked_id);
            }
        }
    }
    if orphaned_blocking == 0 && !tickets_with_blocking.is_empty() {
        results.pass("All blocking references valid");
    }

    println!();
    Ok(())
}

fn check_mixed_id_styles(cwd: &Path, results: &mut DiagnosticResults) -> Result<()> {
    let data_dir = cwd.join(DATA_DIR);
    if !data_dir.exists() {
        return Ok(());
    }

    let mut sequential_ids: Vec<String> = Vec::new();
    let mut random_ids: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = std::fs::read_to_string(&path)?;

            if let Ok(pea) = crate::storage::parse_markdown(&content) {
                // Extract the suffix (part after the last hyphen in the prefix)
                // e.g., "peas-00042" -> "00042", "peas-a1b2c" -> "a1b2c"
                if let Some(suffix) = pea.id.rsplit('-').next() {
                    // Sequential IDs are all digits (possibly with leading zeros)
                    if suffix.chars().all(|c| c.is_ascii_digit()) {
                        sequential_ids.push(pea.id.clone());
                    } else {
                        random_ids.push(pea.id.clone());
                    }
                }
            }
        }
    }

    // Only report if we have both styles
    if !sequential_ids.is_empty() && !random_ids.is_empty() {
        println!("{}", "ID Style Consistency".bold());
        results.warn("Mixed ID styles detected");
        println!(
            "      Sequential IDs: {} (e.g., {})",
            sequential_ids.len(),
            sequential_ids.first().unwrap()
        );
        println!(
            "      Random IDs: {} (e.g., {})",
            random_ids.len(),
            random_ids.first().unwrap()
        );
        results.suggestion(
            "This can happen when switching id_mode - it's functional but inconsistent",
        );
        println!();
    }

    Ok(())
}

fn check_sequential_counter(cwd: &Path, results: &mut DiagnosticResults, fix: bool) -> Result<()> {
    let data_dir = cwd.join(DATA_DIR);
    let counter_path = data_dir.join(".id");

    if !counter_path.exists() {
        // No sequential counter - that's fine, might be using random IDs
        return Ok(());
    }

    println!("{}", "Sequential ID Counter".bold());

    let counter_content = std::fs::read_to_string(&counter_path)?;
    let counter: u64 = counter_content.trim().parse().unwrap_or(0);

    // Find highest sequential ID in use
    let mut highest_id: u64 = 0;
    let mut sequential_tickets = 0;

    for entry in std::fs::read_dir(&data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && path.extension().map(|e| e == "md").unwrap_or(false)
            && let Some(filename) = path.file_name().and_then(|f| f.to_str())
        {
            // Try to extract numeric ID (e.g., "peas-00042--title.md" -> 42)
            if let Some(id_part) = filename.split("--").next()
                && let Some(num_part) = id_part.split('-').next_back()
                && let Ok(num) = num_part.parse::<u64>()
            {
                sequential_tickets += 1;
                if num > highest_id {
                    highest_id = num;
                }
            }
        }
    }

    if sequential_tickets == 0 {
        results.pass("Sequential counter file exists (no sequential tickets yet)");
        println!();
        return Ok(());
    }

    results.pass(&format!(
        "Counter value: {}, highest ticket ID: {}",
        counter, highest_id
    ));

    if counter < highest_id {
        results.error(&format!(
            "Counter ({}) is lower than highest ticket ID ({})",
            counter, highest_id
        ));
        if fix {
            let new_counter = highest_id;
            std::fs::write(&counter_path, new_counter.to_string())?;
            println!("      {} Updated counter to {}", "✓".green(), new_counter);
        } else {
            results.suggestion("Run `peas doctor --fix` to update counter");
        }
    } else {
        results.pass("Counter is in sync");
    }

    println!();
    Ok(())
}

fn print_summary(results: &DiagnosticResults) {
    let total = results.passed + results.warnings + results.errors;

    print!("Summary: ");
    print!("{} passed", format!("{}", results.passed).green());
    if results.warnings > 0 {
        print!(", {}", format!("{} warnings", results.warnings).yellow());
    }
    if results.errors > 0 {
        print!(", {}", format!("{} errors", results.errors).red());
    }
    println!(" ({} checks)", total);

    if results.errors > 0 {
        println!();
        println!(
            "{}",
            "Some issues need attention. Run suggested commands to fix.".red()
        );
    } else if results.warnings > 0 {
        println!();
        println!(
            "{}",
            "Some improvements suggested. Run `peas doctor --fix` to apply.".yellow()
        );
    } else {
        println!();
        println!("{}", "All checks passed!".green());
    }
}
