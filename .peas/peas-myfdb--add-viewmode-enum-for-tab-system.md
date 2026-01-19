+++
id = "peas-myfdb"
title = "Add ViewMode enum for tab system"
type = "feature"
status = "completed"
priority = "high"
parent = "peas-i0vo8"
blocking = ["peas-1uu22"]
created = "2026-01-19T22:39:49.842013700Z"
updated = "2026-01-19T22:52:46.269197300Z"
+++

Add ViewMode enum to App struct in src/tui/app.rs to support switching between views

**Structure:**
```rust
pub enum ViewMode {
    Tickets,  // Current tree view
    Memory,   // New memory list view
}
```

**Changes:**
- Add view_mode: ViewMode field to App struct
- Add switch_view() method to toggle between modes
- Update draw() to render based on view_mode
- Add Tab key handler to switch views

**Default:** ViewMode::Tickets
