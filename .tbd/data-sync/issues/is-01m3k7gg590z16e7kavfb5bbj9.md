---
type: is
id: is-01m3k7gg590z16e7kavfb5bbj9
title: "macOS: regression-check stack 141 (H156, --cache auto) and refresh macOS numbers"
kind: task
status: in_progress
priority: 1
version: 5
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
updated_at: 2026-09-28T11:48:24.339Z
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

2026-09-28 04:50 RESULTS (all cells uncontrolled: every quiet start refused, 31.8% probe, 47.4% peer matrix; unrelated trading/squares/Codex load):
- exp-164 H156 macOS (stacked probe, pr137->pr138): default-tree -1.00% [-5.13,+6.11], first run +0.16% [-5.12,+4.01], placebo cold-scan-index +2.09% [-0.72,+3.62]; user CPU -2.4%. Rejected on macOS, kept candidate; no regression (no median worse than +2.1%).
- exp-165 H160 macOS (pr138->pr139): default-tree -3.09% [-6.75,+2.00], first run -2.98% [-7.12,+5.12], placebo -0.44%; peak RSS -26.29% (381->281 MiB), user CPU -9.65%. Rejected on macOS for wall, kept candidate.
- Peer table (compare_tools, anchor a5c0ab46 --cache off indexed tree, 12 pairs, 3 warmups, 360 processes, 0 invalid, no drift/mutation): fdu 6.371 s (137k files/s, 0.47 GB/s, 285.7 MiB); dumac +8.51% [+3.70,+13.57]; pdu +49.35%; diskus +41.88%; dua +60.79%; gdu +66.97%; dust +57.24% [+51.82,+86.17]; BSD du +801.35%; ncdu +967.50%; GNU du +959.87%. Paired fdu rows: #139 default fdu PATH +0.13% [-4.52,+1.85] (285.8 MiB); #138 default +1.46% [-2.04,+3.84] (385.9 MiB); #137 --cache off +5.62% [-0.08,+10.25] (stack did not regress the published contract).
- Screens (hyperfine, orientation only): --stale-ok 0.69 s vs 6.4-9.5 s walks; cache files as specified at 1M.
- H153 (fdu-9e9d): quiet attempt failed to qualify (20/24 timed samples invalid); no id used; raw run beside exp-159.
Docs on branch claude/macos-rerun-2026-09-28: report-2026-09-26 refreshed in place (+ result JSON 2026-09-28, screens JSON), cache brief, plan status, runbook pickup/standing (next free exp-166, H161), registry rows H153/H156/H160, ledger + evidence page. README numbers handed to the lead for #142 (branch leaves README unedited by instruction).
