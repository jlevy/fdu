---
type: is
id: is-01m368mmcw7jpg7kp4hqtka0q6
title: Stop parallel reconciliation churn when test unwinds
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T04:33:32.059Z
updated_at: 2026-09-23T04:37:39.556Z
---
PR115 Windows CI run 35817347581 stalls after parallel_equivalence begins the changing-tree reconciliation test. The test uses a scoped churn thread whose stop flag is set only after eight reconciles; if the body unwinds before that store, scope joins a thread that loops forever. Add test-only unwind-safe stop signalling while keeping eight reconciles, churn operations, and assertions unchanged. Diagnose the underlying CI stall separately; this guard only ensures panics surface rather than hanging the job. Verify focused test and native Windows CI; do not claim production behavior changed.

## Notes

2026-09-22 local PR115 checkpoint: scoped RAII guard sets the churn stop flag on unwind before thread::scope joins. Existing normal explicit stop/join and all eight reconciles remain intact; production unchanged. Focused test 1/1 and full parallel_equivalence 6/6 pass on macOS with shared target and CARGO_INCREMENTAL=0. A temporary environment-triggered panic immediately after churn starts produced a visible failing test in 0.87s instead of a hang; injection was restored and the focused test passed again. Windows CI run 35817347581 still stalled at the original test with no visible panic; root cause remains unproven. Await reviewed forward publication and native Windows CI.

Publication checkpoint: reviewed test-only guard committed as PR115 eb862ec6 and forwarded without conflicts into PR116 57f01a48 and PR117 adc39d24. Each upper first-parent diff is only the same 10-line test change; no production files changed. New exact-head CI PR115 35819044171, PR116 35819042913, PR117 35819047413 are pending. Original stalled Windows job 107041656026 retained for evidence; underlying stall cause and native validation remain open.
