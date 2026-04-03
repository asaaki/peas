+++
id = "peas-z4xdx"
title = "Batch archive: filter-based bulk archival of tickets"
type = "feature"
status = "in-progress"
priority = "normal"
created = "2026-04-03T13:16:25.307443121Z"
updated = "2026-04-03T13:17:41.541137684Z"
+++

## Problem

Over time, completed and scrapped tickets accumulate and clutter the main `peas list` view. Currently, archiving is one-ticket-at-a-time via `peas archive <ID>`, which is tedious when dozens of old tickets need cleanup.

## Proposed Solution

Extend `peas archive` with filter flags for bulk archival, reusing the same filtering patterns already available in `peas list`:

```
peas archive --status <status>         # archive all tickets with given status
peas archive --older-than <duration>   # archive tickets last updated > N days/weeks ago
peas archive --type <type>             # archive by ticket type
peas archive --tags <tag>              # archive by tag
```

Filters are combinable:
```
peas archive --status completed --older-than 30d
peas archive --status scrapped              # all scrapped work
peas archive --type milestone --status completed
```

### Required flags/behavior

- **`--dry-run`**: Preview which tickets would be archived (list them) without moving anything. Should be the default when no `--confirm` is given — show the list and prompt interactively.
- **`--confirm` / `-y`**: Skip interactive confirmation (for scripts/CI).
- **`--keep-assets`**: Same as single-archive mode, preserve attached assets.
- **`--json`**: Machine-readable output listing all archived ticket IDs.

### Duration format for `--older-than`

Support human-friendly durations: `30d` (days), `4w` (weeks), `6m` (months), `1y` (year). The comparison is against the ticket's `updated` timestamp.

### UX Flow (interactive)

```
$ peas archive --status scrapped

Found 19 tickets to archive:
  peas-hn52  [task]      Refactor main.rs command handlers
  peas-cog0  [task]      Consolidate duplicate filtering logic
  peas-64ctb [milestone] M7: Desktop GUI Application (GPUI)
  ... (16 more)

Archive all 19 tickets? [y/N] y
✓ Archived 19 tickets to .peas/archive/
```

### Implementation notes

- Reuse existing `PeaRepository::list()` filtering + `archive()` per ticket
- Add `--older-than` duration parsing (new, not in list filters today)
- Undo support: record all archived IDs so the batch can be undone
- The `--older-than` filter could also be useful in `peas list` later, but that's out of scope here

### Acceptance criteria

- [ ] `peas archive --status <s>` archives all matching tickets
- [ ] `peas archive --older-than <dur>` archives by age
- [ ] Filters are combinable (AND logic)
- [ ] Dry-run is default (prompts before acting); `--confirm` skips prompt
- [ ] `--json` output includes list of archived IDs
- [ ] Undo works for batch operations (single undo reverts the whole batch)
- [ ] Asset handling respects `--keep-assets` for all tickets in batch
