---
type: is
id: is-01kzpvt1vamkqp8fffnpwhd93v
title: Profile and optimize snapshot-absent real-tree traversal
kind: task
status: in_progress
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt29hvtsrg1pyrq20awxa
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.233Z
updated_at: 2026-08-11T02:09:09.560Z
---
Using the mutation-detecting real-tree baseline, profile snapshot-absent scan production, scan plus index, and end-to-end CLI completion separately. Attribute time to enumeration, metadata, allocation/index application, sorting/rendering, and process startup with OS-native profiles and phase probes. For each candidate, run interleaved before/after trials, require exact oracle parity, commit one accepted improvement at a time, and record rejected changes when gains are within noise or complexity is disproportionate. Do not call normal filesystem-cache state cold; controlled-cold requires the dedicated-host protocol.

## Notes

Portable sweep concluded. Landed: H14 (-7.09% warm), H18 (-15.65% cold), H32 (-12.4% load component) as 92d6212/bb1529d/9f4f029. Cleanly refuted with tight intervals: H17 (nothing left after H14) and H12-serial-part (already landed as abeb377's elision), H13 (H18 captured the same cost). The portable userland backlog at 60k-warm is now spent - remaining levers are the syscall rung (rustix decision), the H12 parallel form (parallel sweep + elision together, revisit exp-002 at scale/cold per H37), snapshot format work (H33/H34), and memory packing (H19-H22). See exp-010/exp-011 for the negative evidence.
