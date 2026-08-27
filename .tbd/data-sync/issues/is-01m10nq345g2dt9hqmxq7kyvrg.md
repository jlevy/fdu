---
type: is
id: is-01m10nq345g2dt9hqmxq7kyvrg
title: Prove Python GIL release, lifecycle, typing, and installed-wheel behavior
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1sf2nph021wtx28p8ahxh
created_at: 2026-08-27T03:55:13.924Z
updated_at: 2026-08-27T05:35:13.180Z
closed_at: 2026-08-27T05:35:13.180Z
close_reason: Implemented and verified the direct opened-root Python surface at 0583a1a/fa85812. Full make check, cross-lint, installed wheel/sdist lifecycle and typing, CLI/Python parity, MSRV, and the complete GitHub Actions matrix all pass. The standalone CLI raw stripped size is unchanged and its golden corpus remains green; no runtime dependency was added.
resolution: null
duplicate_of: null
---
Extend public_smoke.py, run_concurrency.py, typecheck/consumer.py, one-shot parity, and wheel/sdist smoke coverage with one complete five-operation lifecycle. Prove a blocked change poll does not starve Python, reads overlap native commits, clones share close, concurrent close has one result, post-close operations fail predictably, stubs match exports, CLI/Python one-shot answers remain normalized-equivalent, and no worker survives installed-wheel shutdown.

## Notes

Implemented installed-wheel lifecycle coverage for mixed coherent reads, paging, change polling, refresh, foreign identities, shared concurrent close, and post-close errors; embedded Rust coverage proves a blocked Python changes call releases the GIL. Full make check and both cross-lint targets passed before the final timeout-boundary self-review correction; rerun the focused Python gates and final handoff before closure.
