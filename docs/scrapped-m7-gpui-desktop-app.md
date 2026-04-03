# Scrapped: M7 - Desktop GUI Application (GPUI)

**Status:** Won't do
**Scrapped:** 2026-04-03
**Original ticket:** peas-64ctb (+ 10 epics, 85 tasks)

## What was planned

A GPU-accelerated native desktop GUI for Peas using GPUI (Zed's UI framework),
launched via `peas ui`. The plan covered 10 epics:

1. **E7.1** GPUI Foundation and Setup
2. **E7.2** Core GUI Architecture and State Management
3. **E7.3** Main Window and Navigation
4. **E7.4** Ticket Tree View Component
5. **E7.5** Detail View and Editor Components
6. **E7.6** Modals and Dialogs System
7. **E7.7** Memory View and Management
8. **E7.8** Search and Filtering System
9. **E7.9** Theming and Polish
10. **E7.10** Advanced Features and Enhancements

The architecture mirrored the TUI's state machine design, with additions for mouse
interaction, animations, drag-and-drop, resizable panes, and a relationship graph
visualization.

**Tech stack:** gpui 0.2.2, gpui-component 0.5.0, gpui-animation, gpui-router,
gpui-tokio-bridge, gpui-ui-kit, plus optional allui, gpui-terminal, gpui-d3rs.

## Why it was scrapped

- **GPUI is pre-1.0 and tightly coupled to Zed.** Documentation outside the Zed
  codebase is sparse. Pinning to a moving target means constant churn for little
  user benefit.
- **The TUI already covers the interactive use case.** Peas is a flat-file issue
  tracker; a GPU-accelerated desktop app is over-engineered for the problem.
- **Massive scope relative to the rest of the project.** 85+ tasks and 10 epics
  would roughly double the codebase for a feature that doesn't serve the core
  audience (CLI users, AI agents).
- **Maintenance burden.** Cross-platform rendering, GPUI API breakage, and a
  second full UI layer would consume time better spent stabilizing the CLI, TUI,
  and GraphQL interfaces.

## If revisited in the future

- Reassess GPUI maturity (post-1.0 with stable API and docs).
- Consider lighter alternatives: a web UI served from `peas serve` would reuse
  the existing GraphQL layer with zero new native dependencies.
- Evaluate actual user demand before committing to a second UI framework.
