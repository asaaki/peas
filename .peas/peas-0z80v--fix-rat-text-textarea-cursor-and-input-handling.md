+++
id = "peas-0z80v"
title = "Fix rat-text TextArea cursor and input handling"
type = "task"
status = "completed"
priority = "normal"
created = "2026-01-30T17:27:29.339517Z"
updated = "2026-01-30T17:27:29.339517Z"
+++

## Problem
The rat-text TextArea widget in body edit mode was not showing a cursor and couldn't accept input.

## Solution
- Added proper TextArea configuration with .style() and .select_style() methods in ui_views.rs
- Added HasScreenCursor trait import to access screen_cursor() method
- Set frame cursor position using f.set_cursor_position() to make cursor visible
- Updated edit_body.rs to use rat_text::text_area::handle_events() function instead of HandleEvent trait
- Configured text style (foreground color) and selection style (highlighted text background)

## Files Changed
- src/tui/ui_views.rs (lines 872-887): TextArea rendering with proper styling
- src/tui/handlers/edit_body.rs (lines 23-27): Correct event handling

## Commits
- Replace tui-textarea with rat-text for crates.io compatibility
