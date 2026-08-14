---
type: is
id: is-01m0129sjsp6xht1ae8aebvx0r
title: Build the portable abi3 matrix and artifact-first release workflow
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0129t2wsdsv20mt3bq7s0zh
parent_id: is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-14T21:19:27.832Z
updated_at: 2026-08-14T23:36:36.697Z
closed_at: 2026-08-14T23:36:36.696Z
close_reason: "Implemented the authorized non-publishing artifact-first workflow: exact plan resolution, portable five-wheel abi3 matrix, crate/sdist builds, native smokes, full-matrix/content inspection, read-only registry state classification, pinned unprivileged jobs, SBOM/checksum evidence, and tested host rehearsal. Publisher/attestation/GitHub Release work is explicitly transferred to fdu-9cf0."
---
Build a top-level non-publishing release graph around exact tag and manifest identity. Use tested checked-in scripts for plan resolution, artifact inspection, read-only registry-state comparison, and evidence assembly. Build manylinux2014 x86-64/arm64, macOS x86-64/arm64, and Windows x86-64 abi3 wheels plus an install-tested sdist and crate preview. Reuse immutable artifacts, keep build jobs unprivileged, pin actions and tools, and emit checksums and SBOM evidence. Protected publisher jobs, attestations, and the GitHub Release remain part of fdu-9cf0 after external publisher setup.

## Notes

Implemented on codex/python-packaging-release-engineering. release.yml is manual and non-publishing by construction; it builds the five-wheel abi3 matrix, crate, and sdist, smoke-tests native artifacts, validates the exact matrix and artifact contents, classifies public registry state, and retains manifest/checksum evidence. Twelve release-tool tests, make release-rehearse, make check, and Windows cross-lint pass.
