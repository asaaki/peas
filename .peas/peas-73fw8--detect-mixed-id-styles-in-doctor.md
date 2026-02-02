+++
id = "peas-73fw8"
title = "Detect mixed ID styles in doctor"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-02-02T23:12:57.060623500Z"
updated = "2026-02-02T23:13:54.922261900Z"
+++

Add a check to peas doctor that detects when tickets have mixed ID styles (random vs sequential), which can happen when switching id_mode. Warn users about the inconsistency but don't block - it's functional, just messy.
