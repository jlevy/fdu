---
type: is
id: is-01m49mn290gts7b3ekrdaw91bw
title: "Time dut against fdu 0.3.0 on Linux: add a harness adapter"
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T22:16:54.291Z
updated_at: 2026-10-06T22:16:54.291Z
---
The README claims 'Fastest du replacement'. dut (codeberg.org/201984/dut, Linux-only) was read at source level but never timed in the harness (no adapter: report-2026-09-27-fdu-linux-tool-comparison.md:169). In the one unranked screen (research-2026-09-29-linux-peers-matchers-and-hot-path.md ~:515, 2026-09-29, 0.2.1-era engine, host 35-76% busy) dut took 95-102 ms on linux-v6.12 against fdu --no-gitignore 138-141 ms and fdu default 272-295 ms. Design principles name passing 'the benchmark gate against dut and gdu' as Goal 1. Add a dut adapter to the harness, time it interleaved against fdu 0.3.0 (default and --no-gitignore --view summary, the closest like-for-like) on linux-v6.12, node-modules-dense and the generated 1M tree in a quiet Linux session, and record the result. If dut leads, revisit the README tagline and open hypotheses for the gap (its lock-free atomic roll-up, S:305, was never adopted).
