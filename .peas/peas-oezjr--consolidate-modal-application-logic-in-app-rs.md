+++
id = "peas-oezjr"
title = "Consolidate modal application logic in app.rs"
type = "chore"
status = "completed"
priority = "normal"
created = "2026-01-20T10:25:07.740578Z"
updated = "2026-01-20T12:30:00Z"
+++

# Consolidate Modal Application Logic - COMPLETED ✅

## What Was Done

This ticket was completed as part of the app.rs refactoring (peas-0ktvu, Phase 3).

Created `src/tui/modal_operations.rs` (185 LOC) with consolidated modal application logic:

### Generic Pattern
Instead of having 6 nearly-identical ~30 LOC functions with duplicate code, created a generic `apply_property_change()` function:

```rust
fn apply_property_change<T, F>(
    target_ids: &[String],
    all_peas: &[Pea],
    repo: &PeaRepository,
    data_path: &Path,
    _property_name: &str,
    new_value: T,
    mut update_fn: F,
) -> Result<String>
where
    T: std::fmt::Display + Copy,
    F: FnMut(&mut Pea, T),
```

### Consolidated Functions
- `apply_status_change()`
- `apply_priority_change()`
- `apply_type_change()`
- `apply_parent_change()`
- `apply_blocking_change()`
- `apply_tags_change()`

### Impact
- Eliminated ~180 LOC of duplication in app.rs
- Each modal apply method in app.rs simplified from ~30 LOC to ~15 LOC
- Consistent error handling and undo recording across all modals
- Single source of truth for modal application pattern

## Commit
Part of commit: "Phase 3: Extract modal operations consolidation (43 LOC reduction)"
