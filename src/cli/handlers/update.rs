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
    add_blocks: Vec<String>,
    remove_blocks: Vec<String>,
    add_blocked_by: Vec<String>,
    remove_blocked_by: Vec<String>,
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
    // --add-blocks: this pea blocks the given IDs
    for b in &add_blocks {
        if !pea.blocking.contains(b) {
            pea.blocking.push(b.clone());
        }
    }
    for b in &remove_blocks {
        pea.blocking.retain(|x| x != b);
    }
    // --add-blocked-by: the given IDs block this pea (inverse: add this pea's ID to the other pea's blocking list)
    // We collect these to apply after dry-run check, since they modify other peas
    let has_blocked_by_changes = !add_blocked_by.is_empty() || !remove_blocked_by.is_empty();

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
        if pea.blocking != original.blocking {
            changes.push(format!(
                "blocking: {:?} -> {:?}",
                original.blocking, pea.blocking
            ));
        }
        if has_blocked_by_changes {
            for b in &add_blocked_by {
                changes.push(format!("blocked-by: add {} (will update {})", b, b));
            }
            for b in &remove_blocked_by {
                changes.push(format!("blocked-by: remove {} (will update {})", b, b));
            }
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

    // Apply blocked-by changes (modify other peas' blocking lists)
    if has_blocked_by_changes {
        for blocker_id in &add_blocked_by {
            let mut blocker = ctx.repo.get(blocker_id)?;
            if !blocker.blocking.contains(&id) {
                blocker.blocking.push(id.clone());
                ctx.repo.update(&mut blocker)?;
            }
        }
        for blocker_id in &remove_blocked_by {
            let mut blocker = ctx.repo.get(blocker_id)?;
            blocker.blocking.retain(|x| x != &id);
            ctx.repo.update(&mut blocker)?;
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&pea)?);
    } else {
        println!("{} {} {}", "Updated".green(), pea.id.cyan(), filename);
    }
    Ok(())
}
