---
type: is
id: is-01m2tvwkjpgkktrx36jmtje70x
title: "Consolidate leftover 0.1.0 PRs #84-#88 into a 3-PR stack"
kind: task
status: closed
priority: 1
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2s0tq4ppsygrs129nw1m86n
hold: null
hold_until: null
created_at: 2026-09-18T18:19:03.123Z
updated_at: 2026-09-18T18:28:48.269Z
started_at: 2026-09-18T18:19:18.098Z
closed_at: 2026-09-18T18:28:48.267Z
close_reason: "Leftover 0.1.0 PRs collapsed to three: #84 absorbed #88, #87 absorbed #85 (kept #87 CHANGELOG watch wording), #86 left as landing page. CI green on survivors. Nothing merged to main."
resolution: null
duplicate_of: null
---
Collapse leftover 0.1.0 docs/release PRs that are too small to review individually. Not a feature-phase stack.

Plan (theme boundaries, not a 5-way dump):
1. Merge #88 into #84 (channel setup + runbook). Close #88 as superseded.
2. Merge #85 into #87 (leftovers). Keep #87 CHANGELOG watch/metadata-only wording. Close #85 as superseded.
3. Leave #86 as its own landing-page PR.

Do not merge to main. No force-push. Merge commits only.

## Notes

Consolidation executed. Three leftover PRs remain, all targeting main. No merge to main.

Survivors:
- #84 4828d950 docs(release): first-time channel setup and 0.1.0 runbook (absorbed #88 by FF). 964+/194-
- #87 44ff6fdb leftovers (absorbed #85; kept #87 CHANGELOG watch wording). 500+/91-
- #86 9d9dd577 landing page left separate. 479+/680-

Closed:
- #88 MERGED into #84 branch (not main) https://github.com/jlevy/fdu/pull/88
- #85 CLOSED superseded by #87 https://github.com/jlevy/fdu/pull/85

#77 stays closed. Watching CI on #84 and #87.
