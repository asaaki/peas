+++
id = "peas-cf7ul"
title = "Configurable sequential ticket ID generation"
type = "feature"
status = "todo"
priority = "normal"
created = "2026-02-02T19:03:01.460178900Z"
updated = "2026-02-02T19:03:01.460178900Z"
+++

Added configurable ID generation mode to support both random (default) and sequential IDs.

## Changes
- Added id_mode configuration option: random (default) or sequential
- Sequential mode generates IDs like peas-00001, peas-00002, etc.
- Counter is persisted in .peas/.id file
- id_length config now controls suffix length for both modes
- Updated README documentation

## Usage
Set in .peas.toml:

[peas]
id_mode = "sequential"
id_length = 5  # produces peas-00001

