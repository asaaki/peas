+++
id = "peas-b7x29"
title = "Add external issue tracker references metadata field (list of URLs)"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-04-02T15:37:55.009893Z"
updated = "2026-04-03T11:29:05.230620939Z"
+++

Users may file issues in external trackers (e.g. GitHub, Jira) that should be converted into peas tickets. To maintain the connection between the peas ticket and its external origin(s), a new metadata field should be added to store a list of URLs pointing to external issue tracker entries.

Requirements:
- The field should support multiple URLs (M:N relationship)
- It should be tracker-agnostic (not GitHub-specific)
- Represented as a list of URLs in the ticket metadata
