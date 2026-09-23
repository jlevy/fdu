---
type: is
id: is-01m2ymmc4c03yvnc10myb2jmk6
title: Native watch tests report success when the host delivers no events
kind: bug
status: in_progress
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex@spud10
labels:
  - testing
dependencies:
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-20T05:29:11.049Z
updated_at: 2026-09-23T02:28:53.545Z
started_at: 2026-09-20T05:35:29.834Z
---
Confirmed during release correctness validation: unchanged PR91 a_created_file_arrives_as_an_upsert printed a skipped-precondition message after120seconds and libtest reported passed inside the macOS sandbox, while the identical binary/test passed with a real event in0.15seconds outside the sandbox. Similar native-watch precondition helpers return early when no events arrive. A green gate can therefore cover no native watch behavior. Require an explicit environment opt-out for hosts deliberately lacking event delivery; otherwise fail with an actionable precondition diagnostic. Keep CI strict and document the opt-out separately from successful assertion evidence. Coordinate with fdu-wgu8 permission-precondition cleanup; do not simply extend timeouts or weaken post-precondition assertions.

## Notes

Recovered serving layer PR114 requires native event delivery by default; FDU_TEST_ALLOW_NO_NATIVE_WATCH=1 is an explicit opt-out only for hosts deliberately without event delivery, never for lost events after a successful precondition. Reviewed focused native-watch tests passed without opt-out on this host. Final composed gate and cross-platform CI remain pending.
