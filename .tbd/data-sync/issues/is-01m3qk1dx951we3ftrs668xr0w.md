---
type: is
id: is-01m3qk1dx951we3ftrs668xr0w
title: "H185: the tree route skips directory and symlink stats (H72's policy on the folded index)"
kind: task
status: in_progress
priority: 2
version: 3
spec_path: docs/project/research/research-2026-09-29-uniformly-faster-than-pdu.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
hold: null
hold_until: null
created_at: 2026-09-29T22:02:22.505Z
updated_at: 2026-09-29T23:08:25.934Z
started_at: 2026-09-29T23:08:07.287Z
---
Predicted -4% to -6% default-tree on node-modules-dense, -2% to -4% on linux-v6.12: 5,829 and 9,544 fewer statx. Exact: the one-shot tree report reads no directory's or symlink's own attributes; dev is read only under --one-filesystem, where the policy keeps the stat. Deciding default-tree on both real trees, 20 pairs; placebos aggregate-summary --no-controls and cold-scan-index; secondary strace -c statx. Brief: docs/project/research/research-2026-09-29-uniformly-faster-than-pdu.md

## Notes

Built c0da65ae on claude/pdu-uniform-lead. strace -c: statx 92,836 -> 87,006 (linux-v6.12), 79,961 -> 70,416 (node-modules-dense); 54 answer comparisons identical; transient-vs-indexed differential gains a --one-filesystem case. Cell (exp-197) pending a quiet host: cells-h185 control perf_probe-final-ebc06c78, candidate perf_probe-h185-c0da65ae.
