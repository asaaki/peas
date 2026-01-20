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
//! - `j/k` or `↓/↑`: Navigate list
//! - `Tab`: Cycle filters (All, Open, In Progress, etc.)
//! - `/`: Search
//! - `Enter/Space`: Toggle status
//! - `s`: Start (mark in-progress)
//! - `d`: Done (mark completed)
//! - `r`: Refresh
//! - `?`: Help
//! - `q`: Quit

mod app;
mod handlers;
pub mod theme;
mod ui;

pub use app::run_tui;
