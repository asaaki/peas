+++
id = "peas-nvd0r"
title = "Add caching layer to repository"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-w51zp"
created = "2026-01-20T15:11:29.699098600Z"
updated = "2026-01-20T21:21:32.143169200Z"
+++

## Problem
Repository.list() called repeatedly without caching:
- TUI reloads full list on every refresh
- Search does full scan every keystroke
- No memoization of file lookups

## Impact
- Poor performance with large backlogs (1000+ peas)
- Unnecessary disk I/O
- Slow response times

## Solution
1. Add in-memory cache in PeaRepository
2. Invalidate on file watcher events
3. Use HashMap for O(1) ID lookups instead of linear search
4. Cache filtered results for common queries

## Files
- src/storage/repository.rs
- src/tui/app.rs (cache invalidation)
