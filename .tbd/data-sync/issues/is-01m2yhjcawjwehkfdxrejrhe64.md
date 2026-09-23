---
type: is
id: is-01m2yhjcawjwehkfdxrejrhe64
title: Python watch interval accepts nonfinite values and can panic in native conversion
kind: bug
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-20T04:35:39.995Z
updated_at: 2026-09-23T08:14:06.830Z
started_at: 2026-09-20T04:40:17.398Z
closed_at: 2026-09-23T08:14:06.830Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
WatchOptions only checks interval<=0; NaN/infinity pass, then native lib.rs Duration::from_secs_f64 can panic. Reject nonfinite and out-of-range values with ordinary argument errors, including direct native boundary calls, and add focused tests.
