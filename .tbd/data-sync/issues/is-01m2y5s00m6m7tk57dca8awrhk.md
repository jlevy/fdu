---
type: is
id: is-01m2y5s00m6m7tk57dca8awrhk
title: Isolate installed-command tests from host interactive shell profiles
kind: bug
status: closed
priority: 2
version: 4
labels: []
dependencies: []
parent_id: is-01m2y58e5s7yzpstkcqmheymy5
created_at: 2026-09-20T01:09:33.843Z
updated_at: 2026-09-20T01:50:13.885Z
closed_at: 2026-09-20T01:50:13.885Z
close_reason: Senior review fixes are reviewed, committed, and pushed on PR91 870bdcfb / PR92 937f9445. All required CI checks pass on both exact heads. Final finding dispositions are posted. Parent fdu-30ns retains local full checks blocked by host StorageFull.
resolution: null
duplicate_of: null
---

## Notes

Deferred from PR #92 review address: pre-existing installed-command test isolation from host bash profiles. Not this PR's engine; handle in a test-harness change, not on perf/campaign-next.
