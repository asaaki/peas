+++
id = "peas-m20iy"
title = "Copy ticket ID to clipboard (y key)"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-xacva"
created = "2026-01-18T19:21:00.614267500Z"
updated = "2026-01-18T19:21:00.614267500Z"
+++

Add 'y' hotkey to copy the selected ticket's ID to the system clipboard.

Implementation options:
- Use clipboard crate for cross-platform support
- Or shell out to pbcopy/xclip/clip depending on OS

Should show a brief message confirming the copy (e.g., 'Copied: peas-xxxxx').
