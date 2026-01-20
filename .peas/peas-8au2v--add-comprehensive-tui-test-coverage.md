+++
id = "peas-8au2v"
title = "Add comprehensive TUI test coverage"
type = "chore"
status = "todo"
priority = "high"
parent = "peas-w51zp"
created = "2026-01-20T15:11:04.862462Z"
updated = "2026-01-20T15:12:36.874631500Z"
+++

## Problem
TUI has 200+ LOC of handlers but ZERO tests. No coverage for:
- Modal state transitions
- Navigation logic  
- Key event handling
- Edge cases

## Impact
- Regressions go undetected
- Refactoring is risky
- Quality confidence low

## Solution
Add tests for:
1. State machine transitions (modal open/close)
2. Navigation edge cases (empty lists, boundary checks)
3. Key event handling (all handlers)
4. Error scenarios (invalid selections, etc)

Use ratatui TestBackend for UI testing

## Files
- tests/tui_tests.rs (new)
- src/tui/handlers/*.rs
