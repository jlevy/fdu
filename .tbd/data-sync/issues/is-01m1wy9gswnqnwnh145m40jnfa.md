---
type: is
id: is-01m1wy9gswnqnwnh145m40jnfa
title: Validate the timed opened index in the discovery performance oracle
kind: bug
status: in_progress
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - review
  - validation
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
  - type: blocks
    target: is-01m1x444e4rnksjs8v8p37padv
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T03:23:50.459Z
updated_at: 2026-09-07T05:22:48.307Z
---
PR 52 final review R1, head 5d7b86f: crates/fdu-core/examples/perf_probe.rs:960-970 fills engine_digest and retained tallies from a separate scan_into_index, never from the OpenedIndex just timed and closed. The index-digest oracle in explorations/benchmarks/realtree/tree.py:216-241 ignores streamed apply/commit fields, so missing or incorrect opened facts can be accepted as valid performance samples. Negative check on the 8-file tests directory accepted the correct detached digest combined with zero streamed inserts and an empty commit summary. Collect the opened final state through public reads at one terminal version before close, compare it and replayed exact changes with the independent oracle outside the timing interval, and add negative cases that drop or alter an opened entry. Do not require identical batching/debug digest for backend equivalence; compare semantic results and contract invariants.

## Notes

Implementation now reads the measured OpenedIndex through version-checked public Lookup/Diagnostics/Flat/Continue projections before joined close; the independent Python fingerprint validates those rows, not a fresh scan. Missing-entry regression fails on the old probe (reported 2 files while the opened root retained 1) and passes after the fix. Empty/multipage parity, unavailable-version rejection, and unverified profile cases pass. Validation time and calling-thread counters are excluded; process-wide worker folds remain included via a read-only thread_snapshot counter accessor. Exact causal replay remains in the independent engine reference-model/golden tests; the probe commit Debug digest is explicitly diagnostic, not a backend-equivalence constraint. Full gates and CI pending.
