---
type: is
id: is-01m2ebbacqb1qqy526s0bh06k1
title: "PR #49 review FLOOR-1: the same-thread-count invariant is false"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:06.262Z
updated_at: 2026-09-13T21:42:53.906Z
closed_at: 2026-09-13T21:42:53.905Z
close_reason: "Fixed: both probe tiers take --threads {workers}, so every instrument runs a fixed pool of N; the module docstring and rendered table describe that regime; --workers defaults to os.process_cpu_count / sched_getaffinity / cpu_count and is bounded by fdu's MAX_SCAN_THREADS (32), checked against scan.rs by a test."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, High. floor.py:32-33, 185, 195, 709 at 1fa2309. The docstring promises every tier is read against parfloor stat at the same thread count, but the probe tiers run 'summary --root R' and 'scan-index --root R' without --threads, so fdu runs its automatic pool (initial min(cores, 6), growing toward min(2x cores, 16)) while parfloor and arena_spike run {workers} = os.cpu_count(). Every x-floor is biased on any host where those differ, in either direction. Fix chosen: pass --threads {workers} to both probe tiers, describe the regime as a fixed pool of N, derive the default from os.process_cpu_count(), then sched_getaffinity, then os.cpu_count(), and bound --workers by fdu's 32-thread clamp so the invariant cannot silently break above it.
