+++
id = "peas-evs3g"
title = "Fix suggest blocking direction"
type = "bug"
status = "in-progress"
priority = "normal"
created = "2026-10-17T22:21:27.823929Z"
updated = "2026-10-17T22:21:27.823929Z"
+++

## Description

`peas suggest` interprets the `blocking` relationship in the opposite direction from the data model. `A.blocking = [B]` means that A blocks B, but the suggestion filter treats B as a prerequisite of A.

## Reproduction

Create two open peas where A blocks B. `peas suggest --json` selects B, even though B should remain unavailable until A is completed or scrapped.

## Expected Behavior

A is actionable while it is open. B is excluded until every pea that lists B in `blocking` is completed or scrapped.

## Actual Behavior

A is excluded while B is open, and B can be suggested instead.

## Fix

Build an inverse blocked-to-blocker index while processing blocking relationships. Use that index with the existing status map to exclude candidates with an incomplete blocker. Preserve the existing `blocking_count` ranking behavior.

## Out of Scope

`blocking_count` is intentionally left unchanged. It counts how many peas block a given pea (keyed by the blocked ID), while its ranking and display uses (`Blocking N ticket(s)`, `Unblocks N ticket(s)`, JSON `blocks_count`) intend to count how many peas a given pea blocks. This looks like a separate direction bug, but fixing it would change ranking and output, so it is excluded from this minimal fix and should be handled separately.

## Verification

Add a CLI regression test covering A blocks B before and after A is marked completed.
