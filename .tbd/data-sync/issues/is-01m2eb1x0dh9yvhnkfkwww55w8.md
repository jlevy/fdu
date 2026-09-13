---
type: is
id: is-01m2eb1x0dh9yvhnkfkwww55w8
title: "PR #54 review H86-1: exp-102 id collides with #52's committed exp-102"
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:33:57.644Z
updated_at: 2026-09-13T22:34:31.847Z
closed_at: 2026-09-13T22:07:22.144Z
close_reason: "Fixed: #52 brought in by a merge commit (46721fd); 86d2a6a renumbers the artifact to exp-103 (id, filename, research-note and plan links, fdu-xde5 notes) and regenerates the ledger, timeline.json and index.html, keeping prepared 2026-09-07. perf-ledger-check failed with the identifier collision before and passes after; CI 19/19 at 2d36eab."
resolution: null
duplicate_of: null
---
High, merge-blocking. PR #54's docs/project/experiments/exp-102-h86-linux-evidence-stage-relative-gates-pass-floor-gates-fai.md:9 declares id exp-102, which #52's exp-102-point-lookup-for-public-mutation-preflight.md already owns (with evidence/exp-102/). summary.check_identifiers (explorations/benchmarks/realtree/summary.py:82-123) treats a duplicate id as fatal, and timeline.py loads through it, so make perf-ledger-check and perf-report-check fail once the branches meet. Fix: renumber this artifact to the next free id (exp-103): id field, filename, research note link, streaming-parity plan link, bead fdu-xde5 note; regenerate ledger, timeline.json, and index.html on the merged tree keeping the base's prepared date. #52 review PERF-2 agrees #54 renumbers.

## Notes

PR #52 review 5192264318 finding PERF-2 (Medium) reports the same exp-102 collision from #52's side and says #54 renumbers. Tracked here rather than as a duplicate child of fdu-0xmy (Address review: PR #52). Disposition on #52: no change needed there; #54 renumbered its artifact to exp-103 in 86d2a6a.
