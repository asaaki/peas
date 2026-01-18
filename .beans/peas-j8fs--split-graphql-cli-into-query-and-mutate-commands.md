---
# peas-j8fs
title: Split GraphQL CLI into query and mutate commands
status: completed
type: task
priority: normal
created_at: 2026-01-18T14:51:10Z
updated_at: 2026-01-18T14:56:26Z
parent: peas-ep55
---

Current: peas graphql '<query>' requires 'mutation { }' wrapper for mutations

Proposed:
- peas query '<graphql-query>' - for queries
- peas mutate '<mutation-body>' - auto-wraps in 'mutation { }'

Examples:
  peas query '{ stats { total } }'
  peas mutate 'setStatus(id: "peas-abc1", status: IN_PROGRESS) { id status }'

This improves ergonomics by removing the need to type 'mutation { }' wrapper.