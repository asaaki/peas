+++
id = "peas-itd3r"
title = "Disable log output to stdout/stderr in TUI mode"
type = "task"
status = "completed"
priority = "normal"
created = "2026-01-30T17:27:47.645807Z"
updated = "2026-01-30T17:27:47.645807Z"
+++

## Problem
Log messages were printing over the TUI rendering, interfering with the display and causing visual corruption. In TUI mode, stdout/stderr should not have log output.

## Solution
- Updated logging::init() to accept a 'quiet' parameter
- When quiet=true, stderr logging is disabled using Option<Layer> pattern
- File logging still works regardless of quiet mode
- Updated main.rs to detect TUI mode and pass quiet=true to logging::init()
- Used tracing-subscriber's optional layer pattern to conditionally disable stderr output

## Files Changed
- src/logging.rs: Added quiet parameter, restructured to use Option for conditional stderr layer
- src/main.rs: Detect TUI mode with matches!(cli.command, Commands::Tui) and pass to logging::init()

## Result
- TUI rendering is now clean without log interference
- Logs can still be captured with --log-file option
- All other CLI commands still log to stderr normally
- No behavioral change for non-TUI commands
