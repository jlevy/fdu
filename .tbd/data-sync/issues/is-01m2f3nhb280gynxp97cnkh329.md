---
type: is
id: is-01m2f3nhb280gynxp97cnkh329
title: The cache-only error blames a missing snapshot when there is no cache directory, and names a remedy that cannot work
kind: bug
status: closed
priority: 3
version: 4
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:44:06.881Z
updated_at: 2026-09-23T08:14:06.857Z
closed_at: 2026-09-23T08:14:06.857Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
PR #51 delta review PR51-ONLY-1 (https://github.com/jlevy/fdu/pull/51#pullrequestreview-5193989647). crates/fdu-core/src/lib.rs:372-390, 423-455, 696-711 at 5d60aba: the 'no snapshot' branch of the CachePolicy::Only error, added by fdu-wjdk in 66d9f17, fires in two other cases. It fires when there is no cache directory at all, which the CLI can reach with XDG_CACHE_HOME and HOME unset. It also fires when a library open is aimed at another root's snapshot. In the first case the error names 'auto' as the remedy, but no auto run can ever make only succeed, because nothing can be saved. Fix: tell apart 'no cache location' and 'snapshot for a different root' from 'no snapshot yet', give each its own remedy, and test each branch.

## Notes

Implemented distinct no-location, missing snapshot, and wrong-root cache-only refusals on execution layer. Three-branch regression passes; integrated PR115 awaiting stack CI.
