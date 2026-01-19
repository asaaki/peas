//! File-based storage layer for peas.
//!
//! Peas are stored as markdown files with YAML frontmatter in the `.peas/` directory.
//!
//! ## File Format
//!
//! ```markdown
//! ---
//! id: peas-1234
//! title: Fix login bug
//! type: bug
//! status: in-progress
//! priority: high
//! tags: [auth, urgent]
//! created: 2024-01-15T10:30:00Z
//! updated: 2024-01-15T14:20:00Z
//! ---
//!
//! Description of the bug and any additional notes.
//! ```
//!
//! ## Components
//!
//! - [`PeaRepository`]: CRUD operations for peas
//! - [`MemoryRepository`]: CRUD operations for memories
//! - [`parse_markdown`]: Parse a pea from markdown content
//! - [`render_markdown`]: Render a pea to markdown content

mod markdown;
mod memory_repository;
mod repository;

pub use markdown::{
    FrontmatterFormat, detect_format, parse_markdown, parse_markdown_memory,
    parse_markdown_with_format, render_markdown, render_markdown_memory,
    render_markdown_with_format,
};
pub use memory_repository::MemoryRepository;
pub use repository::PeaRepository;
