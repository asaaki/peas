+++
id = "peas-bbjsq"
title = "Undo last operation"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-up8r0"
created = "2026-01-18T17:55:35.181025100Z"
updated = "2026-01-18T17:55:35.181025100Z"
+++

Add 'peas undo' command to revert the last mutation. Since peas is git-backed, this could leverage git to restore previous file state.

Considerations:
- Track last operation in a .peas/.last_op file
- Only undo mutations (create, update, delete, archive)
- Show what will be undone before confirming
