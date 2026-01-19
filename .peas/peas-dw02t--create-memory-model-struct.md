+++
id = "peas-dw02t"
title = "Create Memory model struct"
type = "feature"
status = "completed"
priority = "high"
parent = "peas-kgp6m"
created = "2026-01-19T22:38:53.005606900Z"
updated = "2026-01-19T22:48:46.396141500Z"
+++

Define the Memory struct in src/model/memory.rs

**Structure:**
- key: String (unique identifier, used as filename)
- content: String (markdown body)
- tags: Vec<String> (for categorization)
- created: DateTime<Utc>
- updated: DateTime<Utc>

**Similar to:** Pea struct in src/model/pea.rs
**Export from:** src/model/mod.rs
