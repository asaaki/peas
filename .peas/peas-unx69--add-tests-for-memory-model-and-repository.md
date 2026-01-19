+++
id = "peas-unx69"
title = "Add tests for Memory model and repository"
type = "task"
status = "todo"
priority = "normal"
parent = "peas-kgp6m"
blocking = ["peas-1uu22"]
created = "2026-01-19T22:40:41.816202900Z"
updated = "2026-01-19T22:40:41.816202900Z"
+++

Write unit and integration tests for memory functionality

**Test coverage:**
- Memory struct creation and serialization
- MemoryRepository CRUD operations
- File format validation (frontmatter + body)
- Tag filtering and search
- Error handling (missing files, invalid keys)

**Location:** tests/ directory or inline tests in src/storage/memory_repository.rs
