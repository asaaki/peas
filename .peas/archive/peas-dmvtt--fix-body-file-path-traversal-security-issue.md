+++
id = "peas-dmvtt"
title = "Fix body-file path traversal security issue"
type = "bug"
status = "completed"
priority = "high"
parent = "peas-w51zp"
created = "2026-01-20T15:10:57.300554300Z"
updated = "2026-01-20T20:49:59.522808400Z"
+++

## Problem
User can read any file via --body-file parameter:
`peas create --body-file ../../../../etc/passwd`

No path validation performed.

## Impact
- Information disclosure
- Security vulnerability
- Unintended file reads

## Solution
1. Validate body-file paths are within allowed directories
2. Reject absolute paths or paths with ../
3. Or document as intentional feature with warnings

## Files
- src/cli/handlers/create.rs
- src/cli/handlers/update.rs
