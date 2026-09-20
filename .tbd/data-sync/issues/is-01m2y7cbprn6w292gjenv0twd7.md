---
type: is
id: is-01m2y7cbprn6w292gjenv0twd7
title: Enhance help examples with README workflows and age/size directory searches
kind: task
status: open
priority: 2
version: 3
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7cf9fsawdtq8p5grnr3nq
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-20T01:37:36.982Z
updated_at: 2026-09-20T04:42:40.327Z
---
Enhance short and long help with README workflows and stale-directory inventory
examples. Teach selection (kind/name/path/age/size), view (list and aggregate reports),
and format (tree default for list, paths, long, machine formats).
Show omitted versus explicit tree equivalence and convenient --long/--tree format
aliases. Default output remains the existing directory roll-up tree, including its
display bounds.

Include .venv/venv older than 7d, node_modules and Cargo target older than 30d, combined
name matching, actual size and age via --format long, paths-only output, JSON, and
oldest-first sorting using existing flag grammar.
Explain subtree recency, exclusions, tree context, and expansion of folded output
concisely. Make format compatibility and legacy names discoverable without retaining
directories-as-a-view terminology.

Review help goldens and execute every example’s syntax against a deterministic fixture.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
