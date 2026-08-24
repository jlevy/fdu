---
type: is
id: is-01m0tdy9tx76dachmfcgrq5r3a
title: Reject a zero child-page limit
kind: bug
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:54.973Z
updated_at: 2026-08-24T20:46:14.650Z
---
At PR 47 head e658915, ChildPageRequest accepts Bound::Limit(0). child_page then returns no rows, a nonempty remainder, and next=None because there is no last emitted name. The result says truncated but has_next false and gives the caller no cursor, so a nonempty directory cannot be paged. Fix: require a positive page limit at every public boundary, matching the MetaBrowser query contract, and test zero on the Rust and Python surfaces. Review finding FDU47-R7.

## Notes

DESIGN CONFIRMED (2026-08-24 review). Mechanism verified in `child_page`: with
Limit(0), `admits(0)` is false on the first row, `more = true`, `last = None`, so
`next: more.then_some(last).flatten()` is None -- remainder present (truncated) AND
next absent (terminal). Zero rows, no continuation.

Reject zero where a continuation is the contract: `ChildPageRequest` (Rust: structured
error from the request boundary, not a clamp -- a clamp answers a question the caller
did not ask) and the Python `limit=` (ValueError naming the rule). MetaBrowser's
contract already requires every page/row bound positive, so the adapter never sends it.

Deliberately NOT rejected: `extensions=0` (an extension map with everything in `rest`
is representable and has no cursor to strand) and `depth 0` (root-only is meaningful).
The trap is specifically a paged bound, where truncation must imply resumability.
Record that distinction in the doc comment so the next bound added lands on the right
side of it.
