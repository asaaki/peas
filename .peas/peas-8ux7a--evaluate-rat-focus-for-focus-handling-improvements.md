+++
id = "peas-8ux7a"
title = "Evaluate rat-focus for focus handling improvements"
type = "research"
status = "completed"
priority = "normal"
parent = "peas-g1gqo"
created = "2026-01-19T16:11:38.971205400Z"
updated = "2026-01-19T19:35:00.000000000Z"
+++

Evaluate https://lib.rs/crates/rat-focus for focus handling.

Determine if it can help with our current focus management patterns. If use cases are found, create follow-up work items.

## Evaluation Results

### What rat-focus Provides
- Decentralized focus state with FocusFlag per widget
- FocusBuilder for dynamic widget lists with next/prev navigation
- Tab/BackTab and mouse click event handling
- Container support for nested widgets

### Current peas Focus Management
Our focus handling is very simple and works well:
- `DetailPane` enum with 3 states: Metadata, Body, Relations
- `toggle_detail_pane()` cycles through panes with Tab key
- Only ~12 lines of code in `toggle_detail_pane()`
- No nested widgets or complex focus chains

### Recommendation: **Not Needed**

**Reasons:**
1. **Simplicity**: Our current focus logic is trivial and easy to understand
2. **No complexity**: We only have 3 focusable areas in detail view, no nesting
3. **Already working**: Tab navigation works perfectly for our use case
4. **Overhead**: rat-focus adds dependency and complexity for minimal benefit
5. **Rebuild cost**: FocusBuilder rebuilds widget list per event - unnecessary for our static 3-pane layout

**Conclusion:** rat-focus is designed for complex UIs with many dynamic widgets. Our simple 3-pane toggle doesn't justify the added complexity. Current implementation is optimal.

No follow-up work items needed.
