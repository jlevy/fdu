---
type: is
id: is-01m3qazjjrwjhc2gq9bswa61da
title: "Code analysis is 6x scc's CPU on linux-v6.12: profile the cold --analyze=code path"
kind: task
status: open
priority: 2
version: 1
labels:
  - performance
  - linux
dependencies: []
created_at: 2026-09-29T19:41:33.143Z
updated_at: 2026-09-29T19:41:33.143Z
---
`fdu --analyze=code` is the slowest of the three counters on Linux v6.12 (fdu-bj94, docs/project/research/research-2026-09-29-sloc-tools-survey.md). Quiet cells, 12 adjacent pairs, 4-vCPU Firecracker ext4, fdu at ebc06c78:

- Ignore rules off (copy without .git): fdu 7.92 s wall, 29.9 CPU-s (27.8 user); scc 4.1.0 1.24 s, 4.8 CPU-s; tokei 15.0.0 1.87 s, 7.2 CPU-s. fdu uses 6.2x scc's CPU.
- Each tool's own .gitignore handling (the clone): fdu 9.22 s, 31.0 CPU-s; scc 1.29 s; tokei 1.98 s.
- All three read about the same bytes: fdu opened 86,611 files and read 1.476 GB (FDU_COUNTERS=1); scc read its 81,820 recognized files, 1.460 GB. The gap is CPU per byte, not I/O.
- A repeated run under the default cache policy takes 0.55 s (0.72 CPU-s), ahead of both peers; the cold path is the problem.

Leads to profile first (single untimed runs, not evidence):
- `--analyze=lines --view=summary` on the same copy takes about 2.5 s wall and 6.7 CPU-s, so the code analyzer itself adds about 5.4 s and 21 CPU-s.
- FDU_COUNTERS on one code run: 2.96 M allocations, 7.39 GB allocated (about 5x the bytes read).
- `--ignored=exclude` adds about 1.3 s over `--no-gitignore` (9.2 vs 7.9 s) while `--ignored=include` adds about 0.2 s; cores busy drops from 3.77 to 3.36, so exclude mode serializes something. The README says reading .gitignore adds 1.6% to the default (metadata) tree; with content analysis the exclude population costs 16%.

Profile before changing anything (AGENTS.md, performance loop): perf/`make perf-content-profile` on a kernel-sized C corpus, then decide. Target: cold code analysis within 2x of tokei on linux-v6.12 without changing any count (the per-file differential in the brief is the oracle: fdu and scc agree on 59,931 of 59,953 C files).
