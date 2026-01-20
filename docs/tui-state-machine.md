# TUI State Machine Documentation

## Overview

The Peas TUI is a modal terminal user interface built with Ratatui. It operates as a state machine with clearly defined modes and transitions.

## Core State Components

### 1. Input Modes (`InputMode`)

The primary state machine is driven by `input_mode`:

```rust
pub enum InputMode {
    Normal,           // Browse and navigate
    Filter,           // Search/filter mode
    EditBody,         // Multi-line body editing
    ModalStatus,      // Status selection modal
    ModalPriority,    // Priority selection modal
    ModalType,        // Type selection modal
    ModalParent,      // Parent selection modal
    ModalBlocking,    // Blocking tickets selection modal
    ModalTags,        // Tag editing modal
    ModalDelete,      // Delete confirmation modal
    ModalUrl,         // URL selection modal
    CreatePea,        // Create new pea modal
    CreateMemory,     // Create new memory modal
}
```

### 2. View Modes (`ViewMode`)

The TUI supports multiple views:

```rust
pub enum ViewMode {
    Tickets,  // Main ticket list view
    Memory,   // Memory/session data view
}
```

### 3. Detail Panes (`DetailPane`)

When viewing a ticket, multiple detail panes are available:

```rust
pub enum DetailPane {
    Body,       // Ticket body/description
    Relations,  // Parent/blocking relationships
    Assets,     // Attached files/assets
    Metadata,   // Status, priority, type, tags
}
```

## State Machine Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         TUI Application                      │
│                                                               │
│  ViewMode: Tickets ←→ Memory (Tab key)                      │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                    Normal Mode (Default)                     │
│                                                               │
│  • Navigate list (j/k, arrows, PageUp/Down)                 │
│  • Select tickets (Space for multi-select)                  │
│  • Change detail pane (1-4 keys)                            │
│  • Scroll detail pane (h/l, Ctrl+U/D)                       │
│  • Open modals (s/p/t/P/b/d/u keys)                         │
│  • Create new (c/m keys)                                     │
│  • Filter (/), Help (?), Quit (q)                           │
└─────────────────────────────────────────────────────────────┘
    ↓ /        ↓ c         ↓ m          ↓ e          ↓ s/p/t/P/b/d
┌──────┐  ┌────────┐  ┌────────┐  ┌─────────┐  ┌──────────────┐
│Filter│  │CreatePea│  │CreateMemory│ │EditBody │  │Modal* (7)  │
│      │  │        │  │        │  │         │  │              │
│Input │  │3 fields│  │3 fields│  │Textarea │  │Esc returns   │
│text  │  │cycling │  │cycling │  │editor   │  │to Normal     │
│      │  │        │  │        │  │         │  │              │
│Esc/  │  │Enter   │  │Enter   │  │Esc      │  │Enter applies │
│Enter │  │creates │  │creates │  │saves    │  │selection     │
└──────┘  └────────┘  └────────┘  └─────────┘  └──────────────┘
   ↓          ↓          ↓           ↓              ↓
└──────────────────────  Normal Mode ──────────────────────────┘
```

## State Transitions

### From Normal Mode

| Key | Action | New Mode | Notes |
|-----|--------|----------|-------|
| `/` | Enter filter | Filter | Search/filter tickets |
| `?` | Toggle help | Normal | Help overlay shown |
| `c` | Create ticket | CreatePea | New ticket creation |
| `m` | Create memory | CreateMemory | New memory creation |
| `e` | Edit body | EditBody | Multi-line editor |
| `s` | Change status | ModalStatus | Status selection |
| `p` | Change priority | ModalPriority | Priority selection |
| `t` | Change type | ModalType | Type selection |
| `P` | Set parent | ModalParent | Parent selection |
| `b` | Set blocking | ModalBlocking | Multi-select blocking |
| `T` | Edit tags | ModalTags | Tag input |
| `d` | Delete ticket | ModalDelete | Confirmation required |
| `u` | Undo last op | Normal | No mode change |
| `Tab` | Switch view | Normal | Toggles Tickets ↔ Memory |
| `q` | Quit | - | Exit application |

### From Modal Modes

All modals return to Normal mode on:
- `Esc` - Cancel and discard changes
- `Enter` - Apply changes and close

Modal navigation:
- `j`/`Down` - Next option
- `k`/`Up` - Previous option
- Number keys - Direct selection (where applicable)

### From Filter Mode

| Key | Action | New Mode |
|-----|--------|----------|
| `Esc` | Clear filter | Normal |
| `Enter` | Apply filter | Normal |
| Typing | Update filter | Filter |

### From EditBody Mode

| Key | Action | New Mode |
|-----|--------|----------|
| `Esc` | Save and close | Normal |
| Edit keys | Modify text | EditBody |

## State Invariants

### Valid State Combinations

1. **input_mode = Normal**
   - Any detail_pane valid
   - Any view_mode valid
   - selected_index < filtered_peas.len() (or filtered_memories.len())

2. **input_mode = Filter**
   - view_mode unchanged
   - selected_index unchanged
   - search_query being edited

3. **input_mode = EditBody**
   - body_textarea must be Some()
   - A ticket must be selected
   - view_mode must be Tickets

4. **input_mode = Modal***
   - modal_selection < options.len()
   - previous_mode saved for restoration
   - Specific modal state populated (e.g., parent_candidates for ModalParent)

5. **input_mode = CreatePea/CreateMemory**
   - Previous mode saved
   - Create state fields populated (create_title, create_type, etc.)

### Invalid State Combinations

These should never occur:

1. `body_textarea.is_some()` when `input_mode != EditBody`
2. `modal_selection >= options.len()` in any modal
3. `selected_index >= tree_nodes.len()` in Tickets view
4. `input_mode == EditBody` when `view_mode == Memory`
5. Empty `parent_candidates` in `ModalParent` mode
6. Empty `blocking_candidates` in `ModalBlocking` mode

## Data Flow

### Ticket List Updates

```
User Action → Mutation → Repository → Cache Update → App Refresh → UI Update
                                                         ↓
                                              Build Tree → Filter → Render
```

1. **User Action**: Key press in Normal mode
2. **Mutation**: Create/Update/Delete/Archive operation
3. **Repository**: Disk write + cache update
4. **App Refresh**: `app.refresh()` reloads from cache
5. **Build Tree**: `build_tree()` creates hierarchical view
6. **Filter**: `apply_filter()` applies search query
7. **Render**: UI draws filtered_peas

### Filter Flow

```
User Types → search_query updated → apply_filter() → Filtered results rendered
```

1. Each keystroke updates `search_query`
2. `apply_filter()` called to update `filtered_peas`/`filtered_memories`
3. Supports simple substring, field-specific, and regex searches
4. Filter persists across mode changes

### Modal Flow

```
Open Modal → Load Options → User Selects → Apply Change → Close Modal
```

1. **Open**: Enter modal mode, save previous_mode
2. **Load**: Populate modal-specific data (candidates, options)
3. **Select**: User navigates and chooses option
4. **Apply**: Call repository mutation, update ticket
5. **Close**: Return to previous_mode, refresh display

## Concurrency Considerations

### Multi-Instance Safety

The TUI implements concurrent edit detection:

1. Each ticket has an `updated` timestamp
2. On update, current file timestamp checked against loaded timestamp
3. If timestamps differ, update rejected with error
4. User must reload and retry

This prevents:
- Lost updates from concurrent TUI instances
- Conflicts between TUI and CLI operations
- Race conditions in multi-user scenarios

### Cache Consistency

- Cache invalidated on any mutation
- Repository methods maintain cache consistency
- TUI refresh reloads from cache/disk

## Performance Characteristics

### O(1) Operations
- ID lookups (cache HashMap)
- Mode transitions
- Modal selection navigation

### O(n) Operations
- List rendering (n = filtered items)
- Tree building (n = all peas)
- Filter application (n = all peas)
- Search (n = all peas)

### Expensive Operations
- Full refresh: O(n) disk reads + parse
- Tree rebuild: O(n²) in worst case (deep nesting)
- Regex search: O(n × m) where m = pattern complexity

## Debug Assertions

The following invariants are checked in debug builds:

```rust
// In normal mode, selection must be valid
debug_assert!(
    app.input_mode != InputMode::Normal || 
    app.selected_index < app.tree_nodes.len()
);

// Modal selection must be in bounds
debug_assert!(
    !matches!(app.input_mode, InputMode::Modal*) ||
    app.modal_selection < options_count
);

// Body editor must exist when in EditBody mode
debug_assert!(
    app.input_mode != InputMode::EditBody ||
    app.body_textarea.is_some()
);
```

## Testing Strategy

### State Transition Tests

Test files in `tests/tui_tests.rs`:

- Modal open/close cycles
- View mode switching
- Filter mode entry/exit
- Detail pane switching
- Multi-selection state

### Boundary Tests

- Empty lists
- Single item lists
- Navigation bounds
- Modal option bounds

### Integration Tests

- Create → refresh → verify
- Update → refresh → verify
- Delete → refresh → verify

## Future Improvements

1. **State Machine Validation**
   - Runtime state validator
   - Automated invariant checking
   - State transition logging

2. **Performance**
   - Incremental tree updates
   - Virtual scrolling for large lists
   - Background cache warming

3. **Features**
   - Undo/redo for all operations
   - Clipboard integration
   - Custom key bindings
   - Macro recording

## See Also

- `src/tui/app.rs` - App struct and state management
- `src/tui/handlers/` - Input handlers for each mode
- `src/tui/ui_views.rs` - Rendering logic
- `tests/tui_tests.rs` - State machine tests
