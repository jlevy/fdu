---
type: is
id: is-01m3q1rz8d21m89rk9xr9g0sb8
title: Lint and test the Linux native reader on aarch64
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T17:00:39.564Z
updated_at: 2026-09-29T17:00:39.564Z
---
The manylinux aarch64 wheel ships H169's unsafe getdents64/statx reader, which has never executed: CI tests x86_64 only, the aarch64 wheel skips its smoke test (release.yml), and make cross-lint lacks the target. Add aarch64-unknown-linux-gnu to CROSS_TARGETS and run the linux_dents tests on an arm64 runner before 0.2.2.
