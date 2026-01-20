use anyhow::Result;

use super::CommandContext;
use super::utils::print_pea_list;

pub fn handle_search(ctx: &CommandContext, query: String, json: bool) -> Result<()> {
    let peas = ctx.repo.list()?;

    let query_lower = query.to_lowercase();
    let results: Vec<_> = peas
        .into_iter()
        .filter(|p| {
            p.title.to_lowercase().contains(&query_lower)
                || p.body.to_lowercase().contains(&query_lower)
                || p.id.to_lowercase().contains(&query_lower)
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!("Found {} results for '{}':\n", results.len(), query);
        print_pea_list(&results);
    }
    Ok(())
}
