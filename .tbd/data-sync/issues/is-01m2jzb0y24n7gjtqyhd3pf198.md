---
type: is
id: is-01m2jzb0y24n7gjtqyhd3pf198
title: "PR #61 review PR61-SMOKE-2: crate smoke runs on the runner's python3 and ignores a CARGO override"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2jzadk7w8m1xcsewzwg5wj1
created_at: 2026-09-15T16:45:25.824Z
updated_at: 2026-09-15T17:10:16.712Z
closed_at: 2026-09-15T17:10:16.711Z
close_reason: "7f30d95, 4db083b: smoke_crate fails fast below Python 3.12 (lazy tomllib import so the guard is reached), takes --cargo, Makefile passes $(CARGO); guard unit test and make -n test"
resolution: null
duplicate_of: null
---
PR #61 at eb89150, .github/workflows/release.yml:75 and scripts/release/smoke_crate.py:48,96. The smoke needs Python 3.12+ (tarfile extractall filter=data; tomllib needs 3.11) but the workflow runs it with the runner's python3; smoke_crate defaults cargo='cargo' with no override, so make CARGO=... release-rehearse does not reach it. Fail fast on an old interpreter (tested) or pin the interpreter in the step, and honour a CARGO override.
