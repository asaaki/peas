+++
id = "peas-2zjjm"
title = "Implement memory delete command handler"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-hhrv0"
blocking = ["peas-2026i"]
created = "2026-01-19T22:39:38.880179900Z"
updated = "2026-01-19T22:48:48.718977300Z"
+++

Implement handler for 'peas memory delete' in src/main.rs

**Behavior:**
- Load config and create MemoryRepository
- Delete memory file
- Output success message or JSON
- Handle missing key gracefully

**Example:**
`peas memory delete api-keys`
