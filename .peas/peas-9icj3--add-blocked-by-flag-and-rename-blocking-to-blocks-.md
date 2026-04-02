+++
id = "peas-9icj3"
title = "Add --blocked-by flag and rename --blocking to --blocks in peas create"
type = "feature"
status = "todo"
priority = "normal"
created = "2026-04-02T14:54:53.193503Z"
updated = "2026-04-02T14:57:06.826178Z"
+++

## Summary

The `peas create` command currently supports `--blocking` to specify which tickets the new pea blocks. Two improvements are needed:

1. **Rename `--blocking` to `--blocks`** — makes the flag name more grammatically clear ("this pea blocks X").
2. **Add `--blocked-by` flag** — allows users to express the same relationship from the opposite direction ("this pea is blocked by X"), which is more natural in some contexts.

Both flags result in the same underlying blocking relationship; `--blocked-by` is simply the inverse perspective.

## Acceptance Criteria

- `peas create` accepts `--blocks <ID>` (renamed from `--blocking`)
- `peas create` accepts `--blocked-by <ID>` to specify that the new pea is blocked by an existing pea
- Both flags accept multiple IDs (consistent with existing behavior)
- Old `--blocking` flag is removed or aliased (prefer clean rename)
- Help text clearly describes both flags
