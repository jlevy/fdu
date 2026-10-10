---
type: is
id: is-01m4kwne0hp4grkq3d120wejq5
title: "report_roots: check root aliases once per index state, and measure a several-root second report"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
created_at: 2026-10-10T21:49:19.248Z
updated_at: 2026-10-10T21:49:19.248Z
---
Review E2 on PR #192 (jlevy/fdu#192). On Unix, over several roots, report_roots calls Index::entered_directory_with on each index, which walks the whole arena and hashes each directory's identity: every retained several-root report costs O(entries), against the plan's rule that a retained report costs what its rows cost (exp-212: about 0.13 ms). By estimate a 1M-entry index adds milliseconds per call. #192 documents the cost on report_roots. Remaining: (a) memoize the check per index state (keyed by each index's commit clock, e.g. on a reader holding the indexes), and (b) add a roots-second-report probe job so the cost is measured (design principles: every output surface is a benchmark job). Done when a second report over retained several-root indexes costs no arena scan, or the job shows the scan's cost is negligible on linux-balanced-1m.
