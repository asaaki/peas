---
# peas-g6xv
title: Clean up Rust use statements
status: completed
type: task
priority: normal
created_at: 2026-01-18T14:50:51Z
updated_at: 2026-01-18T14:53:36Z
parent: peas-ep55
---

Consolidate and organize use statements across all source files:
- Group use statements together at top of files
- Merge imports from same crate (e.g., use std::{io, path::Path})
- Order: std, external crates, internal modules
- Remove any unused imports