---
type: is
id: is-01kzn04cg15xabhzvnnnmraqs3
title: Pin claim-grade build and host provenance manifests
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzkzmsegmx4sfswka2084se6
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
  - type: blocks
    target: is-01m01fpdn3qbdv0y2458brk4mw
  - type: blocks
    target: is-01m01ebgg70g940yrq758647t0
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-10T04:50:40.256Z
updated_at: 2026-08-15T01:17:12.777Z
---
Add strict operator-supplied and machine-verified provenance for release evidence: exact build argv/profile/toolchain/target/lockfile/source state, executable hashes, anonymous host class, CPU/memory/filesystem/kernel, collector capabilities, and a manifest identity. Reject incomplete or contradictory claim-grade inputs; keep portable exploratory runs explicitly local-uncontrolled. This blocks compatible release comparisons and published claims, not local correctness or scale spikes.
