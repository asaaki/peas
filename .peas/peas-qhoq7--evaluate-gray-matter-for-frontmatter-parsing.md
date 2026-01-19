+++
id = "peas-qhoq7"
title = "Evaluate gray_matter for frontmatter parsing"
type = "research"
status = "completed"
priority = "normal"
parent = "peas-7z7f5"
created = "2026-01-19T00:22:49.487933400Z"
updated = "2026-01-19T00:31:20.053369900Z"
+++

## Evaluation of gray_matter

### What it does
gray_matter is a Rust port of the JavaScript gray-matter library. It extracts frontmatter from markdown files, supporting YAML, JSON, TOML, and custom formats.

### API
```rust
let matter = Matter::<YAML>::new();
let result = matter.parse(content)?;
// result.content = body text
// result.data = parsed frontmatter struct
```

### Comparison with current implementation
Our current `src/storage/markdown.rs` already handles:
- YAML frontmatter (`---`)
- TOML frontmatter (`+++`)
- Auto-detection of format
- Direct deserialization into Pea struct

### Verdict: NOT RECOMMENDED
Our current implementation is:
1. Already working well with ~80 lines of code
2. Directly deserializes into our Pea type
3. Supports both YAML and TOML
4. Has no external dependencies beyond serde_yaml/toml

gray_matter would add another dependency for marginal benefit. It would require an intermediate step to convert their generic parsed data into our Pea struct.

**Recommendation**: Keep current implementation. It's simple, works, and is tailored to our needs.
