+++
id = "peas-575av"
title = "Add memory deletion in TUI"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-i0vo8"
blocking = ["peas-myfdb"]
created = "2026-01-19T22:40:15.622116600Z"
updated = "2026-01-19T22:40:15.622116600Z"
+++

Add deletion functionality for memory items in TUI

**Trigger:** Press 'd' on selected memory item

**Behavior:**
- Show confirmation modal
- Delete file from .peas/memory/
- Refresh memory list
- Show success message

**Follow pattern:** Similar to ticket DeleteConfirm modal
