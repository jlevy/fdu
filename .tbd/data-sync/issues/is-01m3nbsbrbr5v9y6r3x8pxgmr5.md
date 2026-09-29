---
type: is
id: is-01m3nbsbrbr5v9y6r3x8pxgmr5
title: "H172: exact transient tree tier (directories plus the K largest files) for one-shot tree reports"
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T01:17:09.258Z
updated_at: 2026-09-29T10:40:08.902Z
started_at: 2026-09-29T08:22:53.080Z
closed_at: 2026-09-29T10:40:08.902Z
close_reason: "Accepted and merged (0228ea42): exp-180 linux-v6.12 default-tree -13.48%, exp-181 node-modules-dense -10.30%, balanced screen -3.20% wall and peak RSS -79%; placebos at zero; answers identical (171 + 111 comparisons)."
resolution: null
duplicate_of: null
---
RetainedState::Tree (execution.rs:25, chosen at :439-447 beside summary_is_sufficient :414) when: route OneShot, no index policy, no analysis, views == [Tree], unfiltered, population Include, min-share > 0, K = ceil(100/pct) <= 65,536. push_directory allocates entries for directories only. Each file is classified, folded into its parent's all/unignored roll-ups, and offered to a global bounded min-heap of the K largest by the request's size metric. Unretained files add to the parent's folded tally (entries, files, bytes, allocated, ignored), and their names stay in the listing so the walker frees them (H159). expand adds folded to the Share omission (query_report.rs:3044) via Index::folded_children(id). Exactness: ShareThreshold::admits (query_selection.rs:206-218) admits at most floor(100/pct) files. Licensed by the engine architecture's derived-report plan. Proxy: balanced consumer 523M -> 195M instructions, RSS 66 -> 20 MB at 200k; the wall effect at 1M is predicted, not confirmed. ACCEPT: default-tree on balanced-1M (deciding): wall -3% with interval below zero, and peak RSS -50%; linux-v6.12 default-tree non-inferior; placebos aggregate-summary --no-controls and cold-scan-index include zero; goldens and parity unchanged. TESTS: transient_tree_equals_the_indexed_tree_under_every_bound_case over workers 1/2/4/8, both orders, min-share {1%, 0.5%, 10%, 0.01%}, size metric, sort keys, limit/breadth/depth, ties at K, zero-size trees, ignored files at the boundary, refused controls; parallel_equivalence; goldens; parity; path-independence. OPEN: carrier (pruned Index with folded tally, ~400-500 lines, vs TreeNodes outside the index); K ceiling; whether --sort name/mtime/count keep the tier.
