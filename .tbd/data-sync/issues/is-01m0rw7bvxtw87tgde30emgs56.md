---
type: is
id: is-01m0rw7bvxtw87tgde30emgs56
title: "Invalidation vocabulary: dirty query kinds, all-dirty, and reset as distinct signals"
kind: feature
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:03.165Z
updated_at: 2026-08-25T01:59:54.748Z
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

EXACT CODE STATE at 558461a (2026-08-24). The newly landed carrier sets reset for WatchOverflow, UnpairedRename, and WatchSetupRace in Session.next_batch. This is the concrete site to change under the settled MetaBrowser rule: provider observation loss drives stale state, typed issue, reconciliation, and dirty/all_dirty; reset is only for an unresumable consumer cursor/session. Add separate overflow-recovery and stale-consumer-cursor tests so the two signals cannot collapse again.

EXACT-HEAD REVIEW at PR #47 278457a (2026-08-25). The new stepped-over-commit reset is conceptually the consumer-reset case and is aligned, but the provider-gap collapse remains unchanged at watch_session.rs:271-300: WatchOverflow, UnpairedRename, and WatchSetupRace still set reset rather than stale freshness plus a typed issue and reconciliation. A reset batch also retains dirty_rollups/all_dirty, whereas MetaBrowser's ChangeBatch requires reset to replace dirtiness. Fix and test the two paths separately: an omitted producer/expired consumer position yields reset with no dirty suffix; provider observation loss yields stale state plus typed watcher-gap issue and bounded dirty/all_dirty after reconciliation, never reset merely because the provider reconciled.

EXACT-HEAD UPDATE at PR #47 fad3d2f (2026-08-25). Recovery::of at watch_session.rs:198-201 still maps provider observation escalation to reset, contrary to MetaBrowser's provider-gap stale-state/typed-issue/reconcile path. It also maps a truncated consumer journal to reset + all_dirty; Session copies both into Batch at 320-331. MetaBrowser ChangeBatch explicitly requires reset to replace all dirtiness (inventory_engine/contract.py:776-779). Pin separate tests: an expired consumer position yields reset with no dirty suffix; watcher loss yields state/issues plus bounded dirty or all_dirty after reconciliation and is not reset merely because the provider reconciled.
