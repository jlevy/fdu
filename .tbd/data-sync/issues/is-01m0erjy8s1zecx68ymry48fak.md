---
type: is
id: is-01m0erjy8s1zecx68ymry48fak
title: End-to-end verification round on macOS
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m0erhq35tpxzjecxn3p9jzx2
created_at: 2026-08-20T04:59:03.817Z
updated_at: 2026-08-20T05:51:56.476Z
closed_at: 2026-08-20T05:51:56.474Z
close_reason: |-
  End-to-end round complete on macOS (Darwin 25.5.0).

  Gates: make check passed on unmodified main as a baseline and again on the change; make cross-lint and make docs-format-check pass; 110 golden cases; all 17 CI checks green on macos-latest, ubuntu-latest, and windows-latest.

  Exercised by hand: all 8 views; all 4 formats; all 5 cache policies plus --cache-status and --cache-clear; all 5 analysis profiles; the selection axes (include, exclude, min-size, modified-since, modified-before, kind, sort, reverse, depth, limit, scan-depth, one-filesystem); watch mode in both text and JSONL; the error, partial-result, and exit-code paths; and edge cases (empty tree, unicode and emoji filenames, symlinks including a broken one, 60-level nesting).

  Accounting verified against the system tools on a real 10,925-file tree: files 10925 = find, dirs 2075 = find, apparent bytes 342,183,456 = sum of stat, allocated 371,679,232 B = du -sk 362,968 KiB. Exact on all four. Five repeated cold runs byte-identical; cache-only agreed with the cold scan; scan worker counts 1, 2, and 8 agreed.

  Verified-correct behavior that looks surprising: a repeated one-shot CLI report scans cold rather than reading its own snapshot. Deliberate, and pinned by a_repeated_one_shot_report_scans_cold_while_open_still_revalidates in execution.rs — a one-shot cannot amortise a snapshot load, while a library caller holding the index through open() still takes the warm path.

  Three genuine inconsistencies found, each filed separately: fdu-muzk (p1, dirs count ignores selection filters), fdu-2toe (p2, extensions view silently omits extension-less files), fdu-hs7q (p2, watch text repaints have no boundary).
---
Run the full handoff gate (make check), cross-lint, and a hands-on end-to-end exercise of the CLI surface on real trees: every view, every format, cache policies, analysis profiles, filters, watch mode, and the Python binding. Record any inconsistency found as its own bead.
