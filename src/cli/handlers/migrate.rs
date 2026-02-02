use crate::config::{DATA_DIR, SCHEMA_URL};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// Legacy config file names in order of preference
const LEGACY_CONFIG_FILES: &[&str] = &[".peas.toml", ".peas.yml", ".peas.yaml", ".peas.json"];

pub fn handle_migrate(dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let data_dir = cwd.join(DATA_DIR);
    let new_config_path = data_dir.join("config.toml");

    // Check if already migrated
    if new_config_path.exists() {
        println!(
            "{} Config already at new location: {}",
            "✓".green(),
            new_config_path.display()
        );

        // Check if there are still legacy configs to clean up
        let mut found_legacy = false;
        for filename in LEGACY_CONFIG_FILES {
            let legacy_path = cwd.join(filename);
            if legacy_path.exists() {
                if !found_legacy {
                    println!("\n{} Legacy config files still present:", "!".yellow());
                    found_legacy = true;
                }
                println!("  - {}", legacy_path.display());
            }
        }

        if found_legacy {
            if dry_run {
                println!(
                    "\n{} Would delete the legacy config files above",
                    "dry-run:".cyan()
                );
            } else {
                println!("\nRemoving legacy config files...");
                for filename in LEGACY_CONFIG_FILES {
                    let legacy_path = cwd.join(filename);
                    if legacy_path.exists() {
                        std::fs::remove_file(&legacy_path)?;
                        println!("  {} Removed {}", "✓".green(), legacy_path.display());
                    }
                }
            }
        }

        return Ok(());
    }

    // Find legacy config
    let legacy_config = LEGACY_CONFIG_FILES
        .iter()
        .map(|f| cwd.join(f))
        .find(|p| p.exists());

    let Some(legacy_path) = legacy_config else {
        println!(
            "{} No config file found. Run `peas init` to create a new project.",
            "!".yellow()
        );
        return Ok(());
    };

    println!("Found legacy config: {}", legacy_path.display());
    println!("New location: {}", new_config_path.display());
    println!();

    // Read and process the config
    let content = std::fs::read_to_string(&legacy_path)?;
    let migrated_content = migrate_config_content(&content, &legacy_path)?;

    if dry_run {
        println!("{}", "dry-run: Would perform the following:".cyan());
        println!("  1. Create {} (if needed)", data_dir.display());
        println!(
            "  2. Write migrated config to {}",
            new_config_path.display()
        );
        println!("  3. Remove {}", legacy_path.display());
        println!();
        println!("{}", "Migrated config would be:".cyan());
        println!("{}", "─".repeat(60));
        println!("{}", migrated_content);
        println!("{}", "─".repeat(60));
        return Ok(());
    }

    // Ensure data directory exists
    std::fs::create_dir_all(&data_dir)?;

    // Write migrated config
    std::fs::write(&new_config_path, &migrated_content)?;
    println!("{} Created {}", "✓".green(), new_config_path.display());

    // Remove legacy config
    std::fs::remove_file(&legacy_path)?;
    println!("{} Removed {}", "✓".green(), legacy_path.display());

    println!();
    println!("{} Migration complete!", "✓".green().bold());

    Ok(())
}

fn migrate_config_content(content: &str, path: &Path) -> Result<String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("toml");

    match ext {
        "toml" => migrate_toml_config(content),
        "yml" | "yaml" => migrate_yaml_config(content),
        "json" => migrate_json_config(content),
        _ => Ok(content.to_string()),
    }
}

fn migrate_toml_config(content: &str) -> Result<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut has_schema = false;
    let mut skip_path = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if schema directive already exists
        if trimmed.starts_with("#:schema") {
            has_schema = true;
            // Update to latest schema URL
            lines.push(format!("#:schema {}", SCHEMA_URL));
            continue;
        }

        // Skip the deprecated path option
        if trimmed.starts_with("path") && trimmed.contains('=') {
            skip_path = true;
            continue;
        }

        // Skip comments about path if we just skipped the path line
        if skip_path && trimmed.starts_with('#') && trimmed.to_lowercase().contains("path") {
            continue;
        }
        skip_path = false;

        lines.push(line.to_string());
    }

    // Add schema directive at the top if not present
    if !has_schema {
        lines.insert(0, format!("#:schema {}", SCHEMA_URL));
        lines.insert(1, String::new());
    }

    // Clean up multiple consecutive blank lines
    let mut result = Vec::new();
    let mut last_was_blank = false;
    for line in lines {
        let is_blank = line.trim().is_empty();
        if is_blank && last_was_blank {
            continue;
        }
        result.push(line);
        last_was_blank = is_blank;
    }

    Ok(result.join("\n"))
}

fn migrate_yaml_config(content: &str) -> Result<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut has_schema = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if schema directive already exists
        if trimmed.starts_with("# yaml-language-server:") {
            has_schema = true;
            lines.push(format!("# yaml-language-server: $schema={}", SCHEMA_URL));
            continue;
        }

        // Skip the deprecated path option
        if trimmed.starts_with("path:") {
            continue;
        }

        lines.push(line.to_string());
    }

    // Add schema directive at the top if not present
    if !has_schema {
        lines.insert(0, format!("# yaml-language-server: $schema={}", SCHEMA_URL));
        lines.insert(1, String::new());
    }

    Ok(lines.join("\n"))
}

fn migrate_json_config(content: &str) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(content)?;

    if let serde_json::Value::Object(ref mut map) = json {
        // Add/update $schema
        map.insert(
            "$schema".to_string(),
            serde_json::Value::String(SCHEMA_URL.to_string()),
        );

        // Remove path from peas section
        if let Some(serde_json::Value::Object(peas)) = map.get_mut("peas") {
            peas.remove("path");
        }
    }

    Ok(serde_json::to_string_pretty(&json)?)
}
