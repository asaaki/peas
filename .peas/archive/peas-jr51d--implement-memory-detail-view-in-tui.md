+++
id = "peas-jr51d"
title = "Implement memory detail view in TUI"
type = "feature"
status = "completed"
priority = "high"
parent = "peas-i0vo8"
blocking = ["peas-xc13w"]
created = "2026-01-19T22:40:04.034187400Z"
updated = "2026-01-19T22:58:02.766462800Z"
+++

Create memory detail view when pressing Enter on a memory item

**Features:**
- Full-screen view showing memory content
- Display key, tags, timestamps
- Scrollable markdown body
- Edit mode (e key to enter EditBody mode)
- Return to list with Esc

**Follow pattern:** Similar to ticket DetailView mode but simpler (no relations pane)

**Update:** InputMode enum to handle memory detail/edit states
