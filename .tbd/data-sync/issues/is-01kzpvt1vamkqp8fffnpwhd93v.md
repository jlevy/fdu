---
type: is
id: is-01kzpvt1vamkqp8fffnpwhd93v
title: Profile and optimize snapshot-absent real-tree traversal
kind: task
status: closed
priority: 1
version: 17
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
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
  - is-01kztzfvcgsf3nd7tt5z3mh9fr
  - is-01kztzsap3wvna2dg5kf03qgfe
created_at: 2026-08-10T22:13:36.233Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-12T13:24:49.687Z
close_reason: "Cold real-tree loop complete through exp-032: exact cumulative cold-index wall -53.59% and producer -57.87% versus b565882. Accepted H31 adaptive depth and H3/H26 macOS bulk metadata; post-BFS root-openat, excessive workers, staging reuse, and larger buffers were measured and rejected. Remaining material work is platform-specific Linux evidence or a bounded parent-dirfd design, not another unprofiled constant tweak."
---
Using the mutation-detecting real-tree baseline, profile snapshot-absent scan production, scan plus index, and end-to-end CLI completion separately. Attribute time to enumeration, metadata, allocation/index application, sorting/rendering, and process startup with OS-native profiles and phase probes. For each candidate, run interleaved before/after trials, require exact oracle parity, commit one accepted improvement at a time, and record rejected changes when gains are within noise or complexity is disproportionate. Do not call normal filesystem-cache state cold; controlled-cold requires the dedicated-host protocol.

## Notes

Cold loop through exp-025: H14/H18/H32 landed; H17/H13 and several adaptive triggers rejected; H31 service-time adaptive workers landed; H3/H26 macOS bulk metadata landed. H2 root-relative openat was neutral and reverted (exp-024). Re-running BFS worker depth after bulk metadata showed the old 16-worker target now regresses indexed wall 19.19%, CPU 107%, RSS 33%; current calibration correctly stays at six (exp-025). Final Rust-1.85-compatible exp-032 exact anchor versus b565882: cold index -54.53% and producer -60.05%.
