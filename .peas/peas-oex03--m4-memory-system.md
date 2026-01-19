+++
id = "peas-oex03"
title = "M4: Memory System"
type = "milestone"
status = "todo"
priority = "critical"
tags = [
    "memory",
    "recall",
    "learning",
    "facts",
    "context",
    "scope",
]
created = "2026-01-19T22:38:23.881858500Z"
updated = "2026-01-19T22:42:58.860308200Z"
+++

Add a memory feature to peas for storing general project knowledge, learnings, facts, ideas, and context that doesn't fit into individual tickets.

**Scope:**
- CLI commands: save, query, edit, list, delete
- Storage in .peas/memory/ directory
- TUI integration with tab/view switching
- Markdown files with frontmatter (following existing patterns)

**Key Requirements:**
- Consistent with existing peas patterns (model, storage, CLI, TUI)
- Simple key-value storage with tags
- Searchable and editable
- Accessible from both CLI and TUI
