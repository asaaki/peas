use crate::cli::commands::{PeaPriorityArg, PeaStatusArg, PeaTypeArg};
use anyhow::Result;
use colored::Colorize;

use super::CommandContext;
use super::utils::record_undo_update;

#[allow(clippy::too_many_arguments)]
pub fn handle_update(
    ctx: &CommandContext,
    id: String,
    title: Option<String>,
    r#type: Option<PeaTypeArg>,
    status: Option<PeaStatusArg>,
    priority: Option<PeaPriorityArg>,
    body: Option<String>,
    parent: Option<String>,
    add_tag: Vec<String>,
    remove_tag: Vec<String>,
    json: bool,
    dry_run: bool,
) -> Result<()> {
    let original = ctx.repo.get(&id)?;
    let mut pea = original.clone();

    if let Some(t) = title {
        pea.title = t;
    }
    if let Some(t) = r#type {
        pea.pea_type = t.into();
    }
    if let Some(s) = status {
        pea.status = s.into();
    }
    if let Some(p) = priority {
        pea.priority = p.into();
    }
    if let Some(b) = body {
        pea.body = b;
    }
    if let Some(p) = parent {
        pea.parent = if p.is_empty() { None } else { Some(p) };
    }
    for t in add_tag {
        if !pea.tags.contains(&t) {
            pea.tags.push(t);
        }
    }
    for t in remove_tag {
        pea.tags.retain(|x| x != &t);
    }

    if dry_run {
        // Build a list of changes
        let mut changes = Vec::new();
        if pea.title != original.title {
            changes.push(format!("title: '{}' -> '{}'", original.title, pea.title));
        }
        if pea.pea_type != original.pea_type {
            changes.push(format!("type: {} -> {}", original.pea_type, pea.pea_type));
        }
        if pea.status != original.status {
            changes.push(format!("status: {} -> {}", original.status, pea.status));
        }
        if pea.priority != original.priority {
            changes.push(format!(
                "priority: {} -> {}",
                original.priority, pea.priority
            ));
        }
        if pea.parent != original.parent {
            changes.push(format!("parent: {:?} -> {:?}", original.parent, pea.parent));
        }
        if pea.tags != original.tags {
            changes.push(format!("tags: {:?} -> {:?}", original.tags, pea.tags));
        }
        if pea.body != original.body {
            changes.push("body: [changed]".to_string());
        }

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "dry_run": true,
                    "id": id,
                    "changes": changes,
                    "before": original,
                    "after": pea
                }))?
            );
        } else if changes.is_empty() {
            println!("{} {} (no changes)", "Would update:".yellow(), id.cyan());
        } else {
            println!("{} {}", "Would update:".yellow(), id.cyan());
            for change in changes {
                println!("  {}", change);
            }
        }
        return Ok(());
    }

    // Record undo operation before update
    let old_path = ctx.repo.find_file_by_id(&pea.id)?;
    record_undo_update(ctx, &pea.id, &old_path);

    // NOTE: No touch() call - update() handles it internally now
    let path = ctx.repo.update(&mut pea)?;
    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy())
        .unwrap_or_default();

    if json {
        println!("{}", serde_json::to_string_pretty(&pea)?);
    } else {
        println!("{} {} {}", "Updated".green(), pea.id.cyan(), filename);
    }
    Ok(())
}
