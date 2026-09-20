---
type: is
id: is-01m2ymtwf6fth3a3rk0nn4kw8d
title: "Linux performance iteration after #94"
kind: epic
status: open
priority: 1
version: 14
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
  - is-01m2yssjtfcrct87cd9bc8czy8
created_at: 2026-09-20T05:32:44.390Z
updated_at: 2026-09-20T07:26:20.198Z
---
Stacked on #94. Recorded leftover queue exp-144–153: H144/H145/H146 same leftovers; H84 silent (named-job --threads 8 not a 3% win; default /usr +7.12% regression); H85 rejected 20% (exp-150); H147 recycle keep (exp-151, -4.98%); H72 rejected on v6.12 (exp-152, -1.63%) and accepted on /usr (exp-153, -9.01%). Engine kept f841662c. Do not ship PORTABLE. Next free exp-154 / H148. fdu-tk1b stays open.

## Notes

Queue through exp-153. Leftovers H144-H146 same. H84 silent. H85 rejected 20% (exp-150). H147 recycle keep (exp-151). H72 rejected on v6.12 (exp-152) and accepted on /usr (exp-153). Engine f841662c. Next free exp-154 / H148. fdu-tk1b stays open.
