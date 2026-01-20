+++
id = "peas-4fz98"
title = "Consolidate duplicated modal handler logic"
type = "chore"
status = "todo"
priority = "normal"
parent = "peas-w51zp"
created = "2026-01-20T15:11:22.840533300Z"
updated = "2026-01-20T15:12:37.610491100Z"
+++

## Problem
Status/Priority/Type modal operations are nearly identical but duplicated across:
- src/tui/handlers/modal_status.rs
- src/tui/handlers/modal_priority.rs  
- src/tui/handlers/modal_type.rs

Each follows identical pattern with different field modifications.

## Impact
- Maintenance burden (fix bugs 3 times)
- Inconsistent behavior possible
- Code bloat (~200 LOC duplicated)

## Solution
Create generic modal handler:
```rust
fn handle_enum_modal<T: EnumProperty>(
    app: &mut App,
    options: &[T],
    field_updater: impl Fn(&mut Pea, T)
)
```

## Files
- src/tui/handlers/modal_operations.rs (extend)
- src/tui/handlers/modal_*.rs
