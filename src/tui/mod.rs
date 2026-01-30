//! Terminal user interface for peas.
//!
//! An interactive TUI built with ratatui for managing peas from the terminal.
//!
//! ## Usage
//!
//! ```bash
//! peas tui
//! ```
//!
//! ## Keybindings
//!
//! - `↑/↓`: Navigate up/down
//! - `←/→`: Previous/next page
//! - `Tab`: Switch between Tickets/Memory views
//! - `/`: Search
//! - `Enter`: Open detail view
//! - `Space`: Multi-select toggle
//! - `c`: Create new ticket
//! - `s`: Change status
//! - `t`: Change type
//! - `P`: Change priority
//! - `e`: Edit in $EDITOR
//! - `r`: Refresh
//! - `u`: Undo last operation
//! - `?`: Help
//! - `q`: Quit

pub mod app;
mod body_editor;
mod handlers;
mod modal_operations;
mod relations;
pub mod theme;
mod tree_builder;
mod ui;
mod ui_modals;
mod ui_utils;
mod ui_views;
mod url_utils;

pub use app::run_tui;
