---
type: is
id: is-01m368mmcw7jpg7kp4hqtka0q6
title: Stop parallel reconciliation churn when test unwinds
kind: bug
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T04:33:32.059Z
updated_at: 2026-09-23T08:14:06.980Z
closed_at: 2026-09-23T08:14:06.980Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
PR115 Windows CI run 35817347581 stalls after parallel_equivalence begins the changing-tree reconciliation test. The test uses a scoped churn thread whose stop flag is set only after eight reconciles; if the body unwinds before that store, scope joins a thread that loops forever. Add test-only unwind-safe stop signalling while keeping eight reconciles, churn operations, and assertions unchanged. Diagnose the underlying CI stall separately; this guard only ensures panics surface rather than hanging the job. Verify focused test and native Windows CI; do not claim production behavior changed.

## Notes

2026-09-23: the guard worked: the next Windows occurrence surfaced as a failure with the real error, whose root cause and fix are tracked in fdu-39m3 (95ad163b on #98).
