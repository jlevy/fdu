---
type: is
id: is-01m0pqhtegckget9h3jw7p5x1b
title: "PR #38 review R7: hand-maintained experiment counts drifted in four documents"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:54.031Z
updated_at: 2026-08-23T07:34:38.793Z
closed_at: 2026-08-23T07:34:38.792Z
close_reason: "Fixed: campaign status 7->9 Linux in two places, framework-extraction 64->66, platform-tuning 51->57 of 66 plus a Linux row that no longer says 'not yet in the ledger'. Where the count was not the point (evidence report, floor report) the literal was removed rather than reset."
---
report-2026-08-20-fdu-performance-evidence.md says 64 artifacts and thirty-one kept; plan-2026-08-22-experiment-loop-framework-extraction.md says 28 of 64; report-2026-08-14-performance-campaign-status.md says 9 Linux in one place and 7 in two others; platform-tuning.md says all 51 ledger experiments.
