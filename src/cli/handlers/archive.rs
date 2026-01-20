use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

use super::CommandContext;
use super::utils::record_undo_archive;

pub fn handle_archive(
    ctx: &CommandContext,
    id: String,
    keep_assets: bool,
    json: bool,
) -> Result<()> {
    let pea = ctx.repo.get(&id)?;

    // Check for assets before archiving
    let asset_count = if ctx.asset_manager.has_assets(&id) {
        ctx.asset_manager.list_assets(&id)?.len()
    } else {
        0
    };

    // Get original path before archive
    let original_path = ctx.repo.find_file_by_id(&id)?;

    let archive_path = ctx.repo.archive(&id)?;

    // Record undo operation
    record_undo_archive(ctx, &id, &original_path, &archive_path);

    // Handle asset cleanup
    let mut assets_deleted = 0;
    if asset_count > 0 && !keep_assets {
        if !json {
            print!(
                "Also delete {} asset(s)? [Y/n] ",
                asset_count.to_string().yellow()
            );
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            // Default to yes if user just presses enter
            if input.is_empty() || input.eq_ignore_ascii_case("y") {
                assets_deleted = ctx.asset_manager.cleanup_ticket_assets(&id)?;
            }
        } else {
            // In JSON mode, don't prompt - require explicit --keep-assets
            assets_deleted = ctx.asset_manager.cleanup_ticket_assets(&id)?;
        }
    }

    let filename = archive_path
        .file_name()
        .map(|f| f.to_string_lossy())
        .unwrap_or_default();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "archived",
                "id": id,
                "pea": pea,
                "assets_deleted": assets_deleted
            }))?
        );
    } else {
        println!("{} {} -> {}", "Archived".yellow(), id.cyan(), filename);
        if assets_deleted > 0 {
            println!(
                "  {} {} asset(s)",
                "Removed".red(),
                assets_deleted.to_string().yellow()
            );
        } else if asset_count > 0 {
            println!(
                "  {} {} asset(s) (use without --keep-assets to remove)",
                "Kept".yellow(),
                asset_count.to_string().yellow()
            );
        }
    }
    Ok(())
}
