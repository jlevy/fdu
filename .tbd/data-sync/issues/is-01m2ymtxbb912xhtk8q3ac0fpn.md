---
type: is
id: is-01m2ymtxbb912xhtk8q3ac0fpn
title: "H144: Linux cache-hit leftover after landed stack"
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels:
  - linux
  - campaign-2
dependencies:
  - type: blocks
    target: is-01m2ymtxtgz7xjqrsg5hfvkkxh
  - type: blocks
    target: is-01m2ymtyay315j2ae8fmv8fshx
parent_id: is-01m2ymtwf6fth3a3rk0nn4kw8d
created_at: 2026-09-20T05:32:45.290Z
updated_at: 2026-09-20T05:45:36.299Z
closed_at: 2026-09-20T05:45:36.299Z
close_reason: "exp-144 quiet: same leftover as Darwin H134; apply ~80ms; no new userspace cut. Do not retry H125-H133."
---
content-cache-hit leftover on reconstructible linux-v6.12. Darwin H134 said leftover is already-landed restore work. Name whether Linux still has userspace ≥3%. Do not retry H125/H129/H131/H133. First experiment id exp-144.
