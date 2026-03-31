+++
id = "peas-qcb9j"
title = "Update checker feature"
type = "epic"
status = "completed"
priority = "normal"
created = "2026-03-30T11:51:11.502277Z"
updated = "2026-03-31T09:37:28.313701Z"
+++

## Overview

Implement an automatic update checker for peas that queries the GitHub Releases API, caches the result locally, and surfaces update notices in multiple places.

See spec: docs/superpowers/specs/2026-03-30-update-checker-design.md

## Goals

- Inform users when a newer version of peas is available
- Never block startup or commands with synchronous network I/O
- Respect user opt-out via global config
- Show update notices in: TUI footer, \`peas doctor\`, \`peas --version\`, \`--help\` output

## Success Metrics

- Update check runs in a background thread in all cases
- Cache prevents more than one check per 24h (with progressive retry backoff on failure)
- All notification surfaces show consistent, accurate information
