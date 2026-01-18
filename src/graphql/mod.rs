//! GraphQL schema and resolvers for peas.
//!
//! Provides a GraphQL API for querying and mutating peas, designed for
//! integration with AI coding agents and automation tools.
//!
//! ## Usage
//!
//! ```bash
//! # Start the GraphQL server
//! peas serve --port 4000
//!
//! # Execute a query from CLI
//! peas query '{ stats { total byStatus { todo inProgress } } }'
//!
//! # Execute a mutation from CLI
//! peas mutate 'createPea(input: { title: "New task" }) { id }'
//! ```
//!
//! ## Schema
//!
//! - **Queries**: `pea`, `peas`, `search`, `children`, `stats`
//! - **Mutations**: `createPea`, `updatePea`, `setStatus`, `archivePea`, `deletePea`

mod schema;
mod types;

pub use schema::{PeasSchema, build_schema};
pub use types::*;
