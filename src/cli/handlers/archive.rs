use crate::cli::commands::{PeaPriorityArg, PeaStatusArg, PeaTypeArg};
use crate::model::Pea;
use anyhow::{Result, bail};
use chrono::{Duration, Utc};
use colored::Colorize;
use std::io::{self, Write};

use super::CommandContext;
use super::utils::record_undo_archive;

pub struct ArchiveParams {
    pub id: Option<String>,
    pub status: Option<PeaStatusArg>,
    pub r#type: Option<PeaTypeArg>,
    pub priority: Option<PeaPriorityArg>,
    pub tag: Option<String>,
    pub older_than: Option<String>,
    pub recursive: bool,
    pub keep_assets: bool,
    pub confirm: bool,
    pub dry_run: bool,
    pub json: bool,
}

pub fn handle_archive(ctx: &CommandContext, params: ArchiveParams) -> Result<()> {
    if let Some(ref id) = params.id {
        if params.recursive {
            // Collect the target + all descendants, then batch archive
            let mut peas = collect_descendants(ctx, id)?;
            // Add the root pea itself
            if let Ok(root) = ctx.repo.get(id) {
                peas.insert(0, root);
            }
            return handle_batch_archive_peas(ctx, peas, &params);
        }
        return handle_single_archive(ctx, id, params.keep_assets, params.json);
    }

    // Batch mode: at least one filter must be provided
    if params.status.is_none()
        && params.r#type.is_none()
        && params.priority.is_none()
        && params.tag.is_none()
        && params.older_than.is_none()
    {
        bail!(
            "Provide a pea ID or at least one filter (--status, --type, --priority, --tag, --older-than)"
        );
    }

    handle_batch_archive(ctx, &params)
}

fn handle_single_archive(
    ctx: &CommandContext,
    id: &str,
    keep_assets: bool,
    json: bool,
) -> Result<()> {
    let pea = ctx.repo.get(id)?;

    let asset_count = if ctx.asset_manager.has_assets(id) {
        ctx.asset_manager.list_assets(id)?.len()
    } else {
        0
    };

    let original_path = ctx.repo.find_file_by_id(id)?;
    let archive_path = ctx.repo.archive(id)?;
    record_undo_archive(ctx, id, &original_path, &archive_path);

    let mut assets_deleted = 0;
    if asset_count > 0 && !keep_assets {
        if !json {
            print!(
                "Also delete {} asset(s)? [Y/n] ",
                asset_count.to_string().yellow()
            );
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            if input.is_empty() || input.eq_ignore_ascii_case("y") {
                assets_deleted = ctx.asset_manager.cleanup_ticket_assets(id)?;
            }
        } else {
            assets_deleted = ctx.asset_manager.cleanup_ticket_assets(id)?;
        }
    }

    let filename = archive_path
        .file_name()
        .map(|f| f.to_string_lossy())
        .unwrap_or_default();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "archived",
                "id": id,
                "pea": pea,
                "assets_deleted": assets_deleted
            }))?
        );
    } else {
        println!("{} {} -> {}", "Archived".yellow(), id.cyan(), filename);
        if assets_deleted > 0 {
            println!(
                "  {} {} asset(s)",
                "Removed".red(),
                assets_deleted.to_string().yellow()
            );
        } else if asset_count > 0 {
            println!(
                "  {} {} asset(s) (use without --keep-assets to remove)",
                "Kept".yellow(),
                asset_count.to_string().yellow()
            );
        }
    }
    Ok(())
}

fn handle_batch_archive(ctx: &CommandContext, params: &ArchiveParams) -> Result<()> {
    let mut peas = ctx.repo.list()?;

    // Apply filters
    if let Some(s) = params.status {
        let filter_status = s.into();
        peas.retain(|p| p.status == filter_status);
    }
    if let Some(t) = params.r#type {
        let filter_type = t.into();
        peas.retain(|p| p.pea_type == filter_type);
    }
    if let Some(pr) = params.priority {
        let filter_priority = pr.into();
        peas.retain(|p| p.priority == filter_priority);
    }
    if let Some(ref tag) = params.tag {
        peas.retain(|p| p.tags.contains(tag));
    }
    if let Some(ref dur_str) = params.older_than {
        let duration = parse_duration(dur_str)?;
        let cutoff = Utc::now() - duration;
        peas.retain(|p| p.updated < cutoff);
    }

    handle_batch_archive_peas(ctx, peas, params)
}

fn handle_batch_archive_peas(
    ctx: &CommandContext,
    peas: Vec<Pea>,
    params: &ArchiveParams,
) -> Result<()> {
    if peas.is_empty() {
        if params.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "batch_archive",
                    "archived": [],
                    "count": 0
                }))?
            );
        } else {
            println!("No matching tickets to archive.");
        }
        return Ok(());
    }

    // Show preview
    if !params.json && !params.confirm {
        print_preview(&peas);
    }

    // Dry run: stop here
    if params.dry_run {
        if params.json {
            let ids: Vec<&str> = peas.iter().map(|p| p.id.as_str()).collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "batch_archive_dry_run",
                    "would_archive": ids,
                    "count": peas.len()
                }))?
            );
        } else {
            println!(
                "\n{} Would archive {} ticket(s).",
                "Dry run:".yellow(),
                peas.len()
            );
        }
        return Ok(());
    }

    // Confirm unless --confirm/-y was passed
    if !params.confirm && !params.json {
        print!(
            "\nArchive {} ticket(s)? [y/N] ",
            peas.len().to_string().yellow()
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Execute archival
    let mut archived_ids: Vec<String> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    for pea in &peas {
        match archive_one(ctx, &pea.id, params.keep_assets) {
            Ok(()) => archived_ids.push(pea.id.clone()),
            Err(e) => failed.push((pea.id.clone(), e.to_string())),
        }
    }

    if params.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "action": "batch_archive",
                "archived": archived_ids,
                "failed": failed.iter().map(|(id, err)| serde_json::json!({"id": id, "error": err})).collect::<Vec<_>>(),
                "count": archived_ids.len()
            }))?
        );
    } else {
        println!(
            "\n{} Archived {} ticket(s).",
            "Done.".green(),
            archived_ids.len().to_string().cyan()
        );
        if !failed.is_empty() {
            println!("{} {} ticket(s) failed:", "Warning:".red(), failed.len());
            for (id, err) in &failed {
                println!("  {} {}: {}", "✗".red(), id, err);
            }
        }
    }

    Ok(())
}

/// Recursively collect all descendants of a pea by walking the parent tree.
fn collect_descendants(ctx: &CommandContext, parent_id: &str) -> Result<Vec<Pea>> {
    let all_peas = ctx.repo.list()?;
    let mut result = Vec::new();
    let mut queue = vec![parent_id.to_string()];

    while let Some(current_id) = queue.pop() {
        for pea in &all_peas {
            if pea.parent.as_deref() == Some(&current_id) {
                queue.push(pea.id.clone());
                result.push(pea.clone());
            }
        }
    }

    Ok(result)
}

fn archive_one(ctx: &CommandContext, id: &str, keep_assets: bool) -> Result<()> {
    let original_path = ctx.repo.find_file_by_id(id)?;
    let archive_path = ctx.repo.archive(id)?;
    record_undo_archive(ctx, id, &original_path, &archive_path);

    if !keep_assets && ctx.asset_manager.has_assets(id) {
        let _ = ctx.asset_manager.cleanup_ticket_assets(id);
    }

    Ok(())
}

fn print_preview(peas: &[Pea]) {
    println!(
        "\nFound {} ticket(s) to archive:",
        peas.len().to_string().yellow()
    );
    for pea in peas {
        println!(
            "  {} [{}] {}",
            pea.id.cyan(),
            pea.pea_type.to_string().dimmed(),
            pea.title
        );
    }
}

fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        bail!("Empty duration string");
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let n: i64 = num_str.parse().map_err(|_| {
        anyhow::anyhow!(
            "Invalid duration '{}': expected format like 30d, 4w, 6m, 1y",
            s
        )
    })?;

    if n <= 0 {
        bail!("Duration must be positive: {}", s);
    }

    match unit {
        "d" => Ok(Duration::days(n)),
        "w" => Ok(Duration::weeks(n)),
        "m" => Ok(Duration::days(n * 30)),
        "y" => Ok(Duration::days(n * 365)),
        _ => bail!(
            "Unknown duration unit '{}': use d (days), w (weeks), m (months), y (years)",
            unit
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_days() {
        let d = parse_duration("30d").unwrap();
        assert_eq!(d.num_days(), 30);
    }

    #[test]
    fn test_parse_duration_weeks() {
        let d = parse_duration("4w").unwrap();
        assert_eq!(d.num_days(), 28);
    }

    #[test]
    fn test_parse_duration_months() {
        let d = parse_duration("6m").unwrap();
        assert_eq!(d.num_days(), 180);
    }

    #[test]
    fn test_parse_duration_years() {
        let d = parse_duration("1y").unwrap();
        assert_eq!(d.num_days(), 365);
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("0d").is_err());
        assert!(parse_duration("-5d").is_err());
        assert!(parse_duration("30x").is_err());
    }
}
