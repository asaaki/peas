+++
id = "peas-2026i"
title = "Add Memory CLI subcommand structure"
type = "feature"
status = "completed"
priority = "high"
parent = "peas-hhrv0"
blocking = ["peas-1uu22"]
created = "2026-01-19T22:39:08.305100300Z"
updated = "2026-01-19T22:48:47.061701400Z"
+++

Add Memory command and subcommands to src/cli/commands.rs

**Structure:**
```rust
Memory {
    #[command(subcommand)]
    action: MemoryAction,
}

enum MemoryAction {
    Save { key: String, content: String, tags: Vec<String> },
    Query { key: String, json: bool },
    List { tag: Option<String>, json: bool },
    Edit { key: String },
    Delete { key: String, json: bool },
}
```

**Add to:** Commands enum in src/cli/commands.rs
**Follow pattern:** Similar to Bulk command with subcommands
