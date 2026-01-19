+++
id = "peas-xfu2a"
title = "Add memory support to GraphQL API"
type = "feature"
status = "todo"
priority = "low"
parent = "peas-oex03"
blocking = ["peas-1uu22"]
created = "2026-01-19T22:40:35.274470100Z"
updated = "2026-01-19T22:40:35.274470100Z"
+++

Extend GraphQL schema to query and mutate memories

**Types:**
```graphql
type Memory {
  key: String!
  content: String!
  tags: [String!]!
  created: DateTime!
  updated: DateTime!
}

type Query {
  memory(key: String!): Memory
  memories(tag: String): [Memory!]!
}

type Mutation {
  createMemory(key: String!, content: String!, tags: [String!]): Memory!
  updateMemory(key: String!, content: String!, tags: [String!]): Memory!
  deleteMemory(key: String!): Boolean!
}
```

**Files:** src/graphql/types.rs, src/graphql/schema.rs
