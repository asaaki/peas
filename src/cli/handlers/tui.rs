use anyhow::Result;

use super::CommandContext;

pub fn handle_tui(ctx: CommandContext) -> Result<()> {
    crate::tui::run_tui(ctx.config, ctx.root)?;
    Ok(())
}
