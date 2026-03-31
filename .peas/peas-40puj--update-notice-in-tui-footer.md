+++
id = "peas-40puj"
title = "Update notice in TUI footer"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-qcb9j"
created = "2026-03-30T11:52:29.726854Z"
updated = "2026-03-31T09:37:38.124854Z"
+++

## Description

Integrate the update checker into the TUI: spawn check at startup in a background thread, poll for completion each tick, and append an update badge to the footer when an update is available.

## Acceptance Criteria

- [ ] \`App\` gains \`update_check_handle: Option<JoinHandle<UpdateCheckOutcome>>\` and \`available_update: Option<String>\`
- [ ] \`spawn_update_check\` called during \`App::new\`, handle stored
- [ ] Each render tick: check \`handle.is_finished()\`, join once done, store version in \`available_update\`, set handle to \`None\`
- [ ] Footer (\`draw_footer\`) appends \`  ● update available: vX.Y.Z\` in amber/yellow when \`available_update\` is \`Some\`
- [ ] Silent on \`CheckFailed\`, \`UpToDate\`, \`Skipped\` — nothing added to footer

## Notes

Depends on: peas-wxbki (GlobalPeasConfig), peas-3mvco (updater module)
