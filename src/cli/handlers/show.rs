use anyhow::Result;
use colored::Colorize;

use super::CommandContext;
use super::utils::{format_priority, format_status};

pub fn handle_show(ctx: &CommandContext, id: String, json: bool) -> Result<()> {
    let pea = ctx.repo.get(&id)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&pea)?);
    } else {
        print_pea_with_refs(&pea, ctx);
    }
    Ok(())
}

fn print_pea_with_refs(pea: &peas::model::Pea, ctx: &CommandContext) {
    println!("{} {}", pea.id.cyan().bold(), pea.title.bold());
    println!("Type:     {}", format!("{}", pea.pea_type).blue());
    println!("Status:   {}", format_status(pea.status));
    println!("Priority: {}", format_priority(pea.priority));

    // Show parent with title if available
    if let Some(ref parent_id) = pea.parent {
        let parent_info = if let Ok(parent_pea) = ctx.repo.get(parent_id) {
            format!("{} ({})", parent_id.cyan(), parent_pea.title.dimmed())
        } else {
            parent_id.cyan().to_string()
        };
        println!("Parent:   {}", parent_info);
    }

    // Show blocking with titles if available
    if !pea.blocking.is_empty() {
        let blocking_info: Vec<String> = pea
            .blocking
            .iter()
            .map(|id| {
                if let Ok(blocked_pea) = ctx.repo.get(id) {
                    format!("{} ({})", id.cyan(), blocked_pea.title.dimmed())
                } else {
                    id.cyan().to_string()
                }
            })
            .collect();
        println!("Blocking: {}", blocking_info.join(", "));
    }

    if !pea.tags.is_empty() {
        println!("Tags:     {}", pea.tags.join(", ").magenta());
    }

    println!(
        "Created:  {}",
        pea.created.format("%Y-%m-%d %H:%M").to_string()
    );
    println!(
        "Updated:  {}",
        pea.updated.format("%Y-%m-%d %H:%M").to_string()
    );

    // Print body with resolved ticket references
    if !pea.body.is_empty() {
        let resolved_body = resolve_ticket_refs(&pea.body, &ctx.config.peas.prefix, ctx);
        println!("\n{}", resolved_body);
    }
}

fn resolve_ticket_refs(text: &str, prefix: &str, ctx: &CommandContext) -> String {
    use regex::Regex;

    // Build regex pattern for ticket IDs (e.g., peas-xxxxx)
    let pattern = format!(r"({}[a-z0-9]+)", regex::escape(prefix));
    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return text.to_string(),
    };

    let mut result = text.to_string();
    let mut replacements = Vec::new();

    // Find all ticket references and their titles
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            let id = m.as_str();
            if let Ok(referenced_pea) = ctx.repo.get(id) {
                replacements.push((id.to_string(), referenced_pea.title.clone()));
            }
        }
    }

    // Replace references with annotated versions
    for (id, title) in replacements {
        let annotated = format!("{} ({})", id.cyan(), title.dimmed());
        result = result.replace(&id, &annotated);
    }

    result
}
