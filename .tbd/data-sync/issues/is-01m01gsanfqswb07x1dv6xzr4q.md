---
type: is
id: is-01m01gsanfqswb07x1dv6xzr4q
title: Replace oversized hypothesis chips in performance report
kind: bug
status: closed
priority: 2
version: 7
labels: []
dependencies: []
created_at: 2026-08-15T01:32:36.910Z
updated_at: 2026-09-29T06:05:32.331Z
closed_at: 2026-09-29T06:05:32.331Z
close_reason: "Not reproducible: no commit in the repository's history adds chip styling to the performance report renderer (git log --all -G 'hypothesis.?chip|class=.chip|\\.chip' matches only tbd sync commits carrying this bead's text); hypotheses render as plain text. Recorded in PR #158."
resolution: null
duplicate_of: null
---

## Notes

Flagged stale at the 2026-08-23 handoff: left in_progress by a session that ended without closing it, last touched 8-13 days earlier. Status not changed because this session could not verify whether the work landed. Triage before trusting the in_progress marker -- either close it or restart it deliberately.

2026-09-13: status moved in_progress -> open during a PR-stack organization pass, because no session has touched this since the flag above and in_progress was asserting ownership nobody holds. Correction to an earlier version of this note written minutes before: it attributed the claim to the 2026-08-23 overnight loop, which was wrong -- the claim predates that run, which only flagged it. STILL UNVERIFIED: whether the work actually landed. If it did, close this rather than restarting it.

2026-09-29: evidence-report refresh found no hypothesis chips anywhere in the renderer's history; hypotheses render as plain text. Candidate to close as not reproducible after a visual check.
