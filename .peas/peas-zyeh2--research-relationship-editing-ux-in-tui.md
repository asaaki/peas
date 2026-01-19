+++
id = "peas-zyeh2"
title = "Research relationship editing UX in TUI"
type = "research"
status = "completed"
priority = "normal"
parent = "peas-6592j"
created = "2026-01-19T16:11:19.221465300Z"
updated = "2026-01-19T20:00:00.000000000Z"
+++

Research the UX for editing relationships (parent, blocking) in the TUI.

## Current Implementation

### Existing Relationship Editing (Already Implemented!)
We already have modal-based relationship editing:

**Parent Modal (Key: 'p')**
- Shows all eligible parent candidates (milestone, epic, story, feature)
- Filtered: Can't select self, only container types
- Sorted by type hierarchy (milestone first, then epic, story, feature)
- Option "(none)" to clear parent
- Navigate with ↓/↑, select with Enter, cancel with Esc
- Current parent is pre-selected

**Blocking Modal (Key: 'b')**
- Multi-select modal for blocking relationships
- Shows all other tickets as candidates
- Space to toggle selection, Enter to confirm
- Can add multiple blocking relationships at once

**Relationships Pane in Detail View**
- Shows: Parent, Blocks, Children, BlockedBy
- Navigate with ↓/↑ when focused (Tab to switch panes)
- Enter to jump to selected relationship ticket
- Read-only display with type indicators

### Code Locations
- Parent modal: `src/tui/app.rs:843-893` (open_parent_modal, set_parent)
- Blocking modal: Similar implementation
- Relations pane: `src/tui/app.rs:536-598` (build_relations)
- UI rendering: `src/tui/ui.rs` (draw_parent_modal, etc.)

## Research Findings

### What We Have ✅
1. **Parent editing**: Modal with filtered candidates, "(none)" option
2. **Blocking editing**: Multi-select modal
3. **Relationship viewing**: Dedicated pane in detail view
4. **Navigation**: Jump to related tickets with Enter
5. **Undo support**: Parent changes are undoable

### What's Missing (Future Enhancements)

#### 1. Remove Blocking Relationships
**Current**: Can only add blocking relationships
**Needed**: Modal should show currently blocking tickets with toggle state
**Implementation**: Pre-populate `blocking_selected` from current `pea.blocking`

#### 2. Child Relationship Management
**Current**: Children shown read-only (derived from parent links)
**Consideration**: Children are automatically derived - no direct editing needed
**Note**: This is correct behavior - editing parent on child is the proper way

#### 3. BlockedBy Visibility
**Current**: BlockedBy shown in relations but can't be edited directly
**Note**: This is correct - it's the inverse of another ticket's Blocks list
**Proper UX**: To remove BlockedBy, edit the blocker ticket's Blocks list

#### 4. Quick Actions from Relations Pane
**Enhancement**: When focused on Relations pane, add key bindings:
- 'd' or 'x': Delete/remove this relationship
- 'e': Edit (open modal to change)
- Example: On "Blocks peas-abc12", press 'd' to remove from blocking list

#### 5. Visual Feedback
**Enhancement**: Show relationship state in tree view
- Subtle indicators for blocked items
- Parent chain breadcrumbs
- Already partially implemented with parent context!

## Recommendations

### Priority 1: Fix Blocking Modal to Support Removal
Currently can only add, not remove blocking relationships.

**Changes needed:**
1. Pre-populate `blocking_selected` from current `pea.blocking`
2. Show checkmarks for currently blocking tickets
3. Update `set_blocking()` to replace (not append) blocking list

### Priority 2: Quick Remove from Relations Pane
Add 'd' key when Relations pane focused:
- If on "Blocks X", remove X from blocking list
- If on "Parent X", clear parent (same as setting to "(none)")
- If on "BlockedBy X", show message: "Edit blocker's Blocks list"
- If on "Child X", show message: "Edit child's parent instead"

### Priority 3: Better Modal Titles
- "Select Parent" → "Set Parent (p=none, ↓/↑=navigate, Enter=select, Esc=cancel)"
- "Select Blocking" → "Set Blocking Relationships (Space=toggle, Enter=save, Esc=cancel)"

## Conclusion

The relationship editing UX is **already well-implemented** with modals for parent and blocking. The main gaps are:
1. Can't remove blocking relationships (only add)
2. No quick actions from Relations pane

These are minor enhancements that can be addressed in follow-up work items if needed. The core UX is solid and functional.
