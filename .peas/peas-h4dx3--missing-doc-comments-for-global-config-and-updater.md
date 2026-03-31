+++
id = "peas-h4dx3"
title = "Missing doc comments for global_config and updater modules in lib.rs"
type = "chore"
status = "todo"
priority = "low"
created = "2026-03-31T16:25:24.814788Z"
updated = "2026-03-31T16:25:24.814788Z"
+++

## Description

All other modules in `src/lib.rs` have `///` doc comment blocks above their `pub mod` declaration. The two new modules added by the update checker feature are missing them:

```rust
pub mod global_config;  // no doc comment
pub mod updater;        // no doc comment
```

## Fix

Add brief doc comments consistent with the rest of the file.
