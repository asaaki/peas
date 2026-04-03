+++
id = "peas-jus7w"
title = "Batch command support"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-up8r0"
created = "2026-01-18T17:52:21.823288800Z"
updated = "2026-01-18T18:26:51.598243500Z"
+++

Add 'peas batch create' command to create multiple tickets at once under a single parent. Reduces CLI overhead when adding many related tasks.

Example usage:
peas batch create --parent peas-xyz --type task << EOF
Title 1
Title 2
Title 3
EOF

Should support:
- Reading titles from stdin (one per line)
- Common parent for all items
- Common type for all items
- Optional common tags
