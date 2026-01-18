+++
id = "peas-avbx7"
title = "Dry-run mode for mutations"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-up8r0"
created = "2026-01-18T17:55:16.594331600Z"
updated = "2026-01-18T17:55:16.594331600Z"
+++

Add --dry-run flag to create, update, batch commands to preview changes without applying them.

Example:
peas create --dry-run 'Test ticket' -t task
> Would create: peas-xxxxx [task] Test ticket
