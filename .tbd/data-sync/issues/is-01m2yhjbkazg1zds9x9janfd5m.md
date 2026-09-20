---
type: is
id: is-01m2yhjbkazg1zds9x9janfd5m
title: Watch streams retain rows after files leave attribute selection
kind: bug
status: in_progress
priority: 1
version: 2
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-20T04:35:39.241Z
updated_at: 2026-09-20T04:40:17.382Z
started_at: 2026-09-20T04:40:17.382Z
---
At 937f9445 Session::change_for treats Updated like Inserted and emits nothing if current attributes fail selection. An admitted 8-byte file shrinking below min_size=4 remains in a stream consumer although Session.report excludes it. Correct transitions and prove stream/report membership agreement with deterministic tests.
