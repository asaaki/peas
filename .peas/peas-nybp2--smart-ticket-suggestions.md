+++
id = "peas-nybp2"
title = "Smart ticket suggestions"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-up8r0"
created = "2026-01-18T17:55:42.912945500Z"
updated = "2026-01-18T17:57:38.865056300Z"
+++

Add 'peas suggest' command that recommends the next ticket to work on.

Simple selection rules (in order):
1. Bugs before features before chores
2. Higher priority first
3. Skip blocked tickets (unmet dependencies)
4. Oldest first as tiebreaker

Usage:
  peas suggest          # returns single best ticket
  peas suggest --count 3  # returns top 3
  peas suggest --json   # structured output

No scoring display, no reasoning output, no interactivity.
The logic is implicit and documented in --help.
