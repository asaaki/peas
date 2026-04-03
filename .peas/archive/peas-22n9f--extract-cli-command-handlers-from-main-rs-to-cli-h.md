+++
id = "peas-22n9f"
title = "Extract CLI command handlers from main.rs to cli/handlers/ module"
type = "chore"
status = "completed"
priority = "high"
created = "2026-01-20T10:25:03.335299100Z"
updated = "2026-01-20T11:15:56.512694600Z"
+++

## Problem
Main.rs is 1888 LOC with all CLI command handlers embedded directly in the main match statement. This violates single responsibility principle and makes testing impossible.

## Solution
Extract each command handler into dedicated files in `src/cli/handlers/`:
- create.rs (~150 LOC)
- update.rs (~100 LOC)
- bulk.rs (~200 LOC)
- memory.rs (~100 LOC)
- search.rs, show.rs, list.rs, etc.

Implement command handler pattern/trait for consistency.

## Target
Reduce main.rs from 1888 to ~300 LOC (80% reduction).

## Benefits
- Testable command handlers
- Better separation of concerns
- Easier to add new commands
- Reduced cognitive load
