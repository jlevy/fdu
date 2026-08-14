---
type: is
id: is-01kzzj1137r8kjyv0rwfc6ya70
title: "S6: per-directory extension tallies are retained everywhere and read almost nowhere"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T07:15:48.966Z
updated_at: 2026-08-14T07:15:48.966Z
---
Every directory carries by_ext, a map from interned extension id to tally, and merge_upward merges that MAP at every ancestor of every file. Most invocations read extension tallies for one directory - the root - or the handful a --view types renders, so this is a large memory and merge cost for a projection rarely read at depth. RSS at million scale is the clearest remaining defect and this is retained state scaling with directories times distinct extensions. Two variants to screen independently: a compact sorted (ExtId, ExtTally) slice instead of a BTreeMap, which most directories hold in a handful of entries; or retain tallies only at the root and compute a directory's on demand by subtree traversal when a query asks, which trades a rare query's cost for every scan's cost and needs the query surface audited for who reads rollup_of(..).by_ext at depth. Neither is in the registry. Predict million-entry RSS down at least 15 percent and cold indexed wall down at least 3. Index tier.
