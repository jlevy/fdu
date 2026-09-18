---
type: is
id: is-01m2sgqny5h136xk4zsmdkhgv5
title: "PR #86 review R1: uv run after uv tool install cannot import fdu"
kind: bug
status: closed
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2sgak5bbbfm5y4pp08w8ztx
hold: null
hold_until: null
created_at: 2026-09-18T05:44:52.932Z
updated_at: 2026-09-18T05:50:20.897Z
started_at: 2026-09-18T05:45:13.548Z
closed_at: 2026-09-18T05:50:20.896Z
close_reason: "Fixed on PR #86 in d743e756"
resolution: null
duplicate_of: null
---
plan-2026-09-18-fdu-first-release-verification.md:115-116
Import from the wheel with uv run --with, not a bare uv run after uv tool install.
