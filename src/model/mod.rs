//! Data models for peas.
//!
//! This module defines the core data structures:
//!
//! - [`Pea`]: The main issue/task entity
//! - [`PeaType`]: Issue types (milestone, epic, feature, bug, task)
//! - [`PeaStatus`]: Workflow states (draft, todo, in-progress, completed, scrapped)
//! - [`PeaPriority`]: Priority levels (critical, high, normal, low, deferred)

mod pea;
mod types;

pub use pea::Pea;
pub use types::{PeaPriority, PeaStatus, PeaType};
