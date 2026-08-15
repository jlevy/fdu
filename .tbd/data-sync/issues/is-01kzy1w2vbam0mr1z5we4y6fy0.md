---
type: is
id: is-01kzy1w2vbam0mr1z5we4y6fy0
title: "H70: Tune a shared macOS directory-opener pool"
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
  - research
dependencies:
  - type: blocks
    target: is-01m01cm1sb8xyw9ag3pabb5s3h
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-13T17:14:15.274Z
updated_at: 2026-08-15T00:52:00.866Z
---
The H69 shared two-opener variant kept its openers in __open for about 77% of sampled tops while all six scanners still waited for descriptor responses for about 37% of their tops. Its exact five-pair rich-summary screen improved paired wall 3.98% with 95% CI [-9.87%, -0.70%], zero oracle mismatches, and no tree drift. Before production confirmation, screen exactly two, three, and four shared opener threads with six fixed scan/parser workers on the immutable 901,963-entry APFS tree. Pre-registered primary metric is paired wall; reject any count whose context-switch/CPU cost is disproportionate or whose wall interval crosses zero. Confirm the selected count with at least 12 adjacent pairs and on the independent large topology. The scanner/opener total must remain one explicit bounded concurrency budget; strict parsing, fallback, paths, scope, errors, and partial-result semantics are unchanged.

## Notes

Experimental only; no production code is retained in PR #8. A shared four-opener plus six-scanner prototype matched dumac's approximate 3-second wall on the 901,963-entry APFS tree while remaining far below dumac's memory/CPU, but the interactive host produced extreme paired outliers and no claim-grade confirmation. Resume with 12 quiet adjacent control pairs, direct dumac calibration, and an independent large topology; reject if context-switch or CPU cost is disproportionate.
