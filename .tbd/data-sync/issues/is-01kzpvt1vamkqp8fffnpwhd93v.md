---
type: is
id: is-01kzpvt1vamkqp8fffnpwhd93v
title: Profile and optimize snapshot-absent real-tree traversal
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzpvt29hvtsrg1pyrq20awxa
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-10T22:13:36.233Z
updated_at: 2026-08-11T00:24:47.750Z
---
Using the mutation-detecting real-tree baseline, profile snapshot-absent scan production, scan plus index, and end-to-end CLI completion separately. Attribute time to enumeration, metadata, allocation/index application, sorting/rendering, and process startup with OS-native profiles and phase probes. For each candidate, run interleaved before/after trials, require exact oracle parity, commit one accepted improvement at a time, and record rejected changes when gains are within noise or complexity is disproportionate. Do not call normal filesystem-cache state cold; controlled-cold requires the dedicated-host protocol.

## Notes

Cold path profiled and optimized. exp-001 accepted a bounded parallel producer: cold-scan-index wall 631->321 ms (-48.9%), walk-only -51.6%, byte-identical engine digests at every thread count, no new dependency. Profiles attribute the remainder to syscalls: open 28%, fstatat 19%, getdirentries 10%. exp-003 rejected and reverted (removing 120k path clones per scan changed nothing measurable). Remaining lever H2/H3 (openat, getattrlistbulk/statx) is blocked on promoting libc to a runtime dependency plus a scoped unsafe allowance; see the hypothesis table in docs/project/guides/performance-loop.md. Not closing until that decision is taken or an alternative is found.
