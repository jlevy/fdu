---
type: is
id: is-01m18sm5t0z212c6enknd794eb
title: Stand up a recorded macOS fdu-vs-dust comparison on nominated real trees
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - macos
dependencies: []
created_at: 2026-08-30T07:37:28.125Z
updated_at: 2026-08-30T07:37:28.125Z
---
The fdu-vs-dust question came up from field reports and was first answered with ad-hoc timing, which produced one wrong conclusion (a cold fdu run compared against a warm dust run made dust look 2.4x faster on ~; warm-vs-warm fdu is roughly twice as fast). The project already has the right instrument and it was not reached for first.

Use 'make perf-compare-tools' with PERF_TOOL_CONTROL naming an immutable fdu binary outside the tree, and '--tool dust:dust=<path>'. Note the harness invokes dust as '{binary} -d 1 --no-progress {root}', so any hand-run comparison that leaves the progress spinner on is not comparable.

Known harness behaviour worth recording: on a small subject every fdu sample is invalidated with "adaptive worker policy was not observable: 'undecided'" because the walk ends before the adaptive calibration window completes. That is fdu-9tul showing up in the measurement path, and it makes small trees unusable as comparison subjects.

Scope: nominate ~, ~/wrk, and ~/Library as macOS subjects; run paired and interleaved; record with 'make perf-record'; republish with 'make perf-ledger' and 'make perf-report'.

Acceptance: a recorded artifact exists for the macOS trees with regime stated (platform, host, cache state); the ~/Library case where dust currently leads is either explained or filed; no published fdu-vs-dust figure rests on an unpaired or cache-uncontrolled run.
