---
type: is
id: is-01m01hakg0s70skr7r11qj9fbj
title: Make content self-check enumerate every tracked file type
kind: bug
status: in_progress
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-08-15T01:42:03.007Z
updated_at: 2026-08-23T05:42:49.750Z
---
The self-check asserts that specific tracked types exist but relies on the CLI's default top-10 report limit. Large report assets can push a valid type such as TOML below that display limit. Request all rows before asserting repository-wide type coverage.

## Notes

Flagged stale at the 2026-08-23 handoff: left in_progress by a session that ended without closing it, last touched 8-13 days earlier. Status not changed because this session could not verify whether the work landed. Triage before trusting the in_progress marker -- either close it or restart it deliberately.
