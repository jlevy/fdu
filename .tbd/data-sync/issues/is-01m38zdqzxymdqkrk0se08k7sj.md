---
type: is
id: is-01m38zdqzxymdqkrk0se08k7sj
title: "PR #119 review A-3: Delay test bullet promises an injected clock and a first frame at 500 ms"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m38zd02aamagph2hxgq8548d
hold: null
hold_until: null
created_at: 2026-09-24T05:50:12.476Z
updated_at: 2026-09-24T05:56:04.109Z
started_at: 2026-09-24T05:50:15.988Z
closed_at: 2026-09-24T05:56:04.109Z
close_reason: "Fixed in bb543676 on claude/progress-indicator-plan (A-4: note appended to fdu-vngp)"
resolution: null
duplicate_of: null
---
Plan line 319. The timed receive makes the timings the injectable seam; #120 injects timings and asserts nothing about wall-clock time. Reword the bullet. PR #119 review.
