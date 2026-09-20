---
type: is
id: is-01m2y7c6yzccahx1ntvy2wf1s1
title: Document list formats, directory filters, and stale build inventories
kind: task
status: closed
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7cf9fsawdtq8p5grnr3nq
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
hold: null
hold_until: null
created_at: 2026-09-20T01:37:32.117Z
updated_at: 2026-09-20T07:12:32.238Z
started_at: 2026-09-20T06:16:17.039Z
closed_at: 2026-09-20T07:12:32.238Z
close_reason: "Implemented the tracked file/function plan above PR #96. Core/workspace tests, 168 CLI goldens, 48 Python tests, typing, docs formatting, and cross-platform lint passed. Final aggregate gate and stacked PR CI are tracked separately in fdu-arv8."
resolution: null
duplicate_of: null
---
Update README, docs/usage.md (--docs), portable --skill, Rust API docs, Python README,
models/stubs, machine-schema reference, architecture axes and selection semantics, and
appropriate release notes to the accepted list-view and format model.

Explain unchanged default output, default list/tree equivalence, formats
tree/paths/long/json/jsonl/yaml, automatic grouped tables, format aliases, compatibility
names, invalid combinations, analyzer-driven defaults, and bounds/folding.
Directory kind is a filter.
Size/age describe eligible subtrees; exclusions win, nested rows overlap, summary unions
do not double-count, and structural tree ancestors are not extra matches.
Tree retains its existing directory roll-ups; matching regular files contribute bytes
without new individual leaves.
Flat formats show one row per match.
Describe current tree bounds and flat-list bounds without implying identical rendered
path sets.

Include .venv/venv, node_modules, Cargo target, individual and combined names, 7d/30d
modified-before examples, long size/age columns, exact machine values, oldest-first,
ignored inventory, and repeated queries over one retained index.
Distinguish modification age from last use, scan coverage from display bounds, and
counted bytes from uniquely reclaimable space.
Apply common-doc guidelines and run docs-format.
Check every example against the implementation and keep README/help/skill/Python
terminology aligned.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
