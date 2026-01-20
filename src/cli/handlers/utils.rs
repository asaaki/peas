use crate::model::{Pea, PeaPriority, PeaStatus};
use crate::undo::UndoManager;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Read};
use std::path::Path;

use super::CommandContext;

/// Resolve body content from CLI arg, file, or stdin
pub fn resolve_body(body: Option<String>, body_file: Option<String>) -> Result<Option<String>> {
    if let Some(b) = body {
        if b == "-" {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            return Ok(Some(content.trim().to_string()));
        }
        return Ok(Some(b));
    }
    if let Some(path_str) = body_file {
        // Validate path to prevent reading arbitrary files
        validate_body_file_path(&path_str)?;

        let content = std::fs::read_to_string(&path_str)
            .with_context(|| format!("Failed to read body from {}", path_str))?;
        return Ok(Some(content.trim().to_string()));
    }
    Ok(None)
}

/// Validate body file path to prevent path traversal and reading sensitive files
fn validate_body_file_path(path_str: &str) -> Result<()> {
    use std::path::Path;

    let path = Path::new(path_str);

    // Reject absolute paths on Unix-like systems
    #[cfg(unix)]
    if path.is_absolute() {
        anyhow::bail!(
            "Absolute paths are not allowed for --body-file. Use relative paths only.\n\
             Attempted path: {}",
            path_str
        );
    }

    // Reject absolute paths on Windows (C:\, \\, etc.)
    #[cfg(windows)]
    if path.is_absolute() {
        anyhow::bail!(
            "Absolute paths are not allowed for --body-file. Use relative paths only.\n\
             Attempted path: {}",
            path_str
        );
    }

    // Check for path traversal attempts (..)
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::ParentDir => {
                anyhow::bail!(
                    "Path traversal (..) is not allowed in --body-file paths.\n\
                     Attempted path: {}",
                    path_str
                );
            }
            Component::RootDir => {
                anyhow::bail!(
                    "Root directory paths are not allowed for --body-file.\n\
                     Attempted path: {}",
                    path_str
                );
            }
            _ => {}
        }
    }

    // Canonicalize and check that resolved path is within current directory
    // This catches symlink attacks and other edge cases
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let full_path = current_dir.join(path);

    // Check if file exists before canonicalize (canonicalize requires file to exist)
    if !full_path.exists() {
        anyhow::bail!("Body file does not exist: {}", path_str);
    }

    let canonical_path = full_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path_str))?;

    let canonical_current = current_dir
        .canonicalize()
        .context("Failed to canonicalize current directory")?;

    // Ensure the canonical path is within the current directory tree
    if !canonical_path.starts_with(&canonical_current) {
        anyhow::bail!(
            "Body file must be within the current directory tree.\n\
             Attempted to access: {}",
            canonical_path.display()
        );
    }

    Ok(())
}

/// Format status with color coding
pub fn format_status(status: PeaStatus) -> colored::ColoredString {
    match status {
        PeaStatus::Draft => "draft".dimmed(),
        PeaStatus::Todo => "todo".white(),
        PeaStatus::InProgress => "in-progress".yellow(),
        PeaStatus::Completed => "completed".green(),
        PeaStatus::Scrapped => "scrapped".red(),
    }
}

/// Format priority with color coding
pub fn format_priority(priority: PeaPriority) -> colored::ColoredString {
    match priority {
        PeaPriority::Critical => "critical".red().bold(),
        PeaPriority::High => "high".red(),
        PeaPriority::Normal => "normal".white(),
        PeaPriority::Low => "low".dimmed(),
        PeaPriority::Deferred => "deferred".dimmed(),
    }
}

/// Print a single pea with details
pub fn print_pea(pea: &Pea) {
    println!("{} {}", pea.id.cyan().bold(), pea.title.bold());
    println!("Type:     {}", format!("{}", pea.pea_type).blue());
    println!("Status:   {}", format_status(pea.status));
    println!("Priority: {}", format_priority(pea.priority));

    if let Some(ref parent) = pea.parent {
        println!("Parent:   {}", parent.cyan());
    }
    if !pea.blocking.is_empty() {
        println!("Blocking: {}", pea.blocking.join(", ").cyan());
    }
    if !pea.tags.is_empty() {
        println!("Tags:     {}", pea.tags.join(", ").yellow());
    }
    println!(
        "Created:  {}",
        pea.created.format("%Y-%m-%d %H:%M").to_string().dimmed()
    );
    println!(
        "Updated:  {}",
        pea.updated.format("%Y-%m-%d %H:%M").to_string().dimmed()
    );

    if !pea.body.is_empty() {
        println!();
        println!("{}", pea.body);
    }
}

/// Print a list of peas (compact format)
pub fn print_pea_list(peas: &[Pea]) {
    if peas.is_empty() {
        println!("No peas found.");
        return;
    }

    for pea in peas {
        let status_str = format_status(pea.status);
        let type_str = format!("{}", pea.pea_type).blue();
        println!(
            "{} {} [{}] {}",
            pea.id.cyan(),
            status_str,
            type_str,
            pea.title
        );
    }
}

/// Record create operation with undo manager
pub fn record_undo_create(ctx: &CommandContext, id: &str, path: &Path) {
    let undo_manager = UndoManager::new(&ctx.config.data_path(&ctx.root));
    let _ = crate::undo::record_create(&undo_manager, id, path);
}

/// Record update operation with undo manager
pub fn record_undo_update(ctx: &CommandContext, id: &str, old_path: &Path) {
    let undo_manager = UndoManager::new(&ctx.config.data_path(&ctx.root));
    let _ = crate::undo::record_update(&undo_manager, id, old_path);
}

/// Record delete operation with undo manager
pub fn record_undo_delete(ctx: &CommandContext, id: &str, file_path: &Path) {
    let undo_manager = UndoManager::new(&ctx.config.data_path(&ctx.root));
    let _ = crate::undo::record_delete(&undo_manager, id, file_path);
}

/// Record archive operation with undo manager
pub fn record_undo_archive(ctx: &CommandContext, id: &str, original: &Path, archive: &Path) {
    let undo_manager = UndoManager::new(&ctx.config.data_path(&ctx.root));
    let _ = crate::undo::record_archive(&undo_manager, id, original, archive);
}
