+++
id = "peas-0ktvu"
title = "Decompose TUI App god object (app.rs 2107 LOC)"
type = "chore"
status = "in-progress"
priority = "high"
created = "2026-01-20T10:25:04.413446300Z"
updated = "2026-01-20T11:40:00.343964500Z"
+++

# TUI app.rs Refactor - Phase 1 In Progress

## Goal
Reduce src/tui/app.rs from 2107 LOC to ~450 LOC by extracting into focused modules.

## Overall Plan (8 Phases)
1. ⏳ **Phase 1**: Extract event handlers (642 LOC → 15 files)
2. 📋 Phase 2: Extract state management (45 fields → 4 structs)
3. 📋 Phase 3: Extract data management (430 LOC → 4 files)
4. 📋 Phase 4: Extract modal operations (420 LOC → 9 files)
5. 📋 Phase 5: Extract content utilities (137 LOC → 3 files)
6. 📋 Phase 6: Move type definitions (93 LOC → models.rs)
7. 📋 Phase 7: Final app.rs refactor & integration
8. 📋 Phase 8: Update module exports

## Phase 1 Progress: Event Handlers

### ✅ Completed
- Created `src/tui/handlers/` directory
- Created `handlers/mod.rs` module file
- **Extracted normal_mode.rs (165 LOC)**
  - 30+ keybindings (q, ?, Tab, j/k, Enter, Space, s, p, t, etc.)
  - Navigation, modal opening, actions
  - Compiles successfully

### 🚧 Remaining in Phase 1 (14 handlers, ~480 LOC)
Extract these from run_app() match statement in app.rs:

1. **filter.rs** (~15 LOC, lines ~1676-1690)
   - Handle search input (Esc, Enter, character input)

2. **modal_status.rs** (~15 LOC, lines ~1690-1710)
   - Status modal navigation (Up/Down, Enter, Esc)

3. **modal_priority.rs** (~15 LOC, lines ~1711-1731)
   - Priority modal navigation (Up/Down, Enter, Esc)

4. **modal_type.rs** (~15 LOC, lines ~1732-1752)
   - Type modal navigation (Up/Down, Enter, Esc)

5. **modal_delete.rs** (~15 LOC, lines ~1753-1761)
   - Delete confirmation (y/n/Esc)

6. **modal_parent.rs** (~15 LOC, lines ~1762-1782)
   - Parent selection (Up/Down, Enter, Esc)

7. **modal_blocking.rs** (~25 LOC, lines ~1783-1810)
   - Blocking multi-select (Up/Down, Space, Enter, Esc)

8. **detail_view.rs** (~90 LOC, lines ~1811-1943)
   - Detail pane navigation (Tab, j/k, Enter, e, o, Esc)
   - Complex with multiple sub-panes

9. **modal_create.rs** (~35 LOC, lines ~1944-1993)
   - Ticket creation form (Tab, Enter, Esc, character input)

10. **modal_memory_create.rs** (~35 LOC, lines ~1994-2031)
    - Memory creation form (Tab, Enter, Esc, character input)

11. **edit_body.rs** (~25 LOC, lines ~2032-2054)
    - TextArea passthrough (Ctrl+S, Esc)

12. **modal_tags.rs** (~15 LOC, lines ~2055-2071)
    - Tags input (Enter, Esc, character input)

13. **modal_url.rs** (~20 LOC, lines ~2072-2095)
    - URL selection (Up/Down, Enter, Esc)

14. **Mouse handling** (already in run_app, lines ~1490-1527)
    - Keep in run_app() or extract to mouse.rs

### Next Steps for Phase 1
1. Extract each of the 14 remaining handlers into separate files
2. Update `handlers/mod.rs` to declare and re-export all handlers
3. Refactor `run_app()` to dispatch to handlers:
   ```rust
   match app.input_mode {
       InputMode::Normal => handlers::normal_mode::handle(&mut app, key, terminal)?,
       InputMode::Filter => handlers::filter::handle(&mut app, key)?,
       // ... etc
   }
   ```
4. Test all functionality in TUI
5. Commit Phase 1 completion

### Handler Function Signature Pattern
```rust
pub fn handle(
    app: &mut App,
    key: KeyEvent,
    terminal: Option<&mut Terminal<CrosstermBackend<io::Stdout>>>
) -> io::Result<bool> // Returns true to quit
```

## Reference Files
- **Plan**: C:\Users\asaaki\.claude\plans\functional-drifting-kahan.md
- **Current commit**: WIP Phase 1 - Normal mode extracted
- **Line numbers**: Based on original app.rs before extraction

## Testing Checklist (After Phase 1)
- [ ] Launch TUI: `cargo run -- tui`
- [ ] Test Normal mode navigation (j/k, arrows, page up/down)
- [ ] Test filtering (/)
- [ ] Test all modals (s, p, t, P, B, T, D)
- [ ] Test detail view (Enter, Tab, e)
- [ ] Test view switching (Tab in Normal mode)
- [ ] Test multi-select (Space) and bulk operations
- [ ] Test creation (c, n)
- [ ] Test undo (u)
- [ ] Test URL opening (o)

All behavior must remain identical - this is a pure refactor.
