+++
id = "peas-04w4a"
title = "Evaluate beads import/export"
type = "research"
status = "completed"
priority = "normal"
parent = "peas-macd2"
created = "2026-01-18T16:27:25.091092300Z"
updated = "2026-01-18T16:27:31.670459200Z"
+++

Investigate beads format and assess feasibility of import/export support.

## Findings

**Beads Format:**
- Uses JSONL (JSON Lines) format stored in `.beads/` directory
- Has an invisible SQLite local cache layer for performance
- Uses hierarchical IDs like `bd-a3f8.1.1`
- Git-based versioning ("Git as Database")

**Key Differences from Peas/Beans:**
1. JSONL vs Markdown with frontmatter
2. SQLite cache vs pure flat files
3. Different ID scheme (hierarchical vs flat)
4. More complex dependency model (blocks, related, parent-child)

**Feasibility Assessment:**

Import FROM beads: **Feasible but lossy**
- Can read JSONL and extract basic fields (title, status, priority, description)
- Hierarchical IDs would need flattening or mapping
- Complex dependency relationships may be simplified

Export TO beads: **Not recommended**
- Beads expects its SQLite cache to be in sync with JSONL
- Would need to generate compatible hierarchical IDs
- Missing dependency graph features would create incomplete beads

**Recommendation:**
- Import from beads: Implement as a one-way migration tool (low priority)
- Export to beads: Skip - too much impedance mismatch, users should use beads directly if they need its features

Sources:
- https://github.com/steveyegge/beads
