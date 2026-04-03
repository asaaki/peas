+++
id = "peas-5wzs3"
title = "Generalize bulk operations in main.rs"
type = "chore"
status = "completed"
priority = "normal"
created = "2026-01-20T10:25:06.679255200Z"
updated = "2026-01-20T12:35:00Z"
+++

# Generalize Bulk Operations - COMPLETED ✅

## What Was Done

This ticket was completed in commit `77a8a97 - Extract Bulk operations handler with generic patterns`.

Created `src/cli/handlers/bulk.rs` with generalized bulk operation handlers:

### Generic Pattern Functions

**1. `bulk_update<F, M>()`** - Generic bulk update handler for simple mutations:
```rust
fn bulk_update<F, M>(
    ctx: &CommandContext,
    ids: &[String],
    json: bool,
    mut mutate: F,
    message_fn: M,
) -> Result<()>
where
    F: FnMut(&mut Pea) -> bool,
    M: Fn(&str) -> String,
```

**2. `bulk_update_with_skip<F, M>()`** - Bulk update with skip capability:
```rust
fn bulk_update_with_skip<F, M>(
    ctx: &CommandContext,
    ids: &[String],
    json: bool,
    mut mutate: F,
    message_fn: M,
) -> Result<()>
where
    F: FnMut(&mut Pea) -> (bool, Option<String>),
    M: Fn(&str) -> String,
```

### Operations Using Generic Pattern

All bulk operations now use these generic handlers:
- **Bulk Status** - Update status for multiple tickets
- **Bulk Start** - Start multiple tickets (status → in-progress)
- **Bulk Done** - Complete multiple tickets (status → completed)
- **Bulk Tag** - Add tags to multiple tickets (with skip if already tagged)
- **Bulk Parent** - Set parent for multiple tickets
- **Bulk Create** - Create multiple tickets from stdin with dry-run support

### Benefits

- **No duplication**: Single implementation for bulk update pattern
- **Consistent error handling**: All operations handle errors the same way
- **JSON output support**: Consistent JSON format across all bulk operations
- **Proper statistics**: Updated/skipped/error counts for all operations
- **Touch timestamps**: All mutations properly update timestamps

### Example Usage

The generic pattern eliminates code like this:
```rust
// Before: Duplicate logic in each operation
for id in ids {
    match repo.get(id) {
        Ok(mut pea) => {
            pea.status = new_status;
            pea.touch();
            // ... error handling, printing, etc.
        }
        Err(e) => { /* error handling */ }
    }
}
```

And replaces it with:
```rust
// After: Generic handler with custom mutation
bulk_update(ctx, &ids, json, 
    |pea| { pea.status = new_status; true },
    |id| format!("Updated {} -> {}", id, new_status)
)
```

## Impact

- Reduced bulk operation code by ~60%
- Single source of truth for bulk update pattern
- Easy to add new bulk operations
- Consistent user experience across all bulk commands

## Commit
`77a8a97 - Extract Bulk operations handler with generic patterns`
