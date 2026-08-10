---
type: is
id: is-01kzpvt1vamkqp8fffnpwhd93v
title: Profile and optimize snapshot-absent real-tree traversal
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt29hvtsrg1pyrq20awxa
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.233Z
updated_at: 2026-08-10T22:13:43.380Z
---
Using the mutation-detecting real-tree baseline, profile snapshot-absent scan production, scan plus index, and end-to-end CLI completion separately. Attribute time to enumeration, metadata, allocation/index application, sorting/rendering, and process startup with OS-native profiles and phase probes. For each candidate, run interleaved before/after trials, require exact oracle parity, commit one accepted improvement at a time, and record rejected changes when gains are within noise or complexity is disproportionate. Do not call normal filesystem-cache state cold; controlled-cold requires the dedicated-host protocol.
