---
type: is
id: is-01m2f3nhb280gynxp97cnkh329
title: The cache-only error blames a missing snapshot when there is no cache directory, and names a remedy that cannot work
kind: bug
status: in_progress
priority: 3
version: 3
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:44:06.881Z
updated_at: 2026-09-23T02:21:29.801Z
---
PR #51 delta review PR51-ONLY-1 (https://github.com/jlevy/fdu/pull/51#pullrequestreview-5193989647). crates/fdu-core/src/lib.rs:372-390, 423-455, 696-711 at 5d60aba: the 'no snapshot' branch of the CachePolicy::Only error, added by fdu-wjdk in 66d9f17, fires in two other cases. It fires when there is no cache directory at all, which the CLI can reach with XDG_CACHE_HOME and HOME unset. It also fires when a library open is aimed at another root's snapshot. In the first case the error names 'auto' as the remedy, but no auto run can ever make only succeed, because nothing can be saved. Fix: tell apart 'no cache location' and 'snapshot for a different root' from 'no snapshot yet', give each its own remedy, and test each branch.

## Notes

Implemented distinct no-location, missing snapshot, and wrong-root cache-only refusals on execution layer. Three-branch regression passes; integrated PR115 awaiting stack CI.
