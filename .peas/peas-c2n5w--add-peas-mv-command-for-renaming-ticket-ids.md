+++
id = "peas-c2n5w"
title = "Add peas mv command for renaming ticket IDs"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-02-03T00:04:04.344550500Z"
updated = "2026-02-03T00:07:04.123416200Z"
+++

Add a command to rename/re-ID tickets:

```
peas mv <old-id> <new-suffix>
```

Example: `peas mv peas-4988 4988a` → renames to `peas-4988a`

The command must:
- Rename the ID (prefix stays, only suffix changes)
- Rename the file
- Update all references (parent, blocking)

Validation (blocked by default, --force overrides):
- Length mismatch: new suffix ≠ configured id_length
- Mode mismatch: all-digits in random mode, non-digits in sequential mode
