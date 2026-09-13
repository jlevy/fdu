---
type: is
id: is-01m01hakg0s70skr7r11qj9fbj
title: Make content self-check enumerate every tracked file type
kind: bug
status: open
priority: 2
version: 5
labels: []
dependencies: []
created_at: 2026-08-15T01:42:03.007Z
updated_at: 2026-09-13T16:50:56.497Z
---
The self-check asserts that specific tracked types exist but relies on the CLI's default top-10 report limit. Large report assets can push a valid type such as TOML below that display limit. Request all rows before asserting repository-wide type coverage.

## Notes

Flagged stale at the 2026-08-23 handoff: left in_progress by a session that ended without closing it, last touched 8-13 days earlier. Status not changed because this session could not verify whether the work landed. Triage before trusting the in_progress marker -- either close it or restart it deliberately.

2026-09-13: status moved in_progress -> open during a PR-stack organization pass, because no session has touched this since the flag above and in_progress was asserting ownership nobody holds. Correction to an earlier version of this note written minutes before: it attributed the claim to the 2026-08-23 overnight loop, which was wrong -- the claim predates that run, which only flagged it. STILL UNVERIFIED: whether the work actually landed. If it did, close this rather than restarting it.
