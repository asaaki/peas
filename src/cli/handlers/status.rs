use crate::model::PeaStatus;
use anyhow::Result;
use colored::Colorize;

use super::CommandContext;
use super::utils::record_undo_update;

/// Generic status update handler
fn update_status(ctx: &CommandContext, id: &str, new_status: PeaStatus, json: bool) -> Result<()> {
    let mut pea = ctx.repo.get(id)?;

    // Record undo operation before update
    let old_path = ctx.repo.find_file_by_id(&pea.id)?;
    record_undo_update(ctx, &pea.id, &old_path);

    pea.status = new_status;
    // NOTE: No touch() call - update() handles it internally now
    ctx.repo.update(&mut pea)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&pea)?);
    } else {
        let status_str = match new_status {
            PeaStatus::InProgress => "in-progress".yellow(),
            PeaStatus::Completed => "completed".green(),
            _ => format!("{}", new_status).white(),
        };
        let action = match new_status {
            PeaStatus::InProgress => "Started".green(),
            PeaStatus::Completed => "Done".green(),
            _ => "Updated".green(),
        };
        println!("{} {} is now {}", action, pea.id.cyan(), status_str);
    }

    Ok(())
}

/// Handle start command (set status to InProgress)
pub fn handle_start(ctx: &CommandContext, id: String, json: bool) -> Result<()> {
    update_status(ctx, &id, PeaStatus::InProgress, json)
}

/// Handle done command (set status to Completed)
pub fn handle_done(ctx: &CommandContext, id: String, json: bool) -> Result<()> {
    update_status(ctx, &id, PeaStatus::Completed, json)
}
