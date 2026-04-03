+++
id = "peas-6hiid"
title = "Edit ticket in $EDITOR (e key)"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-xacva"
created = "2026-01-18T19:20:32.034573900Z"
updated = "2026-01-18T19:39:26.385521700Z"
+++

Add 'e' hotkey to open the selected ticket's markdown file in $EDITOR.

Implementation:
- Get the file path for the selected ticket
- Spawn $EDITOR (or fallback to vim/nano)
- Wait for editor to close
- Refresh the ticket list

Note: This requires temporarily leaving the TUI alternate screen.
