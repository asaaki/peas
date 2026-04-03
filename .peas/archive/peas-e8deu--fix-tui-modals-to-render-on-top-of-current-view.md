+++
id = "peas-e8deu"
title = "Fix TUI modals to render on top of current view"
type = "task"
status = "completed"
priority = "normal"
created = "2026-01-30T17:27:39.680490Z"
updated = "2026-01-30T17:27:39.680490Z"
+++

## Problem
When opening modals (status, type, priority, tags, etc.) from the detail view, the TUI was switching back to the tree view and rendering the modal there. This was disorienting and caused visual artifacts.

## Root Cause
The draw function in ui.rs was only showing detail view for DetailView and EditBody modes. When entering modal modes, it would fall through to rendering the tree view.

## Solution
- Updated ui.rs draw function to detect all detail-related modes (DetailView, EditBody, StatusModal, PriorityModal, TypeModal, DeleteConfirm, ParentModal, BlockingModal, TagsModal, UrlModal)
- Changed logic to keep the current view (detail or list) as the base layer when modals are opened
- Modals now properly render on top of whatever view is active
- Added proper view routing for both Tickets and Memory views

## Files Changed
- src/tui/ui.rs: Complete restructure of draw function to handle modal overlays correctly

## Result
- Modals appear cleanly on top of the detail view
- No more jarring view switching
- Works for both Tickets and Memory views
- Screen artifacts eliminated
