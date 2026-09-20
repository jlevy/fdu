---
type: is
id: is-01m2yhjbz0hfj930q39b01rmap
title: Legacy watch Session misses changes made before watcher registration
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T04:35:39.615Z
updated_at: 2026-09-20T04:35:39.615Z
---
At 937f9445 Session binds observation after accepting a supplied Index and returns without closing the scan-to-bind gap. A completed mutation between scan and Session construction may remain stale indefinitely. Bind capture, reconcile baseline, consume captured verified hints before initial report using shared engine lifecycle; deterministic pre-construction mutation regression.
