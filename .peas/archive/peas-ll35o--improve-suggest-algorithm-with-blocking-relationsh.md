+++
id = "peas-ll35o"
title = "Improve suggest algorithm with blocking relationships and dependencies"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-01-20T00:18:02.077264600Z"
updated = "2026-01-20T00:24:50Z"
+++

Enhance the suggest command to provide smarter recommendations.

## Current Limitations
- No awareness of blocking relationships
- Doesn't check if dependencies are satisfied
- No parent/epic context consideration
- No unblocking score (how many tickets this unblocks)

## Proposed Improvements

1. **Blocking Score**: Prioritize tickets that are blocking many others
   - Count how many tickets list this as a blocker
   - Higher blocking count = higher priority

2. **Dependency Ready Check**: Only suggest tickets whose blockers are completed
   - Parse 'blocking' field
   - Verify all blocking tickets are done
   - Filter out tickets with unmet dependencies

3. **Epic/Milestone Context**: Suggest tickets within active work streams
   - If there are in-progress tickets, suggest siblings in same parent
   - Helps maintain focus on completing epics

4. **Age Factor**: Consider ticket age as tiebreaker
   - Older tickets might indicate technical debt
   - Optional weight in scoring

5. **Multi-ticket suggestion**: Show top 3-5 suggestions instead of just one
   - Gives LLMs options to choose based on context
   - Show reason for each suggestion

## Implementation Plan
- Add blocking_count calculation
- Add is_ready() check (all blockers completed)
- Enhance scoring function
- Add --limit flag for number of suggestions
