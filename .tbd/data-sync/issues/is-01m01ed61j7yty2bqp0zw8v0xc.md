---
type: is
id: is-01m01ed61j7yty2bqp0zw8v0xc
title: Run the macOS release-CLI non-inferiority matrix against dust
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - validation
dependencies:
  - type: blocks
    target: is-01m01edfz3bd7x2w91bh4qft2m
  - type: blocks
    target: is-01m01edt62s6s8mfeyqgykasxq
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:51:01.809Z
updated_at: 2026-08-15T00:51:22.433Z
---
Qualify the actual installed fdu command, not only perf_probe, against pinned dust on the representative macOS workload matrix. Use matched human tree work, cache-off first-run semantics, immutable exact-oracle subjects, adjacent paired/interleaved release processes, quiet-host warm-steady cells, separately labeled interactive-host cells, and diagnostic partial-result cells. Preserve the richer-work distinction when fdu builds reusable state, but test the natural command the user runs.

Pre-registered release gate: no supported representative fixture may show fdu more than 3% slower than dust with the entire paired 95% interval above zero; any such cell blocks the performance conclusion until explained and fixed or explicitly scoped out with product justification. Record CPU, system CPU, RSS, faults, context switches, policy decisions, exact totals, errors, versions, hashes, and tree fingerprints. A single headline corpus cannot satisfy this bead.
