+++
id = "peas-64ctb"
title = "M7: Desktop GUI Application (GPUI)"
type = "milestone"
status = "todo"
priority = "high"
created = "2026-01-22T13:37:19.495070400Z"
updated = "2026-01-22T13:37:19.495070400Z"
+++

# Desktop GUI Application with GPUI

Build a modern, GPU-accelerated desktop GUI application for Peas using the GPUI framework. This milestone focuses on creating a native desktop experience that leverages the complete feature set already implemented in the TUI, while adding GUI-specific enhancements like mouse interactions, animations, and visual polish.

## Overview

The GUI will provide an intuitive desktop interface for managing tickets and memories, accessible via the `peas ui` command. Built on GPUI (Zed's GPU-accelerated framework), it will deliver smooth performance, modern UI patterns, and cross-platform support (macOS, Linux, Windows).

## Strategic Approach

### Phase 1: Foundation (E7.1-E7.2)
Set up GPUI infrastructure and core architecture, adapting the proven TUI state machine design to GUI paradigms.

### Phase 2: Core Features (E7.3-E7.6)
Implement the main UI components: window layout, tree view, detail views, and modal system. This phase delivers feature parity with the TUI.

### Phase 3: Polish & Enhancement (E7.7-E7.10)
Add Memory management, search/filtering, theming, animations, and advanced GUI-specific features like drag-and-drop and graph visualizations.

## Success Criteria

- ✅ GPUI application launches via `peas ui` command
- ✅ Full feature parity with TUI (all workflows functional)
- ✅ Smooth 60fps performance with animations
- ✅ Intuitive mouse and keyboard navigation
- ✅ Cross-platform support (macOS, Linux, Windows)
- ✅ Light/dark theme support
- ✅ Real-time file watching and updates
- ✅ Professional UI polish and user experience

## Technology Stack

**Core Framework:**
- `gpui` (0.2.2) - GPU-accelerated UI framework
- `gpui-component` (0.5.0) - 60+ pre-built components

**Enhancement Libraries:**
- `gpui-animation` - Smooth transitions and effects
- `gpui-router` - Navigation and routing
- `gpui-tokio-bridge` - Async task integration
- `gpui-ui-kit` - Additional UI components

**Optional Enhancements:**
- `allui` - SwiftUI-like declarative patterns (if needed)
- `gpui-terminal` - Terminal emulator component (future)
- `gpui-d3rs` / `gpui-px` - Data visualization for relationship graphs

## Architecture Notes

The GUI architecture mirrors the TUI's proven state machine design:
- **GuiApp struct** - Central state container (analogous to TUI's App)
- **ViewMode enum** - Tickets vs Memory views
- **InputMode enum** - Modal states (Normal, DetailView, EditBody, etc.)
- **Repository Integration** - Direct use of PeaRepository and MemoryRepository
- **File Watcher** - Live updates on external changes
- **Undo System** - Full undo/redo support

Key differences from TUI:
- Mouse interactions (click, hover, right-click menus)
- Visual animations and transitions
- More flexible layouts (resizable panes, dock system)
- Richer components (dropdown selectors vs keyboard modals)
- Asset drag-and-drop support

## Epics Breakdown

1. **E7.1: GPUI Foundation** - Dependencies, basic app structure, CLI integration
2. **E7.2: Core Architecture** - State management, repositories, events, actions
3. **E7.3: Main Window** - Layout, navigation, sidebar, status bar, menus
4. **E7.4: Tree View** - Hierarchical ticket list with virtual scrolling
5. **E7.5: Detail View** - Multi-pane viewer/editor (metadata, body, relations, assets)
6. **E7.6: Modals System** - Status/priority/type/parent/blocking/tags/create/delete modals
7. **E7.7: Memory View** - List, detail, create, edit, delete memories
8. **E7.8: Search/Filter** - Real-time search with regex and field-specific queries
9. **E7.9: Theming** - Color schemes, icons, animations, polish
10. **E7.10: Advanced Features** - Undo/redo, clipboard, drag-drop, bulk ops, graph viz

## Dependencies

**Blockers:** None - all required infrastructure exists (repositories, models, TUI reference implementation)

**Builds On:**
- Existing repository system (PeaRepository, MemoryRepository)
- TUI architecture patterns (state machine, tree builder, relations)
- Core data models (Pea, Memory, SearchQuery)
- Configuration system (PeasConfig)

## Estimated Complexity

**Total Tasks:** 85+ work items across 10 epics

**Effort Distribution:**
- Critical (Foundation & Architecture): ~20 tasks
- High Priority (Core Features): ~35 tasks  
- Normal Priority (Polish & Enhancement): ~20 tasks
- Low Priority (Advanced Features): ~10 tasks

**Timeline Estimate:** This is a substantial milestone that builds on proven patterns. The TUI provides an excellent reference implementation, reducing uncertainty.

## Risks & Mitigations

**Risk 1: GPUI API Instability (pre-1.0)**
- *Mitigation:* Pin specific versions, monitor breaking changes, reference Zed source code

**Risk 2: Cross-platform Rendering Issues**
- *Mitigation:* Test early on all platforms, leverage GPUI's platform abstraction

**Risk 3: Performance with Large Ticket Counts**
- *Mitigation:* Virtual scrolling (already planned), incremental rendering, pagination

**Risk 4: Learning Curve for GPUI**
- *Mitigation:* Study Zed codebase, use gpui-component for common patterns, incremental implementation

## Future Enhancements (Post-M7)

- Multi-window support for side-by-side ticket comparison
- Split-pane views for concurrent editing
- Ticket timeline visualization
- Custom graph layouts using gpui-d3rs
- Embedded terminal with gpui-terminal
- Plugin system for custom views
- Mobile-responsive layouts (if GPUI adds mobile support)

---

**Command:** `peas ui`

**Repository:** Uses existing flat-file storage in `.peas/`

**Compatible With:** CLI, TUI, GraphQL server (all share same storage)
