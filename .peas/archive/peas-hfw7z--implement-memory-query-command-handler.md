+++
id = "peas-hfw7z"
title = "Implement memory query command handler"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-hhrv0"
blocking = ["peas-2026i"]
created = "2026-01-19T22:39:19.564818200Z"
updated = "2026-01-19T22:48:47.725846Z"
+++

Implement handler for 'peas memory query' in src/main.rs

**Behavior:**
- Load config and create MemoryRepository
- Read memory by key
- Display formatted output or JSON
- Handle missing key gracefully

**Example:**
`peas memory query api-keys`
