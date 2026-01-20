use anyhow::{Context, Result};
use colored::Colorize;
use peas::model::{Pea, PeaPriority, PeaStatus};
use peas::undo::UndoManager;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

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
    if let Some(path) = body_file {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read body from {}", path))?;
        return Ok(Some(content.trim().to_string()));
    }
    Ok(None)
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

/// Output JSON or formatted text based on flag
pub fn output_json_or_text<T: serde::Serialize>(
    json: bool,
    value: &T,
    text_fn: impl FnOnce(),
) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        text_fn();
    }
    Ok(())
}

/// Record create operation with undo manager
pub fn record_undo_create(ctx: &CommandContext, id: &str, path: &Path) {
    let undo_manager = UndoManager::new(&ctx.config.data_path(&ctx.root));
    let _ = peas::undo::record_create(&undo_manager, id, path);
}

/// Record update operation with undo manager
pub fn record_undo_update(ctx: &CommandContext, id: &str, old_path: &Path) {
    let undo_manager = UndoManager::new(&ctx.config.data_path(&ctx.root));
    let _ = peas::undo::record_update(&undo_manager, id, old_path);
}

/// Record delete operation with undo manager
pub fn record_undo_delete(ctx: &CommandContext, id: &str, file_path: &Path) {
    let undo_manager = UndoManager::new(&ctx.config.data_path(&ctx.root));
    let _ = peas::undo::record_delete(&undo_manager, id, file_path);
}

/// Record archive operation with undo manager
pub fn record_undo_archive(ctx: &CommandContext, id: &str, original: &Path, archive: &Path) {
    let undo_manager = UndoManager::new(&ctx.config.data_path(&ctx.root));
    let _ = peas::undo::record_archive(&undo_manager, id, original, archive);
}
