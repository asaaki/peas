+++
id = "peas-9gez5"
title = "Evaluate replacing rat-text with ratatui-textarea"
type = "research"
status = "todo"
priority = "low"
created = "2026-04-03T14:13:33.577347671Z"
updated = "2026-04-03T14:13:33.577347671Z"
+++

## Context

rat-text 3.1 pulls in fxhash (unmaintained, RUSTSEC-2025-0057) via rat-focus. Not a security risk but a cargo-audit warning.

The original tui-textarea 0.7 was replaced with rat-text in Jan 2026 because tui-textarea was pinned to ratatui 0.29 and we needed 0.30. That incompatibility is now resolved by two forks:

## Options

| Crate | ratatui 0.30 | Maintainer | Notes |
|---|---|---|---|
| rat-text 3.1 (current) | Yes | thscharler | Works, fxhash warning |
| tui-textarea-2 0.10 | Yes | srothgan (fork) | Active fork |
| ratatui-textarea 0.8 | Yes | ratatui org | Official org fork, best long-term bet |

## Recommendation

**ratatui-textarea 0.8** — maintained by the ratatui org, will track ratatui releases.

## Scope

Touches 4 files: app.rs, body_editor.rs, ui_views.rs, edit_body.rs. Need to verify cursor visibility, input handling, save/cancel, and undo buffer integration — these were pain points during the original rat-text migration.

## References

- Original swap commit: 9e02e15
- Cursor fix commit: 6995a7c
- https://github.com/ratatui/ratatui-textarea
- https://github.com/srothgan/tui-textarea
