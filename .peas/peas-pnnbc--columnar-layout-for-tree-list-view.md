+++
id = "peas-pnnbc"
title = "Columnar layout for tree/list view"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-7z7f5"
created = "2026-01-18T19:30:13.842670300Z"
updated = "2026-01-18T19:30:13.842670300Z"
+++

Align tree/list items in columns like beans does.

Current layout (roughly):
```
├─ peas-xxxxx [epic] E6.1: Bug Fixes
```

Beans-style columnar layout:
```
├─ peas-xxxxx     epic      completed    !! Title here
```

Columns (fixed width, left-aligned):
1. Tree prefix + ID (padded)
2. Type
3. Status  
4. Priority indicator + Title

This makes scanning much easier and looks more professional.
