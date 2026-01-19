+++
id = "peas-f8gsq"
title = "Implement memory edit command handler"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-hhrv0"
blocking = ["peas-2026i"]
created = "2026-01-19T22:39:29.789493600Z"
updated = "2026-01-19T22:48:48.391255600Z"
+++

Implement handler for 'peas memory edit' in src/main.rs

**Behavior:**
- Load config and create MemoryRepository
- Open memory file in $EDITOR
- Validate after edit
- Save changes with updated timestamp

**Example:**
`peas memory edit api-keys`

**Follow pattern:** Similar to 'peas update' with editor
