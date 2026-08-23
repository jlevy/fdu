---
type: is
id: is-01kzzbr1d091cjnjtvax16ssym
title: "Classification's residue: the double classify in apply_analysis's staleness guard"
kind: task
status: open
priority: 3
version: 9
labels:
  - campaign-2
dependencies: []
created_at: 2026-08-14T05:26:02.912Z
updated_at: 2026-08-23T09:09:38.839Z
---
Re-scoped per the campaign-2 plan: largely closed by exp-064. The bead's ~34% figure came from a flat callgrind profile; the caller tree put classification at 11.11% inclusive, and H95's indexed tiers took -41.4% absolute off classify_path_with_prefix. What remains is the double classification in apply_analysis's staleness guard, a public-contract change (AnalysisCandidate is constructible by callers) for a corner of 11%. Not on the overnight agenda; close if Phase C's structural item (fdu-jxhk) makes it moot.

## Notes

Re-scoped by exp-064 and exp-065 (2026-08-23). The ~34% figure that made this P0 came from a flat callgrind profile; the caller tree puts classification at 11.11% inclusive, and H95's indexed tiers have since taken -41.4% absolute off classify_path_with_prefix. What remains is the double classification in apply_analysis's staleness guard -- a public-contract change (AnalysisCandidate is caller-constructible) for a corner of 11%. Dropped P0 -> P2; re-scope or close.
