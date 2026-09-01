---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - validation
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
hold: null
hold_until: null
created_at: 2026-09-01T06:33:23.201Z
updated_at: 2026-09-01T11:23:37.035Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

Second subject selected and fingerprinted: current metabrowser-clone source checkout, 113,794 entries, 15,221 directories, 98,525 files, 48 symlinks, max depth 21, engine digest 58bc2ea1deb0e212c7368177328184d412a1b2da24be8a77a7985a4bf6d4bc64. It is control-rich and materially different from cargo-registry-src-v2. Two controlled timing starts were abandoned after the harness invalidated samples for host pressure; no partial evidence was retained, and a controlled-interactive load process left by interrupt cleanup was terminated. Added deterministic 128-to-256-entry allocation-slope guards for detached scan-index and opened discovery, plus detached zero-work assertions and negative fixtures proving one restored allocation per entry and one restored effect path fail. All 287 performance/probe harness tests pass locally. Quiet parity measurement remains to retry when the host settles.
