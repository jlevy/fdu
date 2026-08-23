---
type: is
id: is-01m0p4ya63vm7c49hw49pt4xaw
title: "make perf-floor: the tier-by-subject floor scoreboard"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
created_at: 2026-08-23T01:49:40.418Z
updated_at: 2026-08-23T01:49:40.418Z
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
