+++
id = "peas-g6xv"
title = "Clean up Rust use statements"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-ep55"
created = "2026-01-18T14:50:51Z"
updated = "2026-01-18T14:53:36Z"
+++

Consolidate and organize use statements across all source files:
- Group use statements together at top of files
- Merge imports from same crate (e.g., use std::{io, path::Path})
- Order: std, external crates, internal modules
- Remove any unused imports
