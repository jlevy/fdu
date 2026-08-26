---
type: is
id: is-01m0xp3y8tpxjcgvmbxc7r3ykb
title: "PR #47: replace surgical golden parsing with broad observable output"
kind: bug
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#discussion_r3858495113
    at: 2026-08-26T00:04:39.091Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#discussion_r3858670483
    at: 2026-08-26T00:28:56.979Z
labels:
  - pr47-review
dependencies:
  - type: blocks
    target: is-01m0y1sawtthrp0bq2agcv07f8
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-26T00:04:31.641Z
updated_at: 2026-08-26T04:05:36.261Z
---
The unresolved PR #47 review identified a systematic golden-test anti-pattern. cli-cost runs fdu inside Node, parses its JSON and counter stream, and prints only selected booleans; several cli-content cases similarly reduce complete product output to hand-picked fields. That converts transparent-box goldens into narrow assertions, hides adjacent regressions, and duplicates relations better tested by the opened-root invariant runner or performance harness. Audit every golden parsing site; keep fixture setup scripts, replace product-output extraction with direct complete deterministic output where practical, move cost relations to focused assertions, and add a source/artifact check that rejects grep/jq/head/tail or inline parsing used to hide product output. A compact stable diagnostic may be goldened only when the entire record is the public surface under test.

## Notes

PR #48 review R13 repaired the dangling spec reference. This remains the Phase 1A prerequisite for replacing surgical golden parsing with broad observable-output assertions.
