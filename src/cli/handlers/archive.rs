use anyhow::Result;
use colored::Colorize;

use super::CommandContext;
use super::utils::record_undo_archive;

pub fn handle_archive(ctx: &CommandContext, id: String, json: bool) -> Result<()> {
    let pea = ctx.repo.get(&id)?;

    // Get original path before archive
    let original_path = ctx.repo.find_file_by_id(&id)?;

    let archive_path = ctx.repo.archive(&id)?;

    // Record undo operation
    record_undo_archive(ctx, &id, &original_path, &archive_path);

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
                "pea": pea
            }))?
        );
    } else {
        println!("{} {} -> {}", "Archived".yellow(), id.cyan(), filename);
    }
    Ok(())
}
