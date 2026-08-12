---
type: is
id: is-01kzpvt1vamkqp8fffnpwhd93v
title: Profile and optimize snapshot-absent real-tree traversal
kind: task
status: in_progress
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt29hvtsrg1pyrq20awxa
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
child_order_hints:
  - is-01kztqevexz07mgpf4hcb4ztc9
  - is-01kztvyt9245vp768sgav6dxrx
  - is-01kztwwfnvta9x0vjq9znwpcp4
  - is-01kztxsn4qpkq1y652qtwb6vta
created_at: 2026-08-10T22:13:36.233Z
updated_at: 2026-08-12T12:05:18.102Z
---
Using the mutation-detecting real-tree baseline, profile snapshot-absent scan production, scan plus index, and end-to-end CLI completion separately. Attribute time to enumeration, metadata, allocation/index application, sorting/rendering, and process startup with OS-native profiles and phase probes. For each candidate, run interleaved before/after trials, require exact oracle parity, commit one accepted improvement at a time, and record rejected changes when gains are within noise or complexity is disproportionate. Do not call normal filesystem-cache state cold; controlled-cold requires the dedicated-host protocol.

## Notes

Portable and macOS cold-scan loop through exp-023: H14/H18/H32 landed; H17/H13 and several adaptive triggers rejected; H31 service-time adaptive workers landed; H3/H26 macOS bulk metadata landed with final 60k wall -5.22%/-9.25% and 720k -30.13%/-41.60%. Cumulative vs b565882 is -53.49% cold index and -58.20% producer. Post-H26 profile leaves directory open at 33.86%; H2/openat is next. Warm reconciliation reuse of the bulk reader remains separate.
