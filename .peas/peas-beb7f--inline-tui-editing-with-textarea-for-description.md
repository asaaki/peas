+++
id = "peas-beb7f"
title = "Inline TUI editing with textarea for description"
type = "feature"
status = "in-progress"
priority = "normal"
parent = "peas-6592j"
created = "2026-01-19T16:11:18.839662500Z"
updated = "2026-01-19T21:19:56.452538500Z"
+++

Implement inline editing in TUI using ratatui and relevant dependencies.

## Requirements
- Edit mode opens directly in detail view, jumping to description pane
- Replace markdown rendering with simple textarea input
- Ctrl+S to save
- Ctrl+Arrow keys to switch between panes
- Research dependencies on lib.rs and crates.io

## Potentially useful dependency
- https://lib.rs/crates/terminput
