+++
id = "peas-pabgq"
title = "Refactor: move all markers and emojis into Theme struct"
type = "chore"
status = "completed"
priority = "normal"
parent = "peas-2v2rx"
created = "2026-01-19T18:34:53.069615700Z"
updated = "2026-02-02T23:56:10.222566200Z"
+++

Moved all visual markers (logo, row_marker, pane_markers, page_marker) and type emojis into Theme struct. Renamed Markers to TuiConfig for behavioral flags only. Clean separation between visual styling (Theme) and configuration (TuiConfig).
