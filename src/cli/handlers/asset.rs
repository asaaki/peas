use crate::assets::AssetManager;
use crate::cli::commands::AssetAction;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use super::CommandContext;

pub fn handle_asset(ctx: &CommandContext, action: AssetAction) -> Result<()> {
    let asset_manager = AssetManager::new(&ctx.root);

    match action {
        AssetAction::Add {
            ticket_id,
            file,
            json,
        } => handle_asset_add(&asset_manager, &ctx, &ticket_id, &file, json),
        AssetAction::List { ticket_id, json } => {
            handle_asset_list(&asset_manager, &ticket_id, json)
        }
        AssetAction::Remove {
            ticket_id,
            filename,
            force,
            json,
        } => handle_asset_remove(&asset_manager, &ctx, &ticket_id, &filename, force, json),
        AssetAction::Open {
            ticket_id,
            filename,
        } => handle_asset_open(&asset_manager, &ticket_id, &filename),
    }
}

fn handle_asset_add(
    asset_manager: &AssetManager,
    ctx: &CommandContext,
    ticket_id: &str,
    file: &str,
    json: bool,
) -> Result<()> {
    // Verify ticket exists
    let mut pea = ctx.repo.get(ticket_id)?;

    // Add the asset
    let source_path = Path::new(file);
    if !source_path.exists() {
        anyhow::bail!("File not found: {}", file);
    }

    let asset_name = asset_manager.add_asset(ticket_id, source_path)?;

    // Update the pea's assets list
    if !pea.assets.contains(&asset_name) {
        pea.assets.push(asset_name.clone());
        pea.touch();
        ctx.repo.update(&pea)?;
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ticket_id": ticket_id,
                "asset": asset_name,
                "source": file,
            }))?
        );
    } else {
        println!("{} {}", "Added asset:".green(), asset_name);
        println!("  Ticket: {}", ticket_id.cyan());
        println!("  Source: {}", file);
    }

    Ok(())
}

fn handle_asset_list(asset_manager: &AssetManager, ticket_id: &str, json: bool) -> Result<()> {
    let assets = asset_manager.list_assets(ticket_id)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ticket_id": ticket_id,
                "assets": assets.iter().map(|a| serde_json::json!({
                    "filename": a.filename,
                    "size": a.size,
                    "size_formatted": a.size_string(),
                    "file_type": a.file_type(),
                    "path": a.path,
                })).collect::<Vec<_>>(),
                "count": assets.len(),
            }))?
        );
    } else {
        if assets.is_empty() {
            println!("No assets found for ticket {}", ticket_id.cyan());
        } else {
            println!(
                "{} {} asset{} for {}:",
                "Found".green(),
                assets.len(),
                if assets.len() == 1 { "" } else { "s" },
                ticket_id.cyan()
            );
            for asset in &assets {
                println!(
                    "  {} {} ({}, {})",
                    "•".cyan(),
                    asset.filename.bold(),
                    asset.size_string(),
                    asset.file_type().yellow()
                );
            }
        }
    }

    Ok(())
}

fn handle_asset_remove(
    asset_manager: &AssetManager,
    ctx: &CommandContext,
    ticket_id: &str,
    filename: &str,
    force: bool,
    json: bool,
) -> Result<()> {
    // Verify asset exists
    if !asset_manager.asset_exists(ticket_id, filename) {
        anyhow::bail!("Asset '{}' not found for ticket {}", filename, ticket_id);
    }

    // Confirm deletion if not forced
    if !force {
        print!(
            "Are you sure you want to remove asset '{}' from ticket {}? [y/N] ",
            filename.yellow(),
            ticket_id.cyan()
        );
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Remove the asset file
    asset_manager.remove_asset(ticket_id, filename)?;

    // Update the pea's assets list
    let mut pea = ctx.repo.get(ticket_id)?;
    if let Some(pos) = pea.assets.iter().position(|x| x == filename) {
        pea.assets.remove(pos);
        pea.touch();
        ctx.repo.update(&pea)?;
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ticket_id": ticket_id,
                "removed": filename,
            }))?
        );
    } else {
        println!("{} {}", "Removed asset:".red(), filename);
        println!("  Ticket: {}", ticket_id.cyan());
    }

    Ok(())
}

fn handle_asset_open(asset_manager: &AssetManager, ticket_id: &str, filename: &str) -> Result<()> {
    if !asset_manager.asset_exists(ticket_id, filename) {
        anyhow::bail!("Asset '{}' not found for ticket {}", filename, ticket_id);
    }

    let asset_path = asset_manager.get_asset_path(ticket_id, filename);

    // Open with platform-specific command
    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("cmd")
        .args(["/C", "start", "", asset_path.to_str().unwrap()])
        .status()?;

    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open")
        .arg(&asset_path)
        .status()?;

    #[cfg(target_os = "linux")]
    let status = std::process::Command::new("xdg-open")
        .arg(&asset_path)
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to open asset");
    }

    println!("{} {}", "Opened asset:".green(), filename);

    Ok(())
}
