---
type: is
id: is-01m0py28fhtj20wwfc05s9e148
title: "Overnight research loop: the macOS agenda for campaign 2"
kind: epic
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
child_order_hints:
  - is-01m0py28x68jgcegwj41t3twwj
  - is-01m0py298y6z5d6rd6sxdd429d
  - is-01m0py29ngg76h33a3y0v9ef5r
created_at: 2026-08-23T09:08:44.143Z
updated_at: 2026-08-23T10:46:52.501Z
---
The campaign-2 plan's arithmetic is Linux-warm and its floor instrument (parfloor.c) is Linux-only, so on this Mac -- the host the user's bar is set on -- the phases need a macOS ordering of their own. This epic holds that ordering: what an unattended agent runs first, what it may not start unsupervised, and the instruments the ordering depends on. Runbook: docs/project/guides/performance-loop-runbook.md. Strategy review: docs/project/reports/report-2026-08-23-research-loop-strategy-review.md.

## Notes

Night 1 (2026-08-23, PR #46): Tier 1 items 1-3 and 5 landed -- exp-066 baseline, exp-067 (-10.6% default-tree), exp-068 (TTFB -7.5%/-12.5%), exp-069 (content-cache-hit -31%, content-query -67%); item 4 (PGO) skipped: no llvm-tools, host not quiet. Host: ANECompilerService pid 37146 at ~99% for >1 day; every artifact uncontrolled. Next: fdu-9hdc (macOS floor), then fdu-4xtm/fdu-5yjk/fdu-0pzh; the Path::hash increment on the content roll-up map is a cheap Tier 1 follow-on to exp-069 (bead fdu-78q6 notes). Do not merge #46 unattended.
