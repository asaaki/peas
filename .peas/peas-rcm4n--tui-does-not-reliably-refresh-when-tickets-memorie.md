+++
id = "peas-rcm4n"
title = "TUI does not reliably refresh when tickets/memories change on disk"
type = "bug"
status = "completed"
priority = "high"
tags = ["bug"]
created = "2026-04-01T13:19:50.346997Z"
updated = "2026-04-03T11:11:03.520142538Z"
+++

## User Feedback

> "One thing I found (this maybe cuz I was an early adopter n never updated it) the TUI does not watch for updates."

The user meant that the TUI does not refresh ticket/memory data when files change on disk (e.g. when an AI agent updates tickets externally), NOT the version update checker.

## Current Behavior

- A file watcher (`notify_debouncer_mini`) is set up on the `.peas/` directory with 300ms debounce (`src/tui/app.rs:1373-1381`)
- On events, `app.refresh()` is called which re-reads all tickets and memories (`src/tui/app.rs:1428-1434`)
- However, users report the TUI does not reflect external changes reliably

## Possible Causes

- `try_recv()` only reads one event per loop tick — channel may back up
- Some tools may write files via atomic rename from outside the watched directory, which `notify` may not detect
- The watcher or refresh logic may have other edge cases causing missed updates

## Expected Behavior

When tickets or memories are created, updated, or deleted by external processes (CLI, AI agents), the TUI should reliably detect the changes and refresh the display.

## Relevant Code

- `src/tui/app.rs:1373-1381` — file watcher setup (notify_debouncer_mini, 300ms debounce)
- `src/tui/app.rs:1428-1434` — fs event handling in run_app loop (try_recv + refresh)
- `src/tui/app.rs:331-340` — App::refresh() re-reads all peas and memories
