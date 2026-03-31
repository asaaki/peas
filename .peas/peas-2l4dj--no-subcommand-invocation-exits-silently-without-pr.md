+++
id = "peas-2l4dj"
title = "No-subcommand invocation exits silently without printing help"
type = "bug"
status = "todo"
priority = "normal"
created = "2026-03-31T16:24:52.580623Z"
updated = "2026-03-31T16:24:52.580623Z"
+++

## Description

In `src/main.rs` lines 62-65, when `peas` is run without a subcommand, the fallback code calls `Cli::try_parse_from(["peas", "--help"])` but discards the result with `let _`. No help is ever printed and the process exits with code 1 silently.

## Steps to Reproduce

```
peas
```

## Expected Behavior

Help output is printed and process exits with code 0.

## Actual Behavior

No output. Process exits with code 1.

## Fix

Replace `let _ = Cli::try_parse_from(["peas", "--help"]);` with `Cli::parse_from(["peas", "--help"]);` which prints help and calls `process::exit(0)` internally.
