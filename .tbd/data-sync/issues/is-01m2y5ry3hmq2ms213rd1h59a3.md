---
type: is
id: is-01m2y5ry3hmq2ms213rd1h59a3
title: "PR #91 review R1: retain fractional restore timing"
kind: bug
status: closed
priority: 2
version: 5
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2y58e5s7yzpstkcqmheymy5
hold: null
hold_until: null
created_at: 2026-09-20T01:09:31.888Z
updated_at: 2026-09-20T01:50:13.837Z
started_at: 2026-09-20T01:10:13.050Z
closed_at: 2026-09-20T01:50:13.836Z
close_reason: Senior review fixes are reviewed, committed, and pushed on PR91 870bdcfb / PR92 937f9445. All required CI checks pass on both exact heads. Final finding dispositions are posted. Parent fdu-30ns retains local full checks blocked by host StorageFull.
resolution: null
duplicate_of: null
---

## Notes

PR91 R1 review https://github.com/jlevy/fdu/pull/91#pullrequestreview-5258707801; content_cache.rs195/212 rounded each per-record duration. Implemented c2487f38; targeted tests pass, root reviewed, gate/push pending.
