---
type: is
id: is-01kzy3eakncq1hs75wd35em3y8
title: Reconciliation attribution is populated only by the parallel wave path
kind: bug
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzy4tve6eej9e0jhxfqmqqmz
created_at: 2026-08-13T17:41:41.620Z
updated_at: 2026-08-13T18:28:24.782Z
closed_at: 2026-08-13T18:28:24.781Z
close_reason: "Fixed: incomplete wave-only reconciliation attribution was removed. Both serial and parallel reconciliation now leave the public attribution at zero/not measured until fdu-78wr adds complete comparable instrumentation."
---
ReconcileReport.scan.attribution is public and documented as where the walk's time went. After PR #8 the wave workers fill wall_ns, work_ns, and claims, while the serial reconciler leaves every counter at zero, so a caller cannot tell 'not measured' from 'no work' - the exact failure the perf_probe comment warns about. The parallel wall_ns also sums one worker lifetime per wave, so it is not an elapsed span and is not comparable with a cold scan's. Decide between instrumenting the serial sweep the same way and leaving reconciliation attribution unpopulated; the review only documented the current state.
