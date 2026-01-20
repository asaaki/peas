+++
id = "peas-orwsr"
title = "Implement atomic writes for pea updates"
type = "bug"
status = "completed"
priority = "high"
parent = "peas-w51zp"
created = "2026-01-20T15:11:12.013063Z"
updated = "2026-01-20T20:49:59.146146600Z"
+++

## Problem
Update writes new file then deletes old. Crash between operations can lose data:
1. Write new-file
2. **CRASH HERE** → both files exist or neither
3. Delete old-file

## Impact
- Potential data loss
- Corrupted state on crash
- No recovery mechanism

## Solution
Use atomic write pattern:
1. Write to temp file (.tmp)
2. fsync temp file
3. Rename temp → target (atomic on Unix)
4. Or use temp-file crate with persist()

## Files
- src/storage/repository.rs (update, create)
