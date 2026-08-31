---
type: is
id: is-01m18sm5t0z212c6enknd794eb
title: Stand up a recorded macOS fdu-vs-dust comparison on nominated real trees
kind: task
status: in_progress
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - macos
dependencies: []
created_at: 2026-08-30T07:37:28.125Z
updated_at: 2026-08-31T04:15:22.477Z
---
The fdu-vs-dust question came up from field reports and was first answered with ad-hoc timing, which produced one wrong conclusion (a cold fdu run compared against a warm dust run made dust look 2.4x faster on ~; warm-vs-warm fdu is roughly twice as fast). The project already has the right instrument and it was not reached for first.

Use 'make perf-compare-tools' with PERF_TOOL_CONTROL naming an immutable fdu binary outside the tree, and '--tool dust:dust=<path>'. Note the harness invokes dust as '{binary} -d 1 --no-progress {root}', so any hand-run comparison that leaves the progress spinner on is not comparable.

Known harness behaviour worth recording: on a small subject every fdu sample is invalidated with "adaptive worker policy was not observable: 'undecided'" because the walk ends before the adaptive calibration window completes. That is fdu-9tul showing up in the measurement path, and it makes small trees unusable as comparison subjects.

Scope: nominate ~, ~/wrk, and ~/Library as macOS subjects; run paired and interleaved; record with 'make perf-record'; republish with 'make perf-ledger' and 'make perf-report'.

Acceptance: a recorded artifact exists for the macOS trees with regime stated (platform, host, cache state); the ~/Library case where dust currently leads is either explained or filed; no published fdu-vs-dust figure rests on an unpaired or cache-uncontrolled run.

## Notes

TIMING CORRECTION. An intermediate hand-run measurement of mine reported fdu default at 0.380 s and dust at 0.180 s, i.e. dust twice as fast. That was wrong twice over:

1. It used dust's REFERENCE argv ('-d 1 --no-progress' from measure.py REFERENCE_ARGV) rather than dust's claim-grade CONTRACT argv in compare_tools.py ('-d 0 -n 1 --no-progress --no-colors --no-percent-bars --print-errors --output-format b'). Those are different commands and only the second is comparable.
2. It ran on a machine still loaded from a preceding 12-run cache-clearing sweep.

Re-measured clean, n=9 each, same tree, exact contract argv:
  fdu  --color never ROOT      median 0.150 s (min 0.140 max 0.190)
  dust contract argv           median 0.230 s (min 0.180 max 0.300)
  dust reference argv          median 0.180 s (min 0.170 max 0.190)

This reproduces the recorded harness run (fdu 0.157 s, dust 0.197 s) and confirms the harness numbers stand. fdu default is faster than dust on this subject on point estimate, with the cache ON.

Standing caveat: the harness's 95% interval on the default-tree run was -12.8% to +27.2%, which crosses zero, so 'faster' is not confirmable under the release rule even though every point estimate favours fdu. The resource verdict is the one that was decided: fdu inferior on peak_rss_bytes and minor_faults.
