+++
id = "peas-mnx8o"
title = "Update notice appended to `--help` output"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-qcb9j"
created = "2026-03-30T11:52:11.313902Z"
updated = "2026-03-31T09:37:36.161428Z"
+++

## Description

After clap prints its \`--help\` output for \`peas\` and all subcommands, append an update notice trailing line when an update is available.

## Acceptance Criteria

- [ ] Update notice printed after clap help output (not modifying clap rendering)
- [ ] Only shown when \`UpdateCheckOutcome::UpdateAvailable\` — no trailing line for up-to-date, failed, or skipped
- [ ] Applies to \`peas --help\` and all subcommand \`--help\` flags
- [ ] Spawns update check thread and joins it before printing the notice

## Notes

Depends on: peas-wxbki (GlobalPeasConfig), peas-3mvco (updater module)
