+++
id = "peas-65ipq"
title = "Add doc-tests for public API examples"
type = "task"
status = "completed"
priority = "low"
created = "2026-04-03T12:25:39.180259277Z"
updated = "2026-04-03T13:10:03.563327640Z"
+++

\`cargo test\` reports \`running 0 tests\` for doc-tests. The \`lib.rs\` module docs have narrative descriptions but no testable code examples.

## Suggested approach
- Add \`# Examples\` with runnable code blocks to key public types: \`PeaRepository\`, \`Pea\`, \`PeaType\`, \`PeaStatus\`, \`SearchQuery\`, \`MarkdownParser\`
- These serve double duty: verified documentation and regression tests
- Don't aim for exhaustive coverage — focus on the types external consumers (GraphQL, TUI) actually use
