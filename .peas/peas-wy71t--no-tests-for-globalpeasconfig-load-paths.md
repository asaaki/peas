+++
id = "peas-wy71t"
title = "No tests for GlobalPeasConfig load paths"
type = "chore"
status = "todo"
priority = "low"
created = "2026-03-31T16:25:26.284980Z"
updated = "2026-03-31T16:25:26.284980Z"
+++

## Description

`src/updater.rs` has unit tests for version comparison and retry step-down, but `src/global_config.rs` has no tests. The load-from-absent-file and load-from-malformed-file paths are the most important (must never panic) and are uncovered.

## Fix

Add unit tests using a `tempdir` covering at minimum:
- File absent → returns default (updates enabled)
- File malformed TOML → warns to stderr, returns default
- Valid file with `enabled = false` → returns disabled config
