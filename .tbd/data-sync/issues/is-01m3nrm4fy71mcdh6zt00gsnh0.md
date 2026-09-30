---
type: is
id: is-01m3nrm4fy71mcdh6zt00gsnh0
title: "Full revision of the performance evidence report after 0.2.1 (new PR stacked on #157)"
kind: task
status: closed
priority: 2
version: 5
delegate: claude-code@vm
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-29T05:01:29.469Z
updated_at: 2026-09-29T22:00:17.343Z
started_at: 2026-09-29T05:01:30.866Z
closed_at: 2026-09-29T22:00:17.343Z
close_reason: "PR #158 is complete and reviewed; review fixes pushed (b704ba47); ready to merge after #157."
resolution: null
duplicate_of: null
---
Follow-up to the 2026-09-29 refresh (merged in #155 as 672c2188): bring report-2026-08-20-fdu-performance-evidence.md fully current with 0.2.1 as released, the Linux round, and the 0.2.2 plan; resolve fdu-a53f (exp-167 verdict), fdu-xr7o (H140/H146 leftover caveats), fdu-vyco (minor perf-doc fixes), fdu-16nt (chips), the remaining part of fdu-72bn; regenerate ledger/report views. New branch stacked on #157.

## Notes

2026-09-29 (branch perf-report-revision, stacked on #157 at 8e32ca25):
- Plan: (1) record fixes in guides and experiment records for fdu-a53f, fdu-xr7o, fdu-vyco, fdu-lk8p; (2) full revision of report-2026-08-20-fdu-performance-evidence.md for 0.2.0 / 0.2.1 / the 0.2.2 plan and the standing peer gaps; (3) close or scope fdu-16nt and fdu-72bn; (4) regenerate ledger and page (PREPARED=2026-09-29) and run the perf and docs checks.
- Findings so far: 0.2.1 prepared from c1644575 (tag v0.2.1 exists on origin). exp-167 follows the exp-164/exp-165 precedent (macOS non-regression cell of a Linux-decided change). Paired-vs-marginal stat verified 435/2,146 = 20.3% from timeline.json. Chips existed only in the unmerged white-paper page (d45c9879, branch codex/performance-research-white-paper), never in report_html.py.

2026-09-29T06:05Z: PR #158 (claude/perf-report-revision, stacked on #157): c7bf2d3d, cb968b78, f211f12e (from the revision agent, which was stopped) plus 31241be6 (report brought current with 0.2.1 and the 0.2.2 plan). Closed fdu-vyco, fdu-a53f, fdu-xr7o, fdu-lk8p, fdu-16nt; fdu-72bn remains open. Checks: docs-format, perf schema/evidence/ledger/report all pass.
