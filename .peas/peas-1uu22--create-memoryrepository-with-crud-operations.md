+++
id = "peas-1uu22"
title = "Create MemoryRepository with CRUD operations"
type = "feature"
status = "completed"
priority = "high"
parent = "peas-kgp6m"
blocking = ["peas-dw02t"]
created = "2026-01-19T22:39:01.220506800Z"
updated = "2026-01-19T22:48:46.737181200Z"
+++

Implement MemoryRepository in src/storage/memory_repository.rs

**Operations:**
- new(config, root) - initialize with .peas/memory/ path
- create(memory) -> Result<PathBuf> - save to file
- read(key) -> Result<Memory> - load from file
- update(memory) -> Result<()> - update existing file
- delete(key) -> Result<()> - remove file
- list(tag_filter: Option<String>) -> Result<Vec<Memory>> - list all/filtered
- search(query) -> Result<Vec<Memory>> - full-text search

**File Format:**
- Filename: {key}.md (e.g., api-keys.md)
- TOML/YAML frontmatter + markdown body
- Follow pattern from PeaRepository

**Export from:** src/storage/mod.rs
