use anyhow::Result;
use peas::undo::UndoManager;

use super::CommandContext;

pub fn handle_undo(ctx: &CommandContext, json: bool) -> Result<()> {
    let data_path = ctx.config.data_path(&ctx.root);
    let undo_manager = UndoManager::new(&data_path);

    match undo_manager.undo() {
        Ok(msg) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "success": true,
                        "message": msg
                    }))?
                );
            } else {
                println!("Undo: {}", msg);
            }
        }
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    }))?
                );
            } else {
                println!("Nothing to undo: {}", e);
            }
        }
    }
    Ok(())
}
