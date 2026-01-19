+++
id = "peas-nry73"
title = "Add interactive property editing in metadata pane"
type = "task"
status = "completed"
priority = "high"
created = "2026-01-19T21:25:00Z"
updated = "2026-01-19T21:25:00Z"
+++

## Description

When the Metadata pane is focused in detail view, add a row marker that allows navigating through editable properties (type, status, priority, tags) with ↓/↑ keys. Pressing Enter on a property opens the corresponding modal for editing.

## Implementation

### Added Features

1. **Row marker navigation in Metadata pane**
   - Added `metadata_selection` field to App struct (tracks selected property: 0=type, 1=status, 2=priority, 3=tags)
   - ↓/↑ keys navigate up/down through editable properties
   - ">" marker shows which property is currently selected (only when Metadata pane is focused)

2. **Enter key opens modals**
   - Type → TypeModal
   - Status → StatusModal
   - Priority → PriorityModal
   - Tags → TagsModal (new)

3. **Tags field**
   - Always visible in metadata section (shows "(none)" when empty)
   - New TagsModal for editing comma-separated tags
   - Tags modal supports typing, backspace, Enter to save, Esc to cancel

4. **Tab cycling**
   - Tab key cycles through panes: Metadata → Body → Relations → Metadata
   - Selected row in Metadata pane is preserved when returning to it

## Usage

1. Enter detail view with Enter key
2. Use Tab to focus Metadata pane
3. Use ↓/↑ to navigate through properties (Type, Status, Priority, Tags)
4. Press Enter to open editing modal for selected property
5. For tags: type comma-separated values, press Enter to save

## Technical Details

- Modified `src/tui/app.rs`:
  - Added `metadata_selection: usize` field
  - Added `tags_input: String` field
  - Added `InputMode::TagsModal`
  - Implemented `open_tags_modal()` and `apply_tags_modal()` methods
  - Updated ↓/↑ handlers in DetailView mode for Metadata pane navigation
  - Updated Enter handler to open modals based on selection
  - Added TagsModal key handlers

- Modified `src/tui/ui.rs`:
  - Updated metadata rendering to show row markers ("> ") when focused
  - Tags field now always visible with "(none)" placeholder
  - Created `draw_tags_modal()` function
  - Added TagsModal to draw() match statement
  - Added mode indicator and help text for TagsModal

## Acceptance Criteria

- [x] Add metadata_selection field to App struct
- [x] Implement ↓/↑ navigation in Metadata pane when focused
- [x] Show row marker for selected property
- [x] Enter key opens appropriate modal
- [x] Create TagsModal for editing tags
- [x] Tags field always visible in metadata section
- [x] Tab cycling works with metadata selection preserved
