---
type: is
id: is-01m0tdy9tx76dachmfcgrq5r3a
title: Reject a zero child-page limit
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:54.973Z
updated_at: 2026-08-24T17:43:54.973Z
---
At PR 47 head e658915, ChildPageRequest accepts Bound::Limit(0). child_page then returns no rows, a nonempty remainder, and next=None because there is no last emitted name. The result says truncated but has_next false and gives the caller no cursor, so a nonempty directory cannot be paged. Fix: require a positive page limit at every public boundary, matching the MetaBrowser query contract, and test zero on the Rust and Python surfaces. Review finding FDU47-R7.
