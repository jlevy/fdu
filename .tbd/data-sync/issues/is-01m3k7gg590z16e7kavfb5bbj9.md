---
type: is
id: is-01m3k7gg590z16e7kavfb5bbj9
title: "macOS: regression-check stack 141 (H156, --cache auto) and refresh macOS numbers"
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-27-macos-performance-rerun.md
labels:
  - performance
  - macos
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
  - type: blocks
    target: is-01m3kfjccc35r78ty913v0xfec
created_at: 2026-09-28T05:23:55.688Z
updated_at: 2026-09-28T07:45:11.801Z
---
Stack 141 (main <- #137 <- #138 <- #139) was re-benchmarked on Linux only. On internal APFS, per the macOS rerun plan's storage rules, with immutable release builds of the three stack heads:
1. H156 (detached one-shot index release, #138): pair #137 vs #138 on the cache-off indexed tree and default fdu . --cache off, with summary mode as the placebo. Linux measured -7.3% / -7.0% / +0.5%. macOS uses a different allocator, so check for regression, and record an experiment if it moves more than the 3% rule.
2. --cache auto (#139): pair #138 vs #139 on default fdu . and default --view summary, in both regimes: an earlier snapshot exists, and first run into an empty cache directory. Also confirm --cache on and --stale-ok. Linux: fdu . 1.72 -> 1.24 s from empty; 1.51 -> 1.24 s repeated.
3. Refresh the README macOS speed table and report-2026-09-26 figures on the frozen stack head if they moved; keep the table's --cache off contract explicit.
4. H153 confirmation stays with fdu-9e9d.
Next free ids at the stack top: exp-164, H161. Do not generalize macOS results to Linux, or the reverse.

## Notes

2026-09-28 00:45 macOS run launched with the user's authorization (lead: stack 141 frozen). Branch claude/macos-rerun-2026-09-28.
Builds (release, --locked, own worktree + target each, external SSD): fdu 4e008e78 sha256 5d266715..., ea786683 3dab589a..., a5c0ab46 f928319e...; probes perf_probe-{4e008e78,ea786683,a5c0ab46}, perf_probe-h153-control (a5c0ab46 with the shared metric pass disabled).
Subject: 2026-09-26 internal tree had been cleaned up, so balanced 1M regenerated on internal APFS: semantic digest 4bbd97c0d3d4e2ad, 1,000,001 entries, 875,000 files, same sizes as 2026-09-26. Peers: all nine hashes identical to the 2026-09-26 manifest.
Pre-registered (PREDICT): exp-164 = H156 macOS replication, stacked probe run pr137->pr138, primary default-tree wall, placebo cold-scan-index; regression line = >3% worse on an indexed job. exp-165 = H160 macOS replication, pr138->pr139 in the same interleaved run, primary default-tree wall (snapshot present), default-tree-first = first run, placebo cold-scan-index. H153 confirmation stays under fdu-9e9d (content-query, 100 reports, 12 pairs, quiet gate). Peer table: compare_tools anchor a5c0ab46 --cache off indexed tree, 9 peers + fdu default (#139), old default (#138), #137 --cache off; 12 pairs, 3 warmups. Every cell attempts the 25% quiet gate first; a refused start is recorded, then the cell runs declared uncontrolled (cells 1-3 only).
