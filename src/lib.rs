//! # Peas - A CLI-based, flat-file issue tracker
//!
//! Peas is a lightweight issue tracker that stores issues as markdown files with TOML frontmatter.
//! It provides a CLI interface for humans and a GraphQL API for AI agents and automation.
//!
//! ## Features
//!
//! - **Flat-file storage**: Issues stored as markdown files in a `.peas/` directory
//! - **GraphQL API**: Query and mutate issues programmatically
//! - **TUI**: Terminal user interface for interactive issue management
//! - **Hierarchical structure**: Support for milestones, epics, features, bugs, and tasks
//!
//! ## Quick Start
//!
//! ```bash
//! # Initialize a new peas project
//! peas init
//!
//! # Create an issue
//! peas create "Fix login bug" -t bug
//!
//! # List all issues
//! peas list
//!
//! # Start working on an issue
//! peas start <id>
//!
//! # Mark as complete
//! peas done <id>
//! ```
//!
//! ## Modules
//!
//! - [`cli`]: Command-line interface definitions
//! - [`config`]: Configuration loading and management
//! - [`error`]: Error types and result aliases
//! - [`graphql`]: GraphQL schema and resolvers
//! - [`model`]: Data models (Pea, PeaType, PeaStatus, etc.)
//! - [`storage`]: File-based storage and markdown parsing
//! - [`tui`]: Terminal user interface
//! - [`validation`]: Input validation utilities

/// Command-line interface definitions using clap.
pub mod cli;

/// Configuration loading and management.
///
/// Handles `.peas.toml` configuration files and project discovery.
pub mod config;

/// Error types and result aliases.
///
/// Defines `PeasError` enum and `Result<T>` type alias.
pub mod error;

/// GraphQL schema and resolvers.
///
/// Provides async-graphql schema for querying and mutating peas.
pub mod graphql;

/// Data models for peas.
///
/// Includes `Pea`, `PeaType`, `PeaStatus`, and `PeaPriority`.
pub mod model;

/// File-based storage layer.
///
/// Handles reading/writing peas as markdown files with TOML frontmatter.
pub mod storage;

/// Terminal user interface.
///
/// Interactive TUI built with ratatui for managing peas.
pub mod tui;

/// Input validation utilities.
///
/// Validates titles, bodies, IDs, and tags to prevent invalid data.
pub mod validation;

/// Import and export functionality.
///
/// Supports importing from and exporting to beans format.
pub mod import_export;

pub mod assets;
pub mod logging;
pub mod search;
/// Undo functionality for reverting operations.
///
/// Tracks the last mutation and allows undoing it.
pub mod undo;
