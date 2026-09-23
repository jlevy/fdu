---
type: is
id: is-01m360g63qcs4p5jngz84g5v4y
title: Flush capture publishes retained overflow before acknowledging the barrier
kind: bug
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels: []
dependencies: []
created_at: 2026-09-23T02:11:17.750Z
updated_at: 2026-09-23T08:14:06.803Z
closed_at: 2026-09-23T08:14:06.803Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Serving CI5e9f6061 fails initial_handoff_drains_a_sticky_overflow_after_a_full_intent_queue on Linux. Worker tries sticky overflow at loop start then blocks receiving. Consumer can drain output while worker waits; Flush previously acknowledged without retrying sticky delivery, so handoff final empty poll races the next-loop publication. Deliver retained overflow within Flush before acknowledgement when capacity exists. Keep bounded two-barrier handoff and queue-capacity bound. Existing integration regression must pass repeatedly; no sleeps or weakened assertions.

## Notes

Fixed in isolated codex/alpha-serving-handoff-fix from fa073b0b: Flush retries sticky overflow delivery before acknowledgement, retaining marker if queue remains full. Existing CI-red handoff regression now passes; 100 consecutive direct binary runs passed. Handoff filter10/newer-state11 and core all-build-features/all-targets clippy pass after source/docs invalidation. Full gate and independent review pending, do not close yet.

Review update: da435e22 passed independent parent Astra review with no findings. Serving integration belongs to PR114, then forward merge into execution. Full final candidate gate remains pending.

Lower113 CI at94d1f31f reproduced initial_handoff_drains_a_sticky_overflow_after_a_full_intent_queue missing b.txt. Prepared exact ten-line Flush hunk from reviewed da435e22: retry sticky overflow delivery before acknowledging the barrier, so the consumer cannot complete its final empty poll before loss evidence arrives. Isolated backport watch.rs is byte-identical to lower114 da76ef03; no index prerequisite, assertion changes, or sleeps. Parent independently cleared exact backport; lower focused Rust validation awaits shared compiler slot or CI. Upper runtime remains unchanged when forwarded. Acceptance remains pending final stack gates.
