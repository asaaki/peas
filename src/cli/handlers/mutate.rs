use crate::graphql::build_schema;
use anyhow::Result;

use super::CommandContext;

pub fn handle_mutate(
    ctx: CommandContext,
    mutation: String,
    variables: Option<String>,
) -> Result<()> {
    let schema = build_schema(ctx.config, ctx.root);

    let vars: async_graphql::Variables = if let Some(v) = variables {
        serde_json::from_str(&v)?
    } else {
        async_graphql::Variables::default()
    };

    // Auto-wrap in mutation { }
    let query = format!("mutation {{ {} }}", mutation);
    let request = async_graphql::Request::new(&query).variables(vars);
    let response = tokio::runtime::Runtime::new()?.block_on(schema.execute(request));

    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
