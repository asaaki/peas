+++
id = "peas-ercb1"
title = "Help update notice goes to stderr instead of stdout"
type = "bug"
status = "completed"
priority = "high"
created = "2026-03-31T16:23:39.472982Z"
updated = "2026-04-03T11:11:03.010985643Z"
+++

## Description

In `src/main.rs`, the `--help` update notice is printed via `eprintln!` (stderr) while clap's help output goes to stdout via `e.print()`. Users piping `peas --help | cat` will not see the update notice.

## Steps to Reproduce

```
peas --help 2>/dev/null
```

The update notice (when an update is available) will be suppressed.

## Expected Behavior

Update notice appears on stdout, immediately after the help output.

## Actual Behavior

Update notice goes to stderr.

## Fix

Change `eprintln!` → `println!` in the `DisplayHelp` arm of `src/main.rs`.
