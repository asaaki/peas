+++
id = "peas-xc13w"
title = "Implement memory list view in TUI"
type = "feature"
status = "completed"
priority = "high"
parent = "peas-i0vo8"
blocking = ["peas-myfdb"]
created = "2026-01-19T22:39:56.873376Z"
updated = "2026-01-19T22:52:46.624418500Z"
+++

Create memory list view rendering in src/tui/ui.rs

**Features:**
- Flat list of memory items (no tree structure)
- Display key, tags, updated timestamp
- Selection/navigation with j/k or arrows
- Filter/search support
- Pagination for large lists

**Follow pattern:** Similar to ticket tree rendering but simpler (no hierarchy)

**New function:** draw_memory_list() in ui.rs
