use super::CommandContext;
use crate::config::{DATA_DIR, IdMode};
use anyhow::{Context, Result, bail};
use colored::Colorize;

pub fn handle_mv(ctx: &CommandContext, id: String, new_suffix: String, force: bool) -> Result<()> {
    let prefix = &ctx.config.peas.prefix;
    let id_length = ctx.config.peas.id_length;
    let id_mode = ctx.config.peas.id_mode;

    // Validate source ticket exists
    let pea = ctx
        .repo
        .get(&id)
        .with_context(|| format!("Ticket not found: {}", id))?;

    // Build the new ID
    let new_id = format!("{}{}", prefix, new_suffix);

    // Check if new ID already exists
    if ctx.repo.get(&new_id).is_ok() {
        bail!("Ticket with ID {} already exists", new_id);
    }

    // Validate suffix length
    if new_suffix.len() != id_length && !force {
        bail!(
            "Suffix length {} does not match configured id_length {}. Use --force to override.",
            new_suffix.len(),
            id_length
        );
    }

    // Validate ID mode
    let is_all_digits = new_suffix.chars().all(|c| c.is_ascii_digit());
    match id_mode {
        IdMode::Random if is_all_digits && !force => {
            bail!(
                "Suffix '{}' is all digits but id_mode is 'random'. Use --force to override.",
                new_suffix
            );
        }
        IdMode::Sequential if !is_all_digits && !force => {
            bail!(
                "Suffix '{}' contains non-digits but id_mode is 'sequential'. Use --force to override.",
                new_suffix
            );
        }
        _ => {}
    }

    // Show warnings if force was used
    if force {
        if new_suffix.len() != id_length {
            eprintln!(
                "{}: Suffix length {} differs from configured id_length {}",
                "warning".yellow().bold(),
                new_suffix.len(),
                id_length
            );
        }
        match id_mode {
            IdMode::Random if is_all_digits => {
                eprintln!(
                    "{}: Suffix '{}' is all digits but id_mode is 'random'",
                    "warning".yellow().bold(),
                    new_suffix
                );
            }
            IdMode::Sequential if !is_all_digits => {
                eprintln!(
                    "{}: Suffix '{}' contains non-digits but id_mode is 'sequential'",
                    "warning".yellow().bold(),
                    new_suffix
                );
            }
            _ => {}
        }
    }

    println!("Renaming {} → {}", id, new_id);

    // Find all tickets that reference this ID
    let all_peas = ctx.repo.list()?;
    let mut updated_parents = 0;
    let mut updated_blocking = 0;

    let data_dir = ctx.root.join(DATA_DIR);

    // Update references in other tickets
    for other_pea in &all_peas {
        if other_pea.id == id {
            continue; // Skip the ticket we're renaming
        }

        let mut needs_update = false;
        let mut updated_pea = other_pea.clone();

        // Check parent reference
        if updated_pea.parent.as_ref() == Some(&id) {
            updated_pea.parent = Some(new_id.clone());
            needs_update = true;
            updated_parents += 1;
        }

        // Check blocking references
        if updated_pea.blocking.contains(&id) {
            updated_pea.blocking = updated_pea
                .blocking
                .iter()
                .map(|b| if b == &id { new_id.clone() } else { b.clone() })
                .collect();
            needs_update = true;
            updated_blocking += 1;
        }

        if needs_update {
            ctx.repo.update(&mut updated_pea)?;
        }
    }

    // Now rename the ticket itself
    let mut renamed_pea = pea.clone();
    renamed_pea.id = new_id.clone();

    // Get old and new file paths
    let old_filename = format!(
        "{}--{}.md",
        id,
        slug::slugify(&pea.title)
            .chars()
            .take(50)
            .collect::<String>()
    );
    let new_filename = format!(
        "{}--{}.md",
        new_id,
        slug::slugify(&pea.title)
            .chars()
            .take(50)
            .collect::<String>()
    );

    let old_path = data_dir.join(&old_filename);
    let new_path = data_dir.join(&new_filename);

    // Write the updated ticket content to the new file
    let content = crate::storage::render_markdown_with_format(
        &renamed_pea,
        ctx.config.peas.frontmatter_format(),
    )?;
    std::fs::write(&new_path, content)?;

    // Remove the old file
    if old_path.exists() {
        std::fs::remove_file(&old_path)?;
    }

    // Update the .undo file if it references the old ID
    let undo_path = data_dir.join(".undo");
    if undo_path.exists() {
        let undo_content = std::fs::read_to_string(&undo_path)?;
        if undo_content.contains(&id) {
            let updated_undo = undo_content.replace(&id, &new_id);
            // Also update file paths in undo
            let updated_undo = updated_undo.replace(&old_filename, &new_filename);
            std::fs::write(&undo_path, updated_undo)?;
            println!("  Updated .undo file");
        }
    }

    println!("{} Renamed {} → {}", "✓".green(), id, new_id);
    if updated_parents > 0 {
        println!("  Updated {} parent reference(s)", updated_parents);
    }
    if updated_blocking > 0 {
        println!("  Updated {} blocking reference(s)", updated_blocking);
    }

    Ok(())
}
