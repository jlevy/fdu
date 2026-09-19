---
type: is
id: is-01m2w5armgzqvtyt8yryt5hfet
title: "H113: cache-only completeness should not re-walk analysis_candidates"
kind: task
status: closed
priority: 1
version: 4
delegate: unknown@spud10
labels:
  - macos-agenda
  - campaign-2
  - performance
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-19T06:23:18.662Z
updated_at: 2026-09-19T06:42:08.559Z
started_at: 2026-09-19T06:32:23.444Z
closed_at: 2026-09-19T06:42:08.557Z
close_reason: "exp-110 rejected H113 on wall: median -7.59% but interval [-10.76%, +2.24%] includes zero. File-count shortcut reverted. Incomplete-sidecar fail-closed test kept. Next is H83 / fdu-jxhk."
resolution: null
duplicate_of: null
---
After a cache-only sidecar restore, open_for_report walks analysis_candidates again only to compare hits to len() (lib.rs ~598-602). exp-109 sampled that walk at 12.6% of content_open (~9% of wall). Completeness can use a count already known from restore and still refuse an incomplete sidecar.

Accept: content-cache-hit wall down at least 3% with the interval below zero on deciding-scale metabrowser; content digest identical; incomplete sidecar still refused.

Do this before fdu-jxhk. Do not retry parse-speed (H112).

## Notes

Pre-register exp-110 / H113 (before any engine change).

Metric: content-cache-hit wall_ns on nominated metabrowser-clone (same deciding subject as exp-108/109).
Direction: down.
Accept: median at least 3% faster and 95% paired interval entirely below zero; content digest identical to control; incomplete sidecar still refused (existing fail-closed tests plus a two-file snapshot / one-record sidecar case).
Control: current branch HEAD (includes H112 off-by-default restore phase timers). Claim-grade pair with FDU_COUNTERS unset.
Regime: attempt quiet; if the start gate fails, uncontrolled. No RAM disk. 12 interleaved pairs.
Change: fdu-core only — cache-only completeness uses Index::analysis_candidate_count (root regular-file total) instead of walking analysis_candidates just for len().
