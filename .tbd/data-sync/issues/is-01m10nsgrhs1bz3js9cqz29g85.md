---
type: is
id: is-01m10nsgrhs1bz3js9cqz29g85
title: Run the composed recovery and cancellation fault matrix
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nsh2xbhygwsbxgf2mzqrj
parent_id: is-01m0y1sk24z37hnvpxee6apg8e
created_at: 2026-08-27T03:56:33.424Z
updated_at: 2026-08-27T03:56:33.756Z
---
Inject discovery-budget refusal, query limit, stale/evicted/foreign continuation, consumer journal reset, provider observation gap, missing/incompatible package, second iterator, event-loop shutdown, cancellation during poll and close, worker failure, and continuation reuse after root replacement. Require the documented typed recovery with no lost update, false completeness, silent fallback, or leaked worker.
