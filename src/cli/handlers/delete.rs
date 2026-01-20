use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

use super::CommandContext;
use super::utils::record_undo_delete;

pub fn handle_delete(ctx: &CommandContext, id: String, force: bool, json: bool) -> Result<()> {
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

    ctx.repo.delete(&id)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "deleted",
                "id": id
            }))?
        );
    } else {
        println!("{} {}", "Deleted".red(), id.cyan());
    }
    Ok(())
}
