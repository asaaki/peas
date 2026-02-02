+++
id = "peas-56gzm"
title = "Update TUI help screen with memory keybindings"
type = "task"
status = "completed"
priority = "low"
parent = "peas-i0vo8"
blocking = ["peas-575av", "peas-oxo19", "peas-jr51d", "peas-dxyxp"]
created = "2026-01-19T22:40:27.961387600Z"
updated = "2026-01-19T22:52:46.952790100Z"
+++

Update help screen (? key) to document memory view keybindings

**New keybindings:**
- Tab: Switch between Tickets and Memory views
- (In Memory view) n: New memory
- (In Memory view) d: Delete memory
- (In Memory view) e: Edit memory
- (In Memory view) /: Filter memories
- (In Memory view) Enter: View details

**Update:** src/tui/mod.rs documentation and help rendering in ui.rs
