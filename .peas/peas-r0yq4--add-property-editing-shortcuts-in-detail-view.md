+++
id = "peas-r0yq4"
title = "Add property editing shortcuts in detail view"
type = "task"
status = "completed"
priority = "normal"
created = "2026-01-19T21:17:04.715416800Z"
updated = "2026-01-19T21:17:04.715416800Z"
+++

## Description

When viewing a ticket in detail view, property editing shortcuts (s, P, t, p, b) should work without having to exit back to the tree view.

## Solution

The functionality was already implemented in lines 1524-1538 of app.rs. The issue was that the help text in DetailView mode didn't show these shortcuts, so users didn't know they existed.

Updated the help text from:
```
" ↓/↑:scroll  e:edit  Esc/Enter/q:close "
```

To:
```
" ↓/↑:scroll  e:edit  s:status  P:priority  t:type  p:parent  b:blocking  y:copy-id  Esc/Enter/q:close "
```

## Available Shortcuts in Detail View

- **↓/↑** - Scroll up/down
- **e** - Edit body inline
- **E** - Edit in external editor
- **s** - Change status
- **P** - Change priority (uppercase P)
- **t** - Change type
- **p** - Change parent
- **b** - Change blocking relationships
- **y** - Copy ticket ID to clipboard
- **Esc/Enter/q** - Close detail view

## Acceptance Criteria

- [x] Status modal (s) works in DetailView mode
- [x] Priority modal (P) works in DetailView mode  
- [x] Type modal (t) works in DetailView mode
- [x] Parent modal (p) works in DetailView mode
- [x] Blocking modal (b) works in DetailView mode
- [x] Help text updated to show all available shortcuts
