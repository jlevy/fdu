---
type: is
id: is-01m2h6pa84kf2qm78mpr6fy2vw
title: "PR #55 review PR55-ACCT-2: four imprecisions in the unique-allocated accounting rule"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h6nn916177k0wrc92zmkge
created_at: 2026-09-15T00:15:26.979Z
updated_at: 2026-09-15T00:32:14.405Z
closed_at: 2026-09-15T00:32:14.403Z
close_reason: "0a254bf: bytewise path order; in-scope and out-of-scope hard-link cases; renamed-link case; unique allocated recorded as not observed where (dev, inode) or link count is unavailable, ranking falls back to per-path under that name"
resolution: null
duplicate_of: null
---
PR #55, delta review 5204152578. Plan @4727de0 :165-166 'sorts first by bytes' names no measure; :458-459 hard-link case omits 'in-scope' and the out-of-scope variant; :451-455 has no case for renaming one link of a multi-link file; on Windows inode and dev are 0 (scan.rs:5096-5097 at dda7e6a) and there is no link count, so 'unique' silently equals per-path. Fix all four in the plan text.
