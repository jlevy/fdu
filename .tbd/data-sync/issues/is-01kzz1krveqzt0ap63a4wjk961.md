---
type: is
id: is-01kzz1krveqzt0ap63a4wjk961
title: Normalize fdu product name casing
kind: task
status: closed
priority: 1
version: 8
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-14T02:28:57.317Z
updated_at: 2026-09-17T02:10:45.057Z
closed_at: 2026-09-17T02:10:45.056Z
close_reason: "Lowercase fdu naming landed in PRs #18 and #19"
resolution: null
duplicate_of: null
---
Use lowercase fdu consistently for the product and command name, by analogy with du. Preserve uppercase only inside conventional identifiers such as FDU_BUILD_VERSION or environment variables.

## Notes

Flagged stale at the 2026-08-23 handoff: left in_progress by a session that ended without closing it, last touched 8-13 days earlier. Status not changed because this session could not verify whether the work landed. Triage before trusting the in_progress marker -- either close it or restart it deliberately.

2026-09-13: status moved in_progress -> open during a PR-stack organization pass, because no session has touched this since the flag above and in_progress was asserting ownership nobody holds. Correction to an earlier version of this note written minutes before: it attributed the claim to the 2026-08-23 overnight loop, which was wrong -- the claim predates that run, which only flagged it. STILL UNVERIFIED: whether the work actually landed. If it did, close this rather than restarting it.
