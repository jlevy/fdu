---
type: is
id: is-01kzrtmnanw8e9kx7wn2etbhdv
title: "Adaptive worker pool: in-flight depth from cache-capacity state"
kind: task
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53e2qffv7d9a2q7vg2yth
  - type: blocks
    target: is-01kzrv2fca4h3rb4mtf263b5gc
created_at: 2026-08-11T16:31:39.861Z
updated_at: 2026-08-11T16:39:36.687Z
---
The automatic worker cap of 6 was measured honestly on a WARM 60k tree (exp-001: knee at 4, 8 worse than 4) and is roughly 2x too low on a COLD large tree, which is the whole-drive use case. Evidence: a real home folder (4,366,510 files, 1,016,449 dirs, 224 GiB) took 791 s wall for only 175 s of CPU - 78% blocked, achieved parallelism 0.22x, 923k voluntary context switches, 2.65 GB RSS (~493 B/entry, matching the research estimate). On a cold ~795k-entry subtree, interleaved rounds gave medians 33.7 s at 6 workers, 17.0 s at 16, 19.9 s at 32 - about 2x from in-flight depth alone. Fix: select pool size from the same metadata-cache capacity signal the adaptive cache policy uses (kern.maxvnodes vs recorded entry count; dentry-state on Linux). Small tree fits cache -> syscall-bound -> keep the low cap; large tree misses cache -> latency-bound -> raise depth toward device saturation. Also raise MAX_SCAN_THREADS, which is 32 and has not been probed at its top. This is the frontier research's H31 with a concrete trigger. NOTE ON SEQUENCING: this is worth more than journal resume for whole-drive usage and is orthogonal to it - a first scan has no cursor, so only this can help it, and it lands on every platform. Should precede scoped revalidation.
