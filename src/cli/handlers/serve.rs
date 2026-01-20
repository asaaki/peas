use anyhow::Result;
use peas::graphql::{build_schema, run_server};

use super::CommandContext;

pub fn handle_serve(ctx: CommandContext, port: u16) -> Result<()> {
    let schema = build_schema(ctx.config, ctx.root);

    println!("Starting GraphQL server on http://localhost:{}", port);
    println!("GraphQL Playground: http://localhost:{}", port);

    tokio::runtime::Runtime::new()?.block_on(async { run_server(schema, port).await })?;
    Ok(())
}
