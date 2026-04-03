+++
id = "peas-oi78e"
title = "Add regex and field-specific search"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-w51zp"
created = "2026-01-20T15:11:43.019560400Z"
updated = "2026-01-20T21:24:17.795200500Z"
+++

## Problem
Search is too basic:
- Only substring matching
- No regex support
- Can't search specific fields (e.g., title:bug)
- No saved searches

## Impact
- Hard to find tickets in large backlogs
- Users resort to grep on files
- Poor UX for power users

## Solution
1. Add regex support with --regex flag
2. Field-specific syntax: title:*pattern*, body:*text*, tag:bug
3. Save searches as named queries in memory
4. Add search history in TUI

## Files
- src/cli/handlers/search.rs
- src/tui/handlers/filter.rs
