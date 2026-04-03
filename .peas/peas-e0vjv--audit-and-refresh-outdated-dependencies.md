+++
id = "peas-e0vjv"
title = "Audit and refresh outdated dependencies"
type = "task"
status = "completed"
priority = "normal"
created = "2026-04-03T12:25:43.138983289Z"
updated = "2026-04-03T12:41:30.744433720Z"
+++

\`cargo outdated\` shows widespread staleness in transitive dependencies (tokio, hyper, indexmap, serde, etc.). Several direct deps may also have patch updates available.

## Action
- Run \`cargo update\` to pull in compatible semver updates
- Review \`cargo outdated\` output for any major version bumps worth adopting
- Run full test suite after updating
- Consider adding \`cargo-deny\` or \`cargo-audit\` to catch known vulnerabilities

Note: this partially overlaps with the wl-clipboard-rs ticket but covers the broader dependency tree.
