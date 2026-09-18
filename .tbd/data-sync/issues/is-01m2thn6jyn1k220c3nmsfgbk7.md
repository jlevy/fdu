---
type: is
id: is-01m2thn6jyn1k220c3nmsfgbk7
title: README summary-tier figure invites a cross-table comparison the source report forbids
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-09-18T15:20:14.683Z
updated_at: 2026-09-18T17:57:39.224Z
closed_at: 2026-09-18T17:57:39.223Z
close_reason: "README Speed section now presents the aggregate-only tier as a memory result (15.0 MiB against the 285.4 MiB the tree view held in the same fdu-only round-robin) and states plainly that the tier buys memory, not time. The 4.876 s figure is dropped, so no cross-table speed reading against the matrix 5.206 s is possible. Fixed in 9d9dd577 on PR #86; make docs-format-check clean and all CI checks pass."
resolution: null
duplicate_of: null
---
README.md Speed section places 'fdu --no-gitignore --view summary keeps the aggregate-only tier: the same tallies in 4.876 s at 15.0 MiB' directly after the seven-tool matrix whose fdu row is 5.206 s. Those two numbers are from different tables in report-2026-09-16-fdu-live-tool-comparison.md, and that report says the three round-robin medians are lower only because the matrix interleaves seven tools and evicts metadata between adjacent fdu runs, ending 'Compare rows within a table, never across them.' The report also states plainly that the summary tier no longer costs less than the tree because it now does the same work, and that the tier's value is memory rather than time. As written the README invites the reader to read 4.876 s against 5.206 s as a speed win. Both figures are individually accurate. Predates the restored-numbers commit (749b7977) and rides on PR #86. Fix: say the 4.876 s figure comes from a separate round-robin and that the tier buys memory, not time.
