---
type: is
id: is-01m4d4r3ea9eezr4664nfm1nn4
title: "Evidence report: fix the macOS contradiction, add the index to the headline, and refresh stale pointers (review 2026-10-07)"
kind: task
status: open
priority: 1
version: 1
labels:
  - performance
  - docs
  - review
dependencies: []
created_at: 2026-10-08T06:55:54.313Z
updated_at: 2026-10-08T06:55:54.313Z
---
Review of docs/project/reports/report-2026-08-20-fdu-performance-evidence.md at main 99595252 (2026-10-07). Every count and every figure checked (~180) matches its artifact; perf-report-check, perf-evidence-check and perf-ledger-check exit 0; the macOS index (1.7084 [1.6817, 1.7355], partial 4.7320) recomputes independently. The problems are in the text:

- B1 Contradiction: :77-80, :121-122, :1145-1149, :1433-1434 (and runbook :1050-1051) say no macOS cell measured the 2026-09-29 round, the pdu track or 0.3.0; the report's own index section (:1281-1285) and the committed history cells time v0.2.1 -> exp178 (H171) -> exp180 -> ebc06c78 -> exp201 -> v0.3.0 on macOS. Say "no macOS experiment record or peer comparison", and state the exploratory milestone result (0.2.1 vs 0.3.0: default tree +15.5% [+11.6, +22.3], cold cache +13.2%).
- S1 Headline sections never mention the index; the 1.71x/4.73x appear only at :1289-1292.
- S2 Regime labels missing at :66-69 (mixes quiet 588 ms with uncontrolled exp-141), campaign-1..H86 round headers, Loops table.
- S3 Chained before/after figures without caveat: :74-75, :257 (1,218 -> 778 ms), :147-148 (590 -> 211 ms).
- S4 The index interval (+/-1.6%) excludes session-to-session variation; say so beside "10% within noise".
- S5 Context for "1.71x from 0.2.0": 0.2.0 sits on the .gitignore plateau (cold cache 163 -> 430 -> 433 -> 163 ms); fdu-inph.
- S6 Probe-oracle caveat (spec :173-180, 12.5% of weight) and fdu-92bg (0.3.0 macOS warm revalidate 478 ms vs cold-open-save 303 ms) absent from Qualifications/Open Work.
- S7 Quotability rule disagrees: report and spec say quiet + 20 rounds; perf_index.py evidence_regime also requires a non-exploratory stage and history.py defaults --stage exploratory. Pick one; add stage to fdu-bkj2.
- S8 Report does not name the rounds shortfall (10 of 13 cells at 12 rounds; 9 would have fit 20 in under an hour).
- S9 Stale Open Work: fdu-p6vc closed; index follow-ups (Linux Phase 2, fdu-bkj2, fdu-92bg, fdu-inph) missing; Current Pickup (2026-09-30) predates the index.
- S10 :199-201 "generated-tree peer comparisons not re-measured on 0.2.1" is contradicted by exp-202 and by :1245-1247.
- S11 index.html / timeline.json say Prepared 2026-09-30; republish with PREPARED=2026-10-06 or later.
- S12 Standing Results "Latest" column mixes engines (pre-0.2.0 macOS, pre-0.2.1 Linux) without an engine/date column.
- Nits: :1219 cites fdu-xde5 for bare metal (should be fdu-lf3v/fdu-tk1b); :56-58 probe figures labelled "default fdu PATH" (CLI cells: -17% on dense and generated, not -14%); N5 scale/cold cells lack answer_check (have semantic_mismatches 0); spec Status still "In review" after #176 merged; spec 20,000x vs report 7,700x compounding bases differ.
