+++
id = "peas-jxy81"
title = "Implement concurrent edit detection"
type = "feature"
status = "completed"
priority = "high"
parent = "peas-w51zp"
created = "2026-01-20T15:10:37.571817500Z"
updated = "2026-01-20T20:50:00.298851Z"
+++

## Problem
Multiple TUI instances can clobber each other's changes. File watcher exists but no conflict resolution.

## Impact
- Data loss when multiple users/instances edit same pea
- Silent overwrites

## Solution
Implement one of:
1. File locking mechanism (flock/LockFile)
2. Conflict detection on write (check timestamp)
3. Last-write-wins with warning

## Files
- src/storage/repository.rs
- src/tui/ handlers
