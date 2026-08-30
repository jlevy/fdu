---
type: is
id: is-01m18sm5t0z212c6enknd794eb
title: Stand up a recorded macOS fdu-vs-dust comparison on nominated real trees
kind: task
status: in_progress
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - macos
dependencies: []
created_at: 2026-08-30T07:37:28.125Z
updated_at: 2026-08-30T07:55:57.819Z
---
The fdu-vs-dust question came up from field reports and was first answered with ad-hoc timing, which produced one wrong conclusion (a cold fdu run compared against a warm dust run made dust look 2.4x faster on ~; warm-vs-warm fdu is roughly twice as fast). The project already has the right instrument and it was not reached for first.

Use 'make perf-compare-tools' with PERF_TOOL_CONTROL naming an immutable fdu binary outside the tree, and '--tool dust:dust=<path>'. Note the harness invokes dust as '{binary} -d 1 --no-progress {root}', so any hand-run comparison that leaves the progress spinner on is not comparable.

Known harness behaviour worth recording: on a small subject every fdu sample is invalidated with "adaptive worker policy was not observable: 'undecided'" because the walk ends before the adaptive calibration window completes. That is fdu-9tul showing up in the measurement path, and it makes small trees unusable as comparison subjects.

Scope: nominate ~, ~/wrk, and ~/Library as macOS subjects; run paired and interleaved; record with 'make perf-record'; republish with 'make perf-ledger' and 'make perf-report'.

Acceptance: a recorded artifact exists for the macOS trees with regime stated (platform, host, cache state); the ~/Library case where dust currently leads is either explained or filed; no published fdu-vs-dust figure rests on an unpaired or cache-uncontrolled run.

## Notes

Two recorded runs now exist on rustup-toolchains, both clean (0 invalid samples, 0 semantic mismatches, 0 oracle mismatches, no baseline drift, no mutation): run-fdu-vs-dust-rustup and run-fdu-default-vs-dust-rustup.

Confirmed subject-selection rules the hard way, see prior notes: symlinks == 0 is mandatory for any dust comparison, the tree must be quiescent, and it must be large enough that fdu's adaptive calibration window completes (cargo-registry-src at 5,838 entries is too small; rustup-toolchains at ~119k works).

Still open: ~ and ~/Library are not yet covered, and both are symlink-bearing and live, so neither can be compared against dust under its claim-grade contract. Decide whether to compare those against du/gdu instead, or to build a quiescent macOS subject that resembles them in shape. Artifacts are in /tmp and are not yet recorded via 'make perf-record' or republished.
