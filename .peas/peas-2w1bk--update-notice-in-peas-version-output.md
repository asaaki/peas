+++
id = "peas-2w1bk"
title = "Update notice in `peas --version` output"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-qcb9j"
created = "2026-03-30T11:51:56.068846Z"
updated = "2026-03-31T09:37:33.194641Z"
+++

## Description

Override clap's default \`--version\` output to append an update notice after joining the background update check thread.

## Acceptance Criteria

- [ ] \`peas --version\` spawns update check thread, joins it, then prints result
- [ ] When update available: prints version line + \`A new version is available: X.Y.Z — https://github.com/asaaki/peas/releases/latest\`
- [ ] When check failed: prints version line + \`No update information available (check failed)\`
- [ ] When up-to-date or skipped: prints version line only

## Notes

Depends on: peas-wxbki (GlobalPeasConfig), peas-3mvco (updater module)
