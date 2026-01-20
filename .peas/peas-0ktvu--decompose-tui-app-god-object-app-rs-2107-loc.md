+++
id = "peas-0ktvu"
title = "Decompose TUI App god object (app.rs 2107 LOC)"
type = "chore"
status = "completed"
priority = "high"
created = "2026-01-20T10:25:04.413446300Z"
updated = "2026-01-20T12:09:54.094693500Z"
+++

# TUI app.rs Refactor - COMPLETED ✅

## Final Results
**Reduced src/tui/app.rs from 2107 LOC to 1221 LOC (886 LOC = 42% reduction)**

Successfully extracted logic into 19 focused modules, improving maintainability and testability while preserving all functionality.

## Completed Phases

### ✅ Phase 1: Event Handlers (15 modules, 556 LOC extracted)
Created `src/tui/handlers/` with:
- **normal_mode.rs** (165 LOC) - Main navigation and keybindings
- **filter.rs** - Search input handling
- **modal_status.rs** - Status modal navigation
- **modal_priority.rs** - Priority modal navigation  
- **modal_type.rs** - Type modal navigation
- **modal_delete.rs** - Delete confirmation
- **modal_parent.rs** - Parent selection modal
- **modal_blocking.rs** - Blocking relationship multi-select
- **detail_view.rs** (90 LOC) - Detail pane navigation with sub-panes
- **modal_create.rs** (35 LOC) - Ticket creation form
- **modal_memory_create.rs** (35 LOC) - Memory creation form
- **edit_body.rs** - TextArea body editing
- **modal_tags.rs** - Tags input handling
- **modal_url.rs** - URL selection modal
- **mouse.rs** - Mouse click handling

Result: app.rs reduced from 2107 → 1551 LOC

### ✅ Phase 2: Tree Builder (185 LOC extracted)
Created `src/tui/tree_builder.rs`:
- `build_tree()` - Hierarchical ticket tree construction
- `build_page_table()` - Virtual pagination for large trees
- Moved TreeNode and PageInfo type definitions

Result: app.rs reduced from 1551 → 1380 LOC

### ✅ Phase 3: Modal Operations (185 LOC extracted)
Created `src/tui/modal_operations.rs`:
- Generic `apply_property_change()` function to eliminate duplication
- Specialized functions: `apply_status_change()`, `apply_priority_change()`, `apply_type_change()`, `apply_parent_change()`, `apply_blocking_change()`, `apply_tags_change()`
- Consolidated 6 nearly-identical modal apply methods

Result: app.rs reduced from 1380 → 1337 LOC

### ✅ Phase 4: Utility Modules (149 LOC extracted)
Created three focused utility modules:

**relations.rs** (63 LOC):
- `build_relations()` - Relationship graph building
- Finds parent, children, blocking, and blocked-by relationships

**url_utils.rs** (45 LOC):
- `extract_urls()` - URL extraction with smart punctuation handling
- Validates and deduplicates URLs from ticket bodies

**body_editor.rs** (41 LOC):
- `create_textarea()` - Initialize TextArea for body editing
- `save_body()` - Persist edited content with undo support

Result: app.rs reduced from 1337 → 1221 LOC

### ✅ Code Quality Improvements
- Removed all unused imports from app.rs and handlers
- All modules compile cleanly without warnings
- Consistent error handling patterns throughout
- Preserved all functionality - pure refactor

## Module Structure

```
src/tui/
├── app.rs (1221 LOC) - Core App struct and coordination
├── handlers/ (15 files) - Event handling
│   ├── normal_mode.rs
│   ├── filter.rs, detail_view.rs
│   ├── modal_*.rs (8 modal handlers)
│   ├── edit_body.rs, mouse.rs
│   └── mod.rs
├── modal_operations.rs - Generic modal apply logic
├── tree_builder.rs - Tree construction and pagination
├── relations.rs - Relationship graph building
├── url_utils.rs - URL extraction utilities
├── body_editor.rs - TextArea management
├── ui.rs - Rendering (next refactor target)
└── theme.rs - Color themes
```

## Commits
1. Phase 1: Extract 15 event handler modules (556 LOC)
2. Phase 2: Extract tree builder module (171 LOC)
3. Phase 3: Extract modal operations consolidation (43 LOC)
4. Phase 4: Extract relations, URL utils, and body editor (116 LOC)

## Impact
- **Maintainability**: Logic organized by responsibility
- **Testability**: Individual modules can be unit tested
- **Readability**: app.rs now focuses on coordination, not implementation
- **Performance**: No runtime overhead - zero-cost abstraction

## Next Steps
The TUI refactoring can continue with:
- **peas-cxj58**: Modularize ui.rs (1984 LOC) - HIGH priority
- **peas-oezjr**: Consolidate remaining modal logic
- **peas-5wzs3**: Generalize bulk operations further
