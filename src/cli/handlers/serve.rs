use crate::graphql::build_schema;
use anyhow::Result;

use super::CommandContext;

pub fn handle_serve(ctx: CommandContext, port: u16) -> Result<()> {
    let schema = build_schema(ctx.config, ctx.root);

    println!("Starting GraphQL server on http://localhost:{}", port);
    println!("GraphQL Playground: http://localhost:{}", port);

    tokio::runtime::Runtime::new()?.block_on(async { run_server(schema, port).await })?;
    Ok(())
}

async fn run_server(schema: crate::graphql::PeasSchema, port: u16) -> Result<()> {
    use async_graphql::http::GraphiQLSource;
    use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
    use axum::{
        Router,
        extract::Extension,
        response::{Html, IntoResponse},
        routing::get,
    };

    async fn graphql_handler(
        Extension(schema): Extension<crate::graphql::PeasSchema>,
        req: GraphQLRequest,
    ) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    async fn graphiql() -> impl IntoResponse {
        Html(GraphiQLSource::build().endpoint("/").finish())
    }

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
