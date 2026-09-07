---
type: is
id: is-01m1wy9gswnqnwnh145m40jnfa
title: Validate the timed opened index in the discovery performance oracle
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - review
  - validation
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T03:23:50.459Z
updated_at: 2026-09-07T03:29:18.450Z
---
PR 52 final review R1, head 5d7b86f: crates/fdu-core/examples/perf_probe.rs:960-970 fills engine_digest and retained tallies from a separate scan_into_index, never from the OpenedIndex just timed and closed. The index-digest oracle in explorations/benchmarks/realtree/tree.py:216-241 ignores streamed apply/commit fields, so missing or incorrect opened facts can be accepted as valid performance samples. Negative check on the 8-file tests directory accepted the correct detached digest combined with zero streamed inserts and an empty commit summary. Collect the opened final state through public reads at one terminal version before close, compare it and replayed exact changes with the independent oracle outside the timing interval, and add negative cases that drop or alter an opened entry. Do not require identical batching/debug digest for backend equivalence; compare semantic results and contract invariants.

## Notes

End-to-end fault injection confirmed: in an isolated checkout, changed only the opened probe options to prune hidden names and ran it on a fixture containing visible.txt and .hidden.txt. The actual opened stream inserted 1 file, the reported independent-scan summary contained 2 files, and tree.probe_agrees returned None (accepted). The opened probe must compare its measured retained state, not merely attach an oracle-produced digest. Review repro diff retained at /tmp/fdu-pr52-review-probes.patch; no production branch edits.
