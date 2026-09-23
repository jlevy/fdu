---
type: is
id: is-01m2yhjbkazg1zds9x9janfd5m
title: Watch streams retain rows after files leave attribute selection
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-20T04:35:39.241Z
updated_at: 2026-09-23T08:14:06.816Z
started_at: 2026-09-20T04:40:17.382Z
closed_at: 2026-09-23T08:14:06.816Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
At 937f9445 Session::change_for treats Updated like Inserted and emits nothing if current attributes fail selection. An admitted 8-byte file shrinking below min_size=4 remains in a stream consumer although Session.report excludes it. Correct transitions and prove stream/report membership agreement with deterministic tests.
