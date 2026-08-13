---
type: is
id: is-01kzy3eakncq1hs75wd35em3y8
title: Reconciliation attribution is populated only by the parallel wave path
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-13T17:41:41.620Z
updated_at: 2026-08-13T17:41:41.620Z
---
ReconcileReport.scan.attribution is public and documented as where the walk's time went. After PR #8 the wave workers fill wall_ns, work_ns, and claims, while the serial reconciler leaves every counter at zero, so a caller cannot tell 'not measured' from 'no work' - the exact failure the perf_probe comment warns about. The parallel wall_ns also sums one worker lifetime per wave, so it is not an elapsed span and is not comparable with a cold scan's. Decide between instrumenting the serial sweep the same way and leaving reconciliation attribution unpopulated; the review only documented the current state.
