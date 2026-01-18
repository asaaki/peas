+++
id = "peas-w8ew0"
title = "Multi-select with Space for bulk operations"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-xacva"
created = "2026-01-18T19:20:07.624361800Z"
updated = "2026-01-18T19:20:07.624361800Z"
+++

Implement Space key for multi-selecting tickets in the TUI.

Current behavior: Space toggles status
New behavior: Space toggles selection for bulk operations

Selected tickets should be visually marked (e.g., with a checkbox or highlight). Bulk operations (status, priority, type changes) should apply to all selected tickets.

Related hotkeys that operate on selection:
- P: Change priority (modal)
- s: Change status (modal)  
- t: Change type (modal)
