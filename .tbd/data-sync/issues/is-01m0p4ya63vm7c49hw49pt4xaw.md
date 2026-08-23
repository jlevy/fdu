---
type: is
id: is-01m0p4ya63vm7c49hw49pt4xaw
title: "make perf-floor: the tier-by-subject floor scoreboard"
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - campaign-2
dependencies: []
created_at: 2026-08-23T01:49:40.418Z
updated_at: 2026-08-23T09:09:01.467Z
---
Campaign 2 orders work by each tier's measured distance to the parallel syscall floor,
which makes that distance the scoreboard -- and today deriving it is a by-hand session
with the spikes. Add a harness entry point (make perf-floor) that:

- builds parfloor and peerwalk (benchmarks/spikes/) and the fdu binary
- runs the floor variants and the fdu tiers (aggregate, index, cache-only) across the
  nominated real-tree subject set plus the standard generated subject, paired and
  interleaved, with the shared tally oracle enforced
- emits the x-floor table per tier per subject, in a committed or easily diffable form

Every accepted change re-runs it, which is what makes the shared-cost re-screen and the
termination criteria in the campaign-2 plan checkable rather than asserted. The floor
report (docs/project/reports/report-2026-08-23-metadata-walk-floor.md) documents the
instruments and the protocol this automates.

## Notes

Blocked on a design decision, not on effort. parfloor.c -- the denominator every x-floor threshold in campaign 2 is defined against -- is Linux-only (SYS_getdents64, statx; no Darwin equivalents). arena_spike.rs and peerwalk.rs are portable. So a macOS scoreboard needs either a getattrlistbulk port of the floor or a different floor set with the regime difference recorded, and that should be decided in the plan rather than by a harness falling back. The Linux half is straightforwardly scriptable today.
