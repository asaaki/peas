+++
id = "peas-1bzk3"
title = "Implement memory save command handler"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-hhrv0"
blocking = ["peas-2026i"]
created = "2026-01-19T22:39:14.964250800Z"
updated = "2026-01-19T22:48:47.392515300Z"
+++

Implement handler for 'peas memory save' in src/main.rs

**Behavior:**
- Load config and create MemoryRepository
- Validate key (no special chars, valid filename)
- Create/update Memory struct
- Save to .peas/memory/{key}.md
- Output success message or JSON

**Example:**
`peas memory save api-keys 'API keys stored in 1Password' --tag infrastructure`
