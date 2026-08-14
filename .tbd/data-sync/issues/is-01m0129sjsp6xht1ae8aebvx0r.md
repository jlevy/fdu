---
type: is
id: is-01m0129sjsp6xht1ae8aebvx0r
title: Build the portable abi3 matrix and artifact-first release workflow
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0129t2wsdsv20mt3bq7s0zh
parent_id: is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-14T21:19:27.832Z
updated_at: 2026-08-14T21:19:41.661Z
---
Build immutable manylinux2014 x86-64 and arm64, macOS x86-64 and arm64, and Windows x86-64 abi3 wheels plus the Python sdist and Rust crate. Pass exact artifacts through inspection, smoke, protected approval, independently retryable PyPI and crates.io jobs, checksums, attestations, SBOM retention, and GitHub release verification. Document PyPI pending trusted publishing and the manual crates.io first-release bootstrap.
