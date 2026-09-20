---
type: is
id: is-01m2ymty2v5bt2d0s3h3s6mty8
title: "H146: Linux first-run leftover after H140"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels:
  - linux
  - campaign-2
dependencies: []
parent_id: is-01m2ymtwf6fth3a3rk0nn4kw8d
created_at: 2026-09-20T05:32:46.042Z
updated_at: 2026-09-20T06:10:36.796Z
closed_at: 2026-09-20T06:10:36.796Z
close_reason: "Same leftover identity as Darwin H136 (exp-147, quiet): walk 93% of first-run; isolated save ~24ms is >=3% and not skippable; do not retry H100."
---
H136 analog. default-tree-first on linux-v6.12. Darwin leftover was still the walk; snapshot write not skippable. Linux write cost may move; skippability is the question. Do not retry H100. Do not load a snapshot on fdu PATH.
