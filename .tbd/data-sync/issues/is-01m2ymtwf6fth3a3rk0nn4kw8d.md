---
type: is
id: is-01m2ymtwf6fth3a3rk0nn4kw8d
title: "Linux performance iteration after #94"
kind: epic
status: open
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels:
  - linux
  - campaign-2
dependencies: []
child_order_hints:
  - is-01m2ymtxbb912xhtk8q3ac0fpn
  - is-01m2ymtxtgz7xjqrsg5hfvkkxh
  - is-01m2ymty2v5bt2d0s3h3s6mty8
  - is-01m2ymtyay315j2ae8fmv8fshx
  - is-01m2yq53kmpg4n2gpyd5ewby0m
  - is-01m2yrt6de9vp553ce85zz3zy8
  - is-01m2ys9y5ep69vq65z0xs0ax19
created_at: 2026-09-20T05:32:44.390Z
updated_at: 2026-09-20T06:54:08.145Z
---
Stacked on #94. Queue recorded: H144 same (exp-144), H145 same (exp-145), H84 silent (exp-146), H146 same (exp-147). No leftover named a skippable >=3% userspace cut. H147 not minted. Do not ship PORTABLE. fdu-tk1b stays open.

## Notes

Queue recorded through exp-151. Leftovers H144-H146 same. H84 silent. H85 rejected at 20% (exp-150). H147 accepted 3% keep of recycle (exp-151). Engine patch 5c6e6394. Next free exp-152 / H148. fdu-tk1b stays open.
