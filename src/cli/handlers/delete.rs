use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

use super::CommandContext;
use super::utils::record_undo_delete;

pub fn handle_delete(
    ctx: &CommandContext,
    id: String,
    force: bool,
    keep_assets: bool,
    json: bool,
) -> Result<()> {
    // Check for assets before confirmation
    let asset_count = if ctx.asset_manager.has_assets(&id) {
        ctx.asset_manager.list_assets(&id)?.len()
    } else {
        0
    };

    if !force && !json {
        print!("Delete {} permanently? [y/N] ", id.cyan());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Record undo operation before delete
    let file_path = ctx.repo.find_file_by_id(&id)?;
    record_undo_delete(ctx, &id, &file_path);

    // Delete the pea
    ctx.repo.delete(&id)?;

    // Handle asset cleanup
    let mut assets_deleted = 0;
    if asset_count > 0 && !keep_assets {
        if !force && !json {
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
        } else if force {
            // In force mode, automatically delete assets
            assets_deleted = ctx.asset_manager.cleanup_ticket_assets(&id)?;
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "deleted",
                "id": id,
                "assets_deleted": assets_deleted
            }))?
        );
    } else {
        println!("{} {}", "Deleted".red(), id.cyan());
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
