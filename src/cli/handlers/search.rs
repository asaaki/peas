use anyhow::Result;

use super::CommandContext;
use super::utils::print_pea_list;
use crate::search::SearchQuery;

pub fn handle_search(ctx: &CommandContext, query: String, json: bool) -> Result<()> {
    let peas = ctx.repo.list()?;

    // Parse search query (supports field-specific and regex)
    let search_query = match SearchQuery::parse(&query) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("Invalid search query: {}", e);
            eprintln!("Examples:");
            eprintln!("  peas search bug              # Simple search");
            eprintln!("  peas search title:critical   # Search in title field");
            eprintln!("  peas search tag:urgent       # Search in tags");
            eprintln!("  peas search regex:bug.*fix   # Regex search");
            eprintln!("  peas search title:regex:.*   # Regex in specific field");
            return Err(anyhow::anyhow!(e));
        }
    };

    let results: Vec<_> = peas
        .into_iter()
        .filter(|p| search_query.matches_pea(p))
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!("Found {} results for '{}':\n", results.len(), query);
        print_pea_list(&results);
    }
    Ok(())
}
