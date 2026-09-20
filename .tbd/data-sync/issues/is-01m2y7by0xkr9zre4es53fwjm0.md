---
type: is
id: is-01m2y7by0xkr9zre4es53fwjm0
title: Core list selection with directory subtree metrics and union aggregation
kind: feature
status: in_progress
priority: 1
version: 4
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7c1v3etw9wkzz87vgm6ke
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
hold: null
hold_until: null
created_at: 2026-09-20T01:37:22.965Z
updated_at: 2026-09-20T04:42:28.252Z
started_at: 2026-09-20T01:38:32.676Z
---
Implement the epic’s shared list query in `fdu-core`. Entry kind is a filter; directory
size, counts, and newest activity are subtree metrics even without `--kind dir`. Apply
exclusions throughout eligible contents before aggregate bounds; positive directory
matches cover descendants for summary/grouped union aggregation without adding unmatched
descendants to flat listings.
Nested roots are emitted independently and aggregate totals count their covered contents
only once.

Provide the shared list query with existing directory roll-ups for tree presentation and
exact matching rows for flat/machine formats.
Preserve default tree output; files contribute to directory totals without new
individual leaf rows.
Separate selected matches from structural ancestors and report truncation from
presentation folding.
Use iterative retained-index readers, preserve cache/snapshot identity and raw-entry
semantics, support portable identity, and charge opened-report work budgets for all
traversals.

Acceptance: deterministic size/count/mtime and boundary tests, empty/nested/fresh roots,
file/directory/symlink recency, future/pre-epoch/unknown age, descendant exclusion and
ignored policies, all-kind matching, union totals, repeat queries without I/O, stable
sort and limit, deep-tree safety, and default/explicit-format-independent metrics.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
