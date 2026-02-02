use crate::cli::commands::{BulkAction, PeaPriorityArg, PeaStatusArg, PeaTypeArg};
use crate::model::{Pea, PeaStatus};
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Read};

use super::CommandContext;

/// Parameters for bulk create operation
struct BulkCreateParams {
    r#type: PeaTypeArg,
    parent: Option<String>,
    tag: Vec<String>,
    priority: Option<PeaPriorityArg>,
    status: Option<PeaStatusArg>,
    json: bool,
    dry_run: bool,
}

pub fn handle_bulk(ctx: &CommandContext, action: BulkAction) -> Result<()> {
    match action {
        BulkAction::Status { status, ids, json } => {
            let new_status: PeaStatus = status.into();
            bulk_update(
                ctx,
                &ids,
                json,
                |pea| {
                    pea.status = new_status;
                    true
                },
                |id| format!("{} {} -> {}", "Updated".green(), id.cyan(), new_status),
            )
        }
        BulkAction::Start { ids, json } => bulk_update(
            ctx,
            &ids,
            json,
            |pea| {
                pea.status = PeaStatus::InProgress;
                true
            },
            |id| format!("{} {}", "Started".green(), id.cyan()),
        ),
        BulkAction::Done { ids, json } => bulk_update(
            ctx,
            &ids,
            json,
            |pea| {
                pea.status = PeaStatus::Completed;
                true
            },
            |id| format!("{} {}", "Completed".green(), id.cyan()),
        ),
        BulkAction::Tag { tag, ids, json } => bulk_update_with_skip(
            ctx,
            &ids,
            json,
            |pea| {
                if !pea.tags.contains(&tag) {
                    pea.tags.push(tag.clone());
                    (true, None)
                } else {
                    (false, Some("already has tag".to_string()))
                }
            },
            |id| format!("{} {} +{}", "Tagged".green(), id.cyan(), tag.magenta()),
        ),
        BulkAction::Parent { parent, ids, json } => bulk_update(
            ctx,
            &ids,
            json,
            |pea| {
                pea.parent = Some(parent.clone());
                true
            },
            |id| {
                format!(
                    "{} {} -> parent: {}",
                    "Updated".green(),
                    id.cyan(),
                    parent.cyan()
                )
            },
        ),
        BulkAction::Create {
            r#type,
            parent,
            tag,
            priority,
            status,
            json,
            dry_run,
        } => handle_bulk_create(
            ctx,
            BulkCreateParams {
                r#type,
                parent,
                tag,
                priority,
                status,
                json,
                dry_run,
            },
        ),
    }
}

/// Generic bulk update handler for simple mutations
/// Uses validate-then-apply strategy: loads all peas and validates before writing any
fn bulk_update<F, M>(
    ctx: &CommandContext,
    ids: &[String],
    json: bool,
    mut mutate: F,
    message_fn: M,
) -> Result<()>
where
    F: FnMut(&mut Pea) -> bool,
    M: Fn(&str) -> String,
{
    // Phase 1: Load and validate all peas
    let mut peas_to_update: Vec<Pea> = Vec::new();
    let mut errors_list: Vec<serde_json::Value> = Vec::new();

    for id in ids {
        match ctx.repo.get(id) {
            Ok(mut pea) => {
                if mutate(&mut pea) {
                    // NOTE: No touch() call - update() handles it internally now
                    peas_to_update.push(pea);
                }
            }
            Err(e) => {
                if !json {
                    eprintln!("{} {}: {}", "Error loading".red(), id, e);
                }
                errors_list.push(serde_json::json!({"id": id, "error": e.to_string()}));
            }
        }
    }

    // If any pea failed to load, abort before writing
    if !errors_list.is_empty() {
        if !json {
            eprintln!(
                "\n{} Failed to load {} pea(s). Aborting bulk operation (no changes made).",
                "Error:".red(),
                errors_list.len()
            );
        }
        return Ok(());
    }

    // Phase 2: Apply all updates (now that we know all peas are valid)
    let mut updated_peas = Vec::new();

    for mut pea in peas_to_update {
        if let Err(e) = ctx.repo.update(&mut pea) {
            if !json {
                eprintln!("{} {}: {}", "Error updating".red(), pea.id, e);
            }
            errors_list.push(serde_json::json!({"id": pea.id, "error": e.to_string()}));
        } else {
            if !json {
                println!("{}", message_fn(&pea.id));
            }
            updated_peas.push(pea);
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "updated": updated_peas,
                "errors": errors_list
            }))?
        );
    } else if errors_list.is_empty() {
        println!(
            "\n{} {} peas",
            "Successfully updated".green(),
            updated_peas.len()
        );
    } else {
        println!(
            "\n{} {} peas, {} errors",
            "Partially completed:".yellow(),
            updated_peas.len(),
            errors_list.len()
        );
    }
    Ok(())
}

/// Bulk update with skip capability (for operations like tag that might be no-op)
/// Uses validate-then-apply strategy: loads all peas and validates before writing any
fn bulk_update_with_skip<F, M>(
    ctx: &CommandContext,
    ids: &[String],
    json: bool,
    mut mutate: F,
    message_fn: M,
) -> Result<()>
where
    F: FnMut(&mut Pea) -> (bool, Option<String>),
    M: Fn(&str) -> String,
{
    // Phase 1: Load and validate all peas
    let mut peas_to_update: Vec<Pea> = Vec::new();
    let mut errors_list: Vec<serde_json::Value> = Vec::new();
    let mut skipped = 0;

    for id in ids {
        match ctx.repo.get(id) {
            Ok(mut pea) => {
                let (should_update, skip_reason) = mutate(&mut pea);
                if should_update {
                    // NOTE: No touch() call - update() handles it internally now
                    peas_to_update.push(pea);
                } else {
                    if !json {
                        let reason = skip_reason.unwrap_or_else(|| "no change".to_string());
                        println!("{} {} ({})", "Skipped".yellow(), id.cyan(), reason);
                    }
                    skipped += 1;
                }
            }
            Err(e) => {
                if !json {
                    eprintln!("{} {}: {}", "Error loading".red(), id, e);
                }
                errors_list.push(serde_json::json!({"id": id, "error": e.to_string()}));
            }
        }
    }

    // If any pea failed to load, abort before writing
    if !errors_list.is_empty() {
        if !json {
            eprintln!(
                "\n{} Failed to load {} pea(s). Aborting bulk operation (no changes made).",
                "Error:".red(),
                errors_list.len()
            );
        }
        return Ok(());
    }

    // Phase 2: Apply all updates (now that we know all peas are valid)
    let mut updated_peas = Vec::new();

    for mut pea in peas_to_update {
        if let Err(e) = ctx.repo.update(&mut pea) {
            if !json {
                eprintln!("{} {}: {}", "Error updating".red(), pea.id, e);
            }
            errors_list.push(serde_json::json!({"id": pea.id, "error": e.to_string()}));
        } else {
            if !json {
                println!("{}", message_fn(&pea.id));
            }
            updated_peas.push(pea);
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "updated": updated_peas,
                "skipped": skipped,
                "errors": errors_list
            }))?
        );
    } else {
        println!(
            "\nTagged {} peas, {} skipped, {} errors",
            updated_peas.len(),
            skipped,
            errors_list.len()
        );
    }
    Ok(())
}

/// Handle bulk create from stdin
fn handle_bulk_create(ctx: &CommandContext, params: BulkCreateParams) -> Result<()> {
    // Read titles from stdin, one per line
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let titles: Vec<_> = input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if titles.is_empty() {
        if params.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "created": [],
                    "errors": [],
                    "message": "No titles provided on stdin"
                }))?
            );
        } else {
            println!("No titles provided. Provide one title per line on stdin.");
        }
        return Ok(());
    }

    let pea_type = params.r#type.into();
    let pea_status: Option<PeaStatus> = params.status.map(|s: PeaStatusArg| s.into());
    let pea_priority = params.priority.map(|p: PeaPriorityArg| p.into());

    // Dry-run mode: just show what would be created
    if params.dry_run {
        let mut would_create = Vec::new();
        for title in &titles {
            let id = ctx.repo.generate_id()?;
            let mut pea = Pea::new(id, title.to_string(), pea_type);

            if let Some(ref p) = params.parent {
                pea = pea.with_parent(Some(p.clone()));
            }
            if !params.tag.is_empty() {
                pea = pea.with_tags(params.tag.clone());
            }
            if let Some(s) = pea_status {
                pea = pea.with_status(s);
            }
            if let Some(p) = pea_priority {
                pea = pea.with_priority(p);
            }

            if !params.json {
                println!(
                    "{} {} [{}] {}",
                    "Would create:".yellow(),
                    pea.id.cyan(),
                    format!("{}", pea.pea_type).blue(),
                    pea.title
                );
            }
            would_create.push(pea);
        }

        if params.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "dry_run": true,
                    "would_create": would_create
                }))?
            );
        } else {
            println!("\n{} {} peas", "Would create:".yellow(), would_create.len());
        }
        return Ok(());
    }

    let mut created_peas = Vec::new();
    let mut errors_list: Vec<serde_json::Value> = Vec::new();

    for title in titles {
        let id = ctx.repo.generate_id()?;
        let mut pea = Pea::new(id, title.to_string(), pea_type);

        if let Some(ref p) = params.parent {
            pea = pea.with_parent(Some(p.clone()));
        }
        if !params.tag.is_empty() {
            pea = pea.with_tags(params.tag.clone());
        }
        if let Some(s) = pea_status {
            pea = pea.with_status(s);
        }
        if let Some(p) = pea_priority {
            pea = pea.with_priority(p);
        }

        match ctx.repo.create(&pea) {
            Ok(path) => {
                let filename = path
                    .file_name()
                    .map(|f| f.to_string_lossy())
                    .unwrap_or_default();
                if !params.json {
                    println!("{} {} {}", "Created".green(), pea.id.cyan(), filename);
                }
                created_peas.push(pea);
            }
            Err(e) => {
                if !params.json {
                    eprintln!("{} '{}': {}", "Error".red(), title, e);
                }
                errors_list.push(serde_json::json!({
                    "title": title,
                    "error": e.to_string()
                }));
            }
        }
    }

    if params.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "created": created_peas,
                "errors": errors_list
            }))?
        );
    } else {
        println!(
            "\nCreated {} peas, {} errors",
            created_peas.len(),
            errors_list.len()
        );
    }

    Ok(())
}
