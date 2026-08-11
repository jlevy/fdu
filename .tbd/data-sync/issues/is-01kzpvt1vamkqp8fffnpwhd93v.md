---
type: is
id: is-01kzpvt1vamkqp8fffnpwhd93v
title: Profile and optimize snapshot-absent real-tree traversal
kind: task
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt29hvtsrg1pyrq20awxa
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.233Z
updated_at: 2026-08-11T01:30:56.274Z
---
Using the mutation-detecting real-tree baseline, profile snapshot-absent scan production, scan plus index, and end-to-end CLI completion separately. Attribute time to enumeration, metadata, allocation/index application, sorting/rendering, and process startup with OS-native profiles and phase probes. For each candidate, run interleaved before/after trials, require exact oracle parity, commit one accepted improvement at a time, and record rejected changes when gains are within noise or complexity is disproportionate. Do not call normal filesystem-cache state cold; controlled-cold requires the dedicated-host protocol.

## Notes

bb1529d landed H18 extension interning: cold-scan-index -15.65% paired [-32.77,-0.78] over the previous commit (exp-008). H14 committed as simplification (92d6212), wall verdict pending re-measurement (exp-007 in-progress). H32 single-pass snapshot load held as patch (exp-009 in-progress). Remaining from the backlog before platform work: H13 per-directory apply, H17 merge-join, H12 producer no-op elision (the big warm restructure), H19-H22 packing. H2/H3 still blocked on the rustix decision - research recommends rustix over raw libc, which keeps unsafe_code=deny intact.
