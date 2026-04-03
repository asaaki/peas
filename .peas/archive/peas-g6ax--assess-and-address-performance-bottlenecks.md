+++
id = "peas-g6ax"
title = "Assess and address performance bottlenecks"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-ep55"
created = "2026-01-18T14:51:19Z"
updated = "2026-01-18T14:51:19Z"
+++

## Assessment Complete

Review codebase for performance issues:

### Findings

1. **File I/O on every operation** (`src/storage/repository.rs:103-127`):
   - `list_in_path()` reads and parses all markdown files from disk on every call
   - Acceptable for expected use case (local CLI tool with typically <100 issues)

2. **Repeated `repo.list()` calls** (`src/graphql/schema.rs`):
   - GraphQL queries independently call `repo.list()`
   - If batched queries occur, same peas read multiple times

3. **No in-memory caching**:
   - Each operation hits filesystem
   - For CLI: desirable (always fresh data)
   - For TUI/GraphQL server: could be optimized if needed

### Conclusion: No immediate action needed

For the intended use case (simple, flat-file issue tracker like beans):
- Expected peas per project: 10-100
- Operations are infrequent (human-speed interaction)
- Simplicity is a design goal
- Fresh data on each read is a feature, not a bug

The current implementation correctly prioritizes **correctness and simplicity** over performance.

### Future Optimization Notes (if ever needed)

If performance becomes an issue with large datasets (1000+ peas):
1. Add request-scoped caching in GraphQL context
2. Implement lazy loading for pea bodies
3. Add file watcher for TUI to avoid polling
4. Consider SQLite backend as alternative storage

These optimizations would add complexity and are not recommended unless actual performance issues are observed.
