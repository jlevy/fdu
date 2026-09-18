---
type: is
id: is-01m2thn6jyn1k220c3nmsfgbk7
title: README summary-tier figure invites a cross-table comparison the source report forbids
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-18T15:20:14.683Z
updated_at: 2026-09-18T15:20:14.683Z
---
README.md Speed section places 'fdu --no-gitignore --view summary keeps the aggregate-only tier: the same tallies in 4.876 s at 15.0 MiB' directly after the seven-tool matrix whose fdu row is 5.206 s. Those two numbers are from different tables in report-2026-09-16-fdu-live-tool-comparison.md, and that report says the three round-robin medians are lower only because the matrix interleaves seven tools and evicts metadata between adjacent fdu runs, ending 'Compare rows within a table, never across them.' The report also states plainly that the summary tier no longer costs less than the tree because it now does the same work, and that the tier's value is memory rather than time. As written the README invites the reader to read 4.876 s against 5.206 s as a speed win. Both figures are individually accurate. Predates the restored-numbers commit (749b7977) and rides on PR #86. Fix: say the 4.876 s figure comes from a separate round-robin and that the tier buys memory, not time.
