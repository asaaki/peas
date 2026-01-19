+++
id = "peas-td52k"
title = "Refactor to 3-layer architecture: tree/page-table/render"
type = "feature"
status = "todo"
priority = "normal"
created = "2026-01-19T19:30:40.024596200Z"
updated = "2026-01-19T19:30:40.024596200Z"
+++

Redesign the TUI rendering system to use a clean 3-layer architecture that separates concerns and makes the code more maintainable and extensible.

## Architecture Layers

### Layer 1: Ticket Tree (Render-Agnostic)
- Pure data structure representing the complete ticket hierarchy
- Contains all relationships, dependencies, and ordering logic
- Completely agnostic to rendering - could support multiple output formats (TUI, PDF, HTML, etc.)
- Built from filtered_peas using hierarchical relationships
- Contains TreeNode with depth, parent_lines, is_last, etc.

### Layer 2: Page Table (Viewport-Aware)
- Takes the ticket tree and chunks it into pages based on available height
- Knows about current terminal height and recalculates when it changes
- Handles parent context rows by inserting references to parent nodes
- Uses references to TreeNode from Layer 1 (no data duplication)
- Produces a structured representation of what should appear on each page
- Each page entry contains: item refs, parent context refs, start/end indices

### Layer 3: Rendering (Terminal Output)
- Relatively "dumb" layer that just draws the page table to the terminal
- Takes page data from Layer 2 and renders using ratatui
- Handles styling, colors, highlighting, selection indicators
- Should not contain pagination logic or tree traversal logic

## Benefits

1. **Separation of Concerns**: Each layer has a single, well-defined responsibility
2. **Testability**: Each layer can be tested independently
3. **Maintainability**: Changes to rendering don't affect tree logic
4. **Extensibility**: Easy to add new output formats (PDF, HTML) by creating new Layer 3 implementations
5. **Performance**: Page table only recalculates when height changes or tree changes

## Implementation Tasks

- [ ] Define clear boundaries between layers with proper types
- [ ] Refactor TreeNode to be pure data (Layer 1)
- [ ] Create PageTable struct that references TreeNode (Layer 2)
- [ ] Implement page table building with parent context injection
- [ ] Simplify draw_tree to just render from page table (Layer 3)
- [ ] Update navigation methods to work with page table
- [ ] Ensure page table rebuilds on height change or filter change
