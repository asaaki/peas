+++
id = "peas-6zje7"
title = "Add undo support to TUI"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-w51zp"
created = "2026-01-20T15:11:49.898382300Z"
updated = "2026-01-20T21:17:05.155539600Z"
+++

## Problem
Undo only works from CLI (`peas undo`), not from TUI. Users who make mistakes in TUI must exit and use CLI.

## Impact
- Inconsistent UX
- Disruptive workflow
- Easy to permanently delete by accident

## Solution
1. Add 'u' keybinding in TUI for undo
2. Show undo stack in status line
3. Support multiple undo levels (currently only last operation)
4. Add visual feedback for undo success/failure

## Files
- src/tui/handlers/normal_mode.rs
- src/undo.rs (extend for multi-level)
