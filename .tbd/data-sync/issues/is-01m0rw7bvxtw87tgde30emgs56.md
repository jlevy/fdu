---
type: is
id: is-01m0rw7bvxtw87tgde30emgs56
title: "Invalidation vocabulary: dirty query kinds, all-dirty, and reset as distinct signals"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:03.165Z
updated_at: 2026-08-24T22:53:52.395Z
---
MetaBrowser's changes() contract emits "bounded dirty paths or query kinds", "`all_dirty`
when individual dirtiness exceeds its bound", "`reset` when the requested cursor or
consumer queue has a gap", and "a state-only transition".

fdu today emits one of those four cleanly and the others not at all:

  reset            -> ChangeSet.truncated. Built, and the SSE example already models it.
  dirty paths      -> Watch.dirty_rollups, a bare Vec<PathBuf>. Bounded only incidentally.
  all_dirty        -> NOT DISTINGUISHABLE. When the watch escalates to a root
                      invalidation the dirty set is not labelled as such, so a consumer
                      cannot tell "the root's own rollup moved" from "throw everything
                      away". Those demand opposite amounts of work.
  dirty query kinds-> ABSENT. The contract's algebra is nine kinds (entry, directory,
                      filtered_tree, rollup, navigation, recent, catalog, metadata,
                      diagnostics); fdu can say which PATHS moved but not which
                      PROJECTIONS are stale, so a consumer holding a `recent` list must
                      re-derive whether its own view is affected.
  state-only       -> fdu-jxs0. Provenance changes bypass the clocked delta contract
                      entirely, so a trust transition produces no signal at all.

Land the labelled vocabulary as an enum on the batch rather than as extra Vec fields: an
unlabelled empty list already means two different things today, and adding a second
unlabelled list would compound that.

Depends on fdu-jxs0 for the state-only member.

## Notes

METABROWSER DECISION LANDED at 68eeaac (2026-08-24). Reset and provider observation-gap recovery are distinct. reset means the requested consumer cursor/session cannot resume coherently and requires checkpoint+reread. A primary watcher overflow instead marks freshness stale, emits a typed watcher-gap issue, reconciles the affected scope when possible, and publishes bounded dirty/all_dirty or state transitions on the provider clock; it is not itself reset. The current WatchBatch carrier sets reset for WatchOverflow/UnpairedRename/WatchSetupRace, so this bead must split those meanings and pin them with separate tests. Unrecoverable observer failure remains stale with a typed issue rather than masquerading as reset recovery failure.
