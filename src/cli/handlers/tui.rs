use anyhow::Result;

use super::CommandContext;

pub fn handle_tui(ctx: CommandContext) -> Result<()> {
    peas::tui::run_tui(ctx.config, ctx.root)?;
    Ok(())
}
