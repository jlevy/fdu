---
type: is
id: is-01m360g63qcs4p5jngz84g5v4y
title: Flush capture publishes retained overflow before acknowledging the barrier
kind: bug
status: in_progress
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels: []
dependencies: []
created_at: 2026-09-23T02:11:17.750Z
updated_at: 2026-09-23T02:11:52.593Z
---
Serving CI5e9f6061 fails initial_handoff_drains_a_sticky_overflow_after_a_full_intent_queue on Linux. Worker tries sticky overflow at loop start then blocks receiving. Consumer can drain output while worker waits; Flush previously acknowledged without retrying sticky delivery, so handoff final empty poll races the next-loop publication. Deliver retained overflow within Flush before acknowledgement when capacity exists. Keep bounded two-barrier handoff and queue-capacity bound. Existing integration regression must pass repeatedly; no sleeps or weakened assertions.
