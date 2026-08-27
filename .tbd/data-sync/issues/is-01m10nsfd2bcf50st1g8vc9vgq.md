---
type: is
id: is-01m10nsfd2bcf50st1g8vc9vgq
title: Prove cross-platform path, ordering, completeness, and total semantics
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nsfqq5vawhed0nhy4wa43
parent_id: is-01m0y1sjnptgqhgvqcx1cjkkhw
created_at: 2026-08-27T03:56:32.033Z
updated_at: 2026-08-27T03:56:32.375Z
---
Run both providers and the fdu Python binding over non-ASCII names, invalid Unix bytes, Windows separators and unpaired surrogates, special objects, exact tree and flat order, portable-directory incompleteness, three-valued lookup, maintained totals, and capped totals. Compare reviewed expected results rather than provider-to-provider agreement alone.
