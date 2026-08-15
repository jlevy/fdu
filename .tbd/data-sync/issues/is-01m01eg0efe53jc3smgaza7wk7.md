---
type: is
id: is-01m01eg0efe53jc3smgaza7wk7
title: "H89: Qualify adaptive scheduling across macOS filesystem-cache states"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
  - experiment
  - macos
dependencies:
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:52:34.382Z
updated_at: 2026-08-15T00:52:41.414Z
---
Measure the worker-policy candidates in the macOS cache states they claim to handle. Keep warm-steady as the primary interactive regime, label /usr/sbin/purge runs only as purge-cold diagnostics, and use a dedicated ordinary APFS test volume with a repeatable unmount/remount or equivalent verified preparation for stronger cold evidence. Do not use a RAM disk for device-latency conclusions. Record preparation success per sample, device/filesystem facts, tree fingerprint, policy history, wall/resources, and invalid runs.

Determine whether the same controller and thresholds cover warm-steady and latency-bound cold scans or whether the evidence supports a documented platform/cache-state strategy. Feed the result into H86; do not generalize from one state, external volume, or unverified purge. Preserve the existing exact-oracle, paired/interleaved, scratch-state, and temporary-volume lifecycle rules.
