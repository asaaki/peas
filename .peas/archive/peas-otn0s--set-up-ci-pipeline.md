+++
id = "peas-otn0s"
title = "Set up CI pipeline"
type = "task"
status = "scrapped"
priority = "high"
created = "2026-04-03T12:25:28.418005733Z"
updated = "2026-04-03T12:30:40.113394769Z"
+++

No automated CI is gating merges. For a project that completed an M6 "Code Quality and Production Readiness" milestone, this is a gap.

## Suggested scope
- GitHub Actions workflow running on push/PR to main
- \`just ci\` (clippy + tests + formatting check)
- Matrix: at minimum stable Rust on ubuntu-latest; ideally also macOS and Windows
- Consider: cargo-deny or cargo-audit for dependency auditing
