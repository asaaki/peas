use anyhow::Result;

use super::CommandContext;

pub fn handle_import_beans(ctx: &CommandContext, path: String, dry_run: bool) -> Result<()> {
    let beans_path = std::path::Path::new(&path);

    let peas = crate::import_export::import_beans_directory(beans_path)?;

    if peas.is_empty() {
        println!("No beans files found to import in {}", path);
        return Ok(());
    }

    println!("Found {} beans to import:", peas.len());
    for pea in &peas {
        println!("  {} [{}] {}", pea.id, pea.pea_type, pea.title);
    }

    if dry_run {
        println!("\nDry run - no changes made.");
    } else {
        let mut imported = 0;
        let mut skipped = 0;
        for pea in peas {
            // Check if already exists
            if ctx.repo.find_file_by_id(&pea.id).is_ok() {
                println!("  Skipping {} (already exists)", pea.id);
                skipped += 1;
                continue;
            }
            match ctx.repo.create(&pea) {
                Ok(_) => imported += 1,
                Err(e) => eprintln!("  Failed to import {}: {}", pea.id, e),
            }
        }
        println!("\nImported {} peas, skipped {}", imported, skipped);
    }
    Ok(())
}
