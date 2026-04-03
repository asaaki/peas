+++
id = "peas-xx5x7"
title = "Expand test coverage for under-tested subsystems"
type = "task"
status = "completed"
priority = "normal"
created = "2026-04-03T12:25:36.304069105Z"
updated = "2026-04-03T13:10:04.093081126Z"
+++

154 tests for ~12,600 lines across 82 files leaves significant gaps. High-value areas to cover:

- **TUI interaction flows** — current tests are state-machine unit checks only; no rendering or multi-step interaction sequences
- **GraphQL mutation edge cases** — error paths, invalid inputs, concurrent access
- **Undo/redo** — multi-step undo sequences, undo after different operation types
- **Search** — regex edge cases, field-specific queries with special characters
- **Error paths** — corrupted markdown files, missing frontmatter fields, disk full scenarios
- **Concurrent file access** — what happens if two CLI invocations modify the same ticket

This doesn't need to happen all at once. Prioritize areas where bugs would cause data loss (storage, undo) over cosmetic issues (TUI rendering).
