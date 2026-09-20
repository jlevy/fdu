---
type: is
id: is-01m2y4f6c3ea0q3ngn99nz7eek
title: "H142: Linux first-pass analyze leftover"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
  - H142
dependencies: []
parent_id: is-01m2y4f4g34vdbgxf0jcvt8dw3
created_at: 2026-09-20T00:46:44.098Z
updated_at: 2026-09-20T04:39:17.073Z
closed_at: 2026-09-20T04:39:17.073Z
close_reason: "exp-143: same leftover identity; content-basic still file I/O (86634 opens, 184057 reads); do not retry H124"
---
Optional after H139-H141. content-basic leftover. Determination only. Do not retry H124.

## Notes

PREDICT (2026-09-20, Linux KVM, exp-143)

Hypothesis: H142 Linux first-pass content-basic leftover after H124.
Job: same-binary 12-pair content-basic on reconstructible linux-v6.12 (86,643 files).
Attribution: FDU_COUNTERS=1 hits (file opens, read calls, classify/commit share).
Determination: leftover is still file I/O (openat/read), or a userspace stage >=3% Darwin did not see.
Do not retry H124 type/size or read-ahead. Do not retry H118/H119.
Quiet first. Do not lower the 25% bar.
