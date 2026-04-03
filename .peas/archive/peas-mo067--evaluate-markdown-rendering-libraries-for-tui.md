+++
id = "peas-mo067"
title = "Evaluate markdown rendering libraries for TUI"
type = "research"
status = "completed"
priority = "normal"
created = "2026-01-18T23:43:00.903192500Z"
updated = "2026-01-18T23:48:32.751263400Z"
+++

Evaluate options for rendering markdown in the TUI detail view.

## Options

### Option 1 - tui-markdown (TRYING FIRST)
- https://lib.rs/crates/tui-markdown
- Used in: https://lib.rs/crates/markdown-reader
- Screenshot of the rendering looks exactly like we want
- Doesn't support all md elements, but would be quite okay as a starter

### Option 2 - ratatui-toolkit
- https://lib.rs/crates/ratatui-toolkit
- Offers quite some interesting components beyond markdown
- Seems to depend on ratatui 0.29 though (we're on 0.30)

### Option 3 - rat-salsa / rat-markdown
- https://lib.rs/crates/rat-markdown
- Seems to be a complete overhaul of how to use ratatui
- Maybe a bit too immature for now

### Option 4 - ratskin
- https://lib.rs/crates/ratskin
- Seems to wrap termimad for ratatui
- Might play nicer with rendering into a ratatui area than using termimad directly

## Current State
Currently using termimad directly, which requires rendering after ratatui's frame and doesn't integrate cleanly with ratatui's buffer system.
