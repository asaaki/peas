use anyhow::Result;

use super::CommandContext;

pub fn handle_export_beans(ctx: &CommandContext, output: String) -> Result<()> {
    let output_path = std::path::Path::new(&output);

    std::fs::create_dir_all(output_path)?;

    let peas = ctx.repo.list()?;
    if peas.is_empty() {
        println!("No peas to export");
        return Ok(());
    }

    let mut exported = 0;
    for pea in &peas {
        let content = peas::import_export::export_to_beans(pea)?;
        let filename = peas::import_export::beans_filename(pea);
        let file_path = output_path.join(&filename);
        std::fs::write(&file_path, content)?;
        exported += 1;
    }

    println!("Exported {} peas to {}", exported, output);
    Ok(())
}
