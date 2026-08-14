---
type: is
id: is-01m0129sjsp6xht1ae8aebvx0r
title: Build the portable abi3 matrix and artifact-first release workflow
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0129t2wsdsv20mt3bq7s0zh
parent_id: is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-14T21:19:27.832Z
updated_at: 2026-08-14T21:58:40.211Z
---
Build one top-level, non-uploading-or-publishing release graph around exact tag/manifest identity. Use tested checked-in scripts for plan resolution, artifact inspection, registry-state comparison, and archive assembly. Build manylinux2014 x86-64/arm64, macOS x86-64/arm64, and Windows x86-64 abi3 wheels plus an install-tested sdist and crate preview. Promote immutable Python artifacts; reproduce Cargo's package from the exact source bundle and verify its registry checksum. Keep build jobs unprivileged, put OIDC only in direct protected publisher jobs, pin all actions and tools, classify registry state as missing/identical/conflict, and emit checksums, attestations, SBOM evidence, and the GitHub release after registry verification.
