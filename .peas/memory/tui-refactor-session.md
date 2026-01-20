+++
key = "tui-refactor-session"
tags = [
    "session",
    "tui",
    "refactor",
    "wip",
]
created = "2026-01-20T11:40:17.904722200Z"
updated = "2026-01-20T11:40:17.904722200Z"
+++

# TUI app.rs Refactor Session State

## Current Status
Working on ticket **peas-0ktvu**: Decompose TUI App god object (app.rs 2107 LOC)

### Completed Today
1. ✅ Extracted main.rs handlers (1888 LOC → 134 LOC) - DONE
   - Ticket peas-22n9f marked as completed
   - 24 handler modules created in cli/handlers/
   - All functionality tested and working

2. ✅ Started TUI app.rs refactor - Phase 1 IN PROGRESS
   - Created src/tui/handlers/ structure
   - Extracted normal_mode.rs (165 LOC)
   - Code compiles successfully
   - Committed as WIP

### Next Session: Continue Phase 1
Extract 14 remaining event handlers from app.rs run_app() function:

**Line references (original app.rs):**
- filter.rs: lines 1676-1690
- modal_status.rs: lines 1690-1710
- modal_priority.rs: lines 1711-1731
- modal_type.rs: lines 1732-1752
- modal_delete.rs: lines 1753-1761
- modal_parent.rs: lines 1762-1782
- modal_blocking.rs: lines 1783-1810
- detail_view.rs: lines 1811-1943
- modal_create.rs: lines 1944-1993
- modal_memory_create.rs: lines 1994-2031
- edit_body.rs: lines 2032-2054
- modal_tags.rs: lines 2055-2071
- modal_url.rs: lines 2072-2095
- Mouse handling: lines 1490-1527

**Pattern:**
Each handler: `pub fn handle(app: &mut App, key: KeyEvent, ...) -> io::Result<bool>`

**After Phase 1:**
- Update run_app() to dispatch to handlers
- Test all TUI functionality
- Commit Phase 1 completion

### Remaining Phases (2-8)
- Phase 2: State management grouping
- Phase 3: Data management extraction
- Phase 4: Modal operations
- Phase 5: Content utilities  
- Phase 6: Type definitions
- Phase 7: Final app.rs integration
- Phase 8: Module exports

## Key Files
- Plan: C:\Users\asaaki\.claude\plans\functional-drifting-kahan.md
- Ticket: peas-0ktvu (in-progress)
- Working dir: D:\Development\Shared\PROJECT_LAB\peas
- Current branch: main

## Quick Start Command
```bash
cd D:\Development\Shared\PROJECT_LAB\peas
cargo run -- show peas-0ktvu  # See full continuation plan
cargo run -- tui              # Test current functionality
```

Target: 2107 LOC → ~450 LOC (79% reduction)
