---
type: is
id: is-01m2ymmc4c03yvnc10myb2jmk6
title: Native watch tests report success when the host delivers no events
kind: bug
status: open
priority: 2
version: 1
labels:
  - testing
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T05:29:11.049Z
updated_at: 2026-09-20T05:29:11.049Z
---
Confirmed during release correctness validation: unchanged PR91 a_created_file_arrives_as_an_upsert printed a skipped-precondition message after120seconds and libtest reported passed inside the macOS sandbox, while the identical binary/test passed with a real event in0.15seconds outside the sandbox. Similar native-watch precondition helpers return early when no events arrive. A green gate can therefore cover no native watch behavior. Require an explicit environment opt-out for hosts deliberately lacking event delivery; otherwise fail with an actionable precondition diagnostic. Keep CI strict and document the opt-out separately from successful assertion evidence. Coordinate with fdu-wgu8 permission-precondition cleanup; do not simply extend timeouts or weaken post-precondition assertions.
