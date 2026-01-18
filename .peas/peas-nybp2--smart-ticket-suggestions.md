+++
id = "peas-nybp2"
title = "Smart ticket suggestions"
type = "research"
status = "todo"
priority = "normal"
parent = "peas-up8r0"
created = "2026-01-18T17:55:42.912945500Z"
updated = "2026-01-18T17:55:42.912945500Z"
+++

Design a 'peas suggest' command that recommends what to work on next.

Needs design input on CLI UX:
- How to present suggestions? List with reasoning?
- Should it consider: dependencies, priorities, blockers, age?
- Interactive mode to accept/start a suggestion?
- Integration with 'peas start'?

Example output concepts:
1. peas suggest → shows top 3 recommendations with reasons
2. peas suggest --start → shows suggestions, lets you pick one to start
3. peas suggest --json → structured output for LLM consumption
