+++
id = "peas-c8rg3"
title = "Implement memory list command handler"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-hhrv0"
blocking = ["peas-2026i"]
created = "2026-01-19T22:39:24.624593500Z"
updated = "2026-01-19T22:48:48.050355900Z"
+++

Implement handler for 'peas memory list' in src/main.rs

**Behavior:**
- Load config and create MemoryRepository
- List all memories or filter by tag
- Display as table or JSON
- Sort by updated timestamp (newest first)

**Example:**
`peas memory list --tag infrastructure`
