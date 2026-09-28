---
type: is
id: is-01m3k7gg590z16e7kavfb5bbj9
title: "macOS: regression-check stack 141 (H156, --cache auto) and refresh macOS numbers"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-27-macos-performance-rerun.md
labels:
  - performance
  - macos
dependencies: []
created_at: 2026-09-28T05:23:55.688Z
updated_at: 2026-09-28T05:23:55.688Z
---
Stack 141 (main <- #137 <- #138 <- #139) was re-benchmarked on Linux only. On internal APFS, per the macOS rerun plan's storage rules, with immutable release builds of the three stack heads:
1. H156 (detached one-shot index release, #138): pair #137 vs #138 on the cache-off indexed tree and default fdu . --cache off, with summary mode as the placebo. Linux measured -7.3% / -7.0% / +0.5%. macOS uses a different allocator, so check for regression, and record an experiment if it moves more than the 3% rule.
2. --cache auto (#139): pair #138 vs #139 on default fdu . and default --view summary, in both regimes: an earlier snapshot exists, and first run into an empty cache directory. Also confirm --cache on and --stale-ok. Linux: fdu . 1.72 -> 1.24 s from empty; 1.51 -> 1.24 s repeated.
3. Refresh the README macOS speed table and report-2026-09-26 figures on the frozen stack head if they moved; keep the table's --cache off contract explicit.
4. H153 confirmation stays with fdu-9e9d.
Next free ids at the stack top: exp-164, H161. Do not generalize macOS results to Linux, or the reverse.
