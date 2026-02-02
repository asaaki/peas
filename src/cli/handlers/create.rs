use crate::cli::commands::{PeaPriorityArg, PeaStatusArg, PeaTypeArg, TemplateArg};
use crate::model::Pea;
use anyhow::Result;
use colored::Colorize;

use super::CommandContext;
use super::utils::{record_undo_create, resolve_body};

#[allow(clippy::too_many_arguments)]
pub fn handle_create(
    ctx: &CommandContext,
    title: String,
    r#type: PeaTypeArg,
    status: Option<PeaStatusArg>,
    priority: Option<PeaPriorityArg>,
    body: Option<String>,
    body_file: Option<String>,
    parent: Option<String>,
    blocking: Vec<String>,
    tag: Vec<String>,
    template: Option<TemplateArg>,
    json: bool,
    dry_run: bool,
) -> Result<()> {
    let body_content = resolve_body(body, body_file)?;
    let id = ctx.repo.generate_id()?;

    // Apply template settings if specified, then allow CLI args to override
    let (pea_type, default_priority, default_status, default_tags, body_template) =
        if let Some(tmpl) = template {
            let settings = tmpl.settings();
            (
                settings.pea_type,
                settings.priority,
                settings.status,
                settings.tags,
                settings.body_template,
            )
        } else {
            (r#type.into(), None, None, vec![], None)
        };

    let mut pea = Pea::new(id, title, pea_type);

    // Apply template defaults first, then override with explicit CLI args
    if let Some(s) = status {
        pea = pea.with_status(s.into());
    } else if let Some(s) = default_status {
        pea = pea.with_status(s);
    }

    if let Some(p) = priority {
        pea = pea.with_priority(p.into());
    } else if let Some(p) = default_priority {
        pea = pea.with_priority(p);
    }

    // Merge template tags with CLI tags (CLI tags take precedence/add to)
    let mut all_tags: Vec<String> = default_tags;
    for t in tag {
        if !all_tags.contains(&t) {
            all_tags.push(t);
        }
    }
    if !all_tags.is_empty() {
        pea = pea.with_tags(all_tags);
    }

    if parent.is_some() {
        pea = pea.with_parent(parent);
    }
    if !blocking.is_empty() {
        pea = pea.with_blocking(blocking);
    }

    // Body: CLI body overrides template, template is fallback
    if let Some(b) = body_content {
        pea = pea.with_body(b);
    } else if let Some(bt) = body_template {
        pea = pea.with_body(bt.to_string());
    }

    if dry_run {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "dry_run": true,
                    "would_create": pea
                }))?
            );
        } else {
            println!(
                "{} {} [{}] {}",
                "Would create:".yellow(),
                pea.id.cyan(),
                format!("{}", pea.pea_type).blue(),
                pea.title
            );
        }
        return Ok(());
    }

    let path = ctx.repo.create(&pea)?;

    // Record undo operation
    record_undo_create(ctx, &pea.id, &path);

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy())
        .unwrap_or_default();

    if json {
        println!("{}", serde_json::to_string_pretty(&pea)?);
    } else {
        println!("{} {} {}", "Created".green(), pea.id.cyan(), filename);
    }
    Ok(())
}
