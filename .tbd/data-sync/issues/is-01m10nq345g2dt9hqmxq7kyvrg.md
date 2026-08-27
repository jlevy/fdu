---
type: is
id: is-01m10nq345g2dt9hqmxq7kyvrg
title: Prove Python GIL release, lifecycle, typing, and installed-wheel behavior
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1sf2nph021wtx28p8ahxh
created_at: 2026-08-27T03:55:13.924Z
updated_at: 2026-08-27T04:03:07.330Z
---
Extend public_smoke.py, run_concurrency.py, typecheck/consumer.py, one-shot parity, and wheel/sdist smoke coverage with one complete five-operation lifecycle. Prove a blocked change poll does not starve Python, reads overlap native commits, clones share close, concurrent close has one result, post-close operations fail predictably, stubs match exports, CLI/Python one-shot answers remain normalized-equivalent, and no worker survives installed-wheel shutdown.
