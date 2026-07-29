# 3. Use Skim Fuzzy Search Algorithm for Sub-millisecond Search

Date: 2026-07-29

## Status

Accepted

## Context

Notational Velocity requires sub-millisecond search query filtering across note titles and content as the user types.

## Decision

We use `fuzzy-matcher` (Skim algorithm) with:
- 3x score weighting on title matches over content body matches.
- In-memory index refreshed on startup and updated incrementally on note CRUD operations.
- Secondary sorting tiebreaker based on modification timestamp and filename for 100% deterministic ranking.

## Consequences

- Search responses complete in < 1ms for thousands of notes.
- Instant feedback as user types with match character highlighting.
