---
type: is
id: is-01kzx1ayzr1y0jfja0et8gzybq
title: "Phase 2e: Run evidence-gated basic-content performance iterations"
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bgtkqya7jat1t5z11zpx
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:39.831Z
updated_at: 2026-08-13T12:03:03.010Z
closed_at: 2026-08-13T09:32:29.837Z
close_reason: Added cold/warm/cache-hit/query/default-off/self-host performance jobs, captured the immutable self-host profile and paired semantic digests, and recorded exp-040 rejecting the serial fast path after a 66.34% wall regression.
---
Only after the Phase 2 semantic lock, extend perf_probe.rs, corpora.json, scenarios.json, schemas, realtree, and the experiment ledger with content-disabled, content-basic-cold, warm-fs, cache-hit, churn-1pct, binary-gate, and immutable selfhost-content jobs. Profile, preregister one-mechanism hypotheses, use at least 12 paired interleaved trials, retain semantic digests, and record accepted and rejected changes.
