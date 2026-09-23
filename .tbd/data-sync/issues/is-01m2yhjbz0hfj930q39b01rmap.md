---
type: is
id: is-01m2yhjbz0hfj930q39b01rmap
title: Legacy watch Session misses changes made before watcher registration
kind: bug
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-20T04:35:39.615Z
updated_at: 2026-09-23T08:14:06.823Z
started_at: 2026-09-20T04:40:17.391Z
closed_at: 2026-09-23T08:14:06.823Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
At 937f9445 Session binds observation after accepting a supplied Index and returns without closing the scan-to-bind gap. A completed mutation between scan and Session construction may remain stale indefinitely. Bind capture, reconcile baseline, consume captured verified hints before initial report using shared engine lifecycle; deterministic pre-construction mutation regression.

## Notes

2026-09-20 root review of initial fix f7aff5cd: native watch tests must run outside the macOS sandbox; unchanged PR91 control delivered an event in0.15s outside and silently skipped after120s inside. Additional handoff acceptance: flush/drain must account for sticky overflow requiring another capture barrier, and accept_partial=false must refuse a newly partial initial reconciliation. Follow-up implementation assigned before closure.
