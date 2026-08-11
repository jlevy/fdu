---
type: is
id: is-01kzrt4tjjx3jj50w8cvb1wgfp
title: "Size-adaptive cache policy: rescan small trees, journal large ones"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53e2qffv7d9a2q7vg2yth
created_at: 2026-08-11T16:23:00.945Z
updated_at: 2026-08-11T16:23:57.205Z
---
The spike's numbers show the warm path must be chosen per tree, not fixed. Measured on a 60k tree: rescan 37 ms, load+full-sweep 102 ms, journal replay 10-200 ms fixed - so at project scale the fastest cache is no cache. At home-folder scale (millions of entries, far past kern.maxvnodes=263168 on this host) rescan is minutes and the journal's fixed cost is a rounding error. Also establishes a conclusion beyond FSEvents: for stat-tier queries the full sweep is DOMINATED by rescanning at every size (same syscalls plus a load), so it should run only for content-tier queries and change feeds. Implement CachePlan::choose() as a pure function over (entry count, recorded scan us/entry from the snapshot header, metadata-cache capacity, reducer tier); the snapshot carries its own cost model so the estimate self-calibrates per machine and storage. G11's replay budget becomes derived: never spend longer replaying than rescanning would have cost. Includes the capacity probe (kern.maxvnodes / dentry-state), which is the same signal the frontier research's H36 knee experiment needs.
