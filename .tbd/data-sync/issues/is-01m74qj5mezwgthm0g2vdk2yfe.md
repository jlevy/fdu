---
type: is
id: is-01m74qj5mezwgthm0g2vdk2yfe
title: "PR #209 G3: hard links move every name's change time; live routes miss it and docs do not name it"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m74qj25f6rvk77ybzaz8rszg
created_at: 2026-11-11T07:18:38.477Z
updated_at: 2026-11-11T08:42:32.993Z
closed_at: 2026-11-11T08:42:32.993Z
close_reason: "Fixed in e1533c6d: hard links named in every change-time caveat, live-route limit documented and pinned by a watch test; design deferred to fdu-yl8u."
resolution: null
duplicate_of: null
---
Medium. fdu-design-principles.md:340-349, watch_session.rs:2414, docs/usage.md:292-297/:336, SKILL.md:181/:436, cli.rs:318-322, CHANGELOG.md:20-23. Fix: document hard links and the live-route limit, pin it with a test, track the design. Review G on PR #209: https://github.com/jlevy/fdu/pull/209#issuecomment-6789444811
