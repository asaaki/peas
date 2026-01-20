+++
id = "peas-anmim"
title = "Add asset cleanup on pea deletion"
type = "bug"
status = "completed"
priority = "normal"
parent = "peas-w51zp"
created = "2026-01-20T15:12:13.150164600Z"
updated = "2026-01-20T20:50:01.027914400Z"
+++

## Problem
When a pea is deleted, its assets directory remains:
- .peas/assets/{ticket-id}/ not removed
- Orphaned files accumulate
- Wasted disk space

## Impact
- Disk bloat over time
- Confusing when browsing .peas/assets/
- No cleanup mechanism exists

## Solution
In delete/archive operations:
1. Check if .peas/assets/{id}/ exists
2. Prompt user: "Also delete N assets?"
3. Remove directory if confirmed
4. Add --keep-assets flag for retention

## Files
- src/cli/handlers/delete.rs
- src/cli/handlers/archive.rs
- src/assets.rs (add cleanup method)
