+++
id = "peas-yil4s"
title = "Document TUI state machine"
type = "chore"
status = "completed"
priority = "normal"
parent = "peas-w51zp"
created = "2026-01-20T15:12:05.449160300Z"
updated = "2026-01-20T21:47:38.906907300Z"
+++

## Problem
App struct has 50+ public fields with no documentation of:
- Valid state combinations
- State transition rules
- Invariants that must be maintained

Examples of unclear states:
- Can input_mode be EditBody while detail_pane is Relations?
- What happens if modal_selection > options length?
- When should relations_items be empty?

## Impact
- Hard to understand code
- Easy to introduce bugs
- Refactoring is risky

## Solution
1. Add state machine diagram to docs/
2. Document valid state combinations in app.rs
3. Add debug assertions for invariants
4. Create state transition tests

## Files
- src/tui/app.rs (add doc comments)
- docs/tui-state-machine.md (new)
