+++
id = "peas-bkvek"
title = "Double cache read on update check fetch failure"
type = "bug"
status = "completed"
priority = "normal"
created = "2026-03-31T16:23:39.448620Z"
updated = "2026-04-03T11:11:02.755325599Z"
+++

## Description

In `src/updater.rs` around line 162, when the HTTP fetch fails, the code re-reads the cache from disk a second time solely to recover `retry_interval_hours`. The initial `read_cache()` call at the top of `run_update_check` already had this value but it was discarded after the staleness check.

## Expected Behavior

The interval from the initial cache read is reused in the failure branch — no second disk read.

## Actual Behavior

Cache file is read twice on failure: once for the staleness check, once to get the retry interval. Creates a minor TOCTOU window and redundant I/O.

## Fix

Carry `retry_interval_hours` from the first `read_cache()` call through to the failure branch.
