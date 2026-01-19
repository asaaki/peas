+++
id = "peas-oxo19"
title = "Add memory creation modal in TUI"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-i0vo8"
blocking = ["peas-myfdb"]
created = "2026-01-19T22:40:10.397141100Z"
updated = "2026-01-19T22:40:10.397141100Z"
+++

Add modal for creating new memory items in TUI

**Trigger:** Press 'n' in memory view (similar to ticket creation)

**Fields:**
- Key input (validated for filename safety)
- Tags input (comma-separated)
- Content textarea

**Behavior:**
- Validate key uniqueness
- Save to .peas/memory/
- Refresh memory list
- Show success message

**Follow pattern:** Similar to CreateModal for tickets
