---
type: is
id: is-01m0rw7bvxtw87tgde30emgs56
title: "Invalidation vocabulary: dirty query kinds, all-dirty, and reset as distinct signals"
kind: feature
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:03.165Z
updated_at: 2026-08-25T03:44:15.611Z
closed_at: 2026-08-25T03:44:15.610Z
close_reason: |
  Shipped, both halves. `make check` green, parity holds.

  THE FOUR SIGNALS, now distinct.

    reset       -> consumer history expired (journal truncation), and nothing else beside it
    all_dirty   -> the dirty path list gave up enumerating
    dirty paths -> `dirty_rollups`, bounded
    dirty kinds -> `dirty_queries`, the nine projections
    state-only  -> `Batch::state`, delivered by `fdu-jxs0`

  RESET WAS TWO FACTS ABOUT TWO PARTIES WEARING ONE FLAG, and that was the substantive half.
  A dropped event means the *provider* stopped observing precisely; it re-scans, so the index
  is right, the rows are complete, and the consumer's position is resumable. A truncated
  journal means the *consumer's* history has expired. Reporting the first as the second costs
  a full re-read on every kernel queue overflow and teaches a consumer that reset does not
  mean what it says. Observation loss is now `IssueKind::ObservationGap` beside the re-scan's
  own invalidations; reset is truncation only, and `Batch::reset_at` makes "reset replaces
  every other signal" a property of the constructor rather than of a reviewer's attention.

  DIRTY QUERY KINDS. `QueryKind` is the contract's nine, declared whole. `dirty_rollups` says
  which *paths* moved; this says which *answers* those paths were part of, which is the
  question a consumer holding one projection actually has -- otherwise every consumer
  re-derives it from change paths, and differently wrong.

  Absence is the guarantee, presence is not, and the asymmetry runs one way on purpose:
  naming a kind that turns out unaffected costs one re-read, and omitting one that is affected
  leaves an answer nothing will ever contradict. So the derivation is conservative, and the
  tests assert the *negative* cases, which are the ones a consumer can act on.

  Two asymmetries come from what an op does not say. `Remove` does not say what it removed, so
  a removed directory is treated as having taken files with it -- the same guess
  `dirty_rollups` makes. An `Upsert` does not distinguish created from modified, so a catalog
  of identities is named for both though only the first moves it. Both are recorded where the
  derivation happens.

  What the answer is worth is visible in what it excludes: a directory's own attributes do not
  name `Recent` or `Catalog`; a state-only delta names `Diagnostics` and nothing else; a
  re-tag names the tag-bearing projections but not mtime order. `Metadata` is declared and
  never named -- a rebind keeps the tag-rules fingerprint, because rebinding changes what the
  rules read rather than which are enabled, and changing those opens a new session.

  `QueryKind` joins `IssueKind` and `Phase` in `scripts/check-vocabularies.mjs`, so a member
  added to one surface and not the other fails in a second rather than in a ten-minute gate.

  A TEST DEFECT THE MUTATION CHECK FOUND, worth recording because the test passed. The Python
  loop nested `for batch in watch` inside a `while deadline`, and the inner loop broke only on
  a dirty batch -- so once the tree went quiet it never returned to the deadline. It passed
  only because the thing it waited for arrived in the first dirty batch; under mutation it
  hung instead of failing. One loop now, deadline checked every iteration. A test that cannot
  fail is worse than no test, and a test that hangs instead of failing is worse than both.
resolution: null
duplicate_of: null
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

The reset half is shipped. `make check` green, parity holds. The bead stays open for the
dirty-query-kinds half.

RESET WAS TWO FACTS ABOUT TWO PARTIES WEARING ONE FLAG. A dropped event, an unpaired
rename, or a watch registered after its directory already had contents means the *provider*
stopped observing precisely; it re-scans, so the index is right, the batch's rows are
complete, and the consumer's own position is perfectly resumable. A truncated journal means
the *consumer's* history has expired: nothing can be replayed to it. Reporting the first as
the second costs a full re-read on every kernel queue overflow, and teaches a consumer that
reset does not mean what it says.

Now: `reset` is journal truncation and nothing else. Provider observation loss becomes a
typed `IssueKind::ObservationGap` on the batch, beside the re-scan's own `Invalidate`
changes and the dirty set it moved -- which is what the consumer contract asks for.

RESET REPLACES EVERY OTHER SIGNAL, as the contract's own validation requires
(`reset and (all_dirty or dirty_paths or dirty_queries)` is rejected there). `Batch::reset_at`
is a constructor rather than a struct literal, because the shape *is* the contract: a batch
saying "re-read everything, and also here are the changes" gets its second half applied to
state the consumer just discarded. `Recovery` is gone -- it existed to combine two things
that turned out not to combine.

`ObservationGap` is an issue, not a coverage reason, and that distinction is the same one
that kept `CoverageReason::WatcherGap` out earlier: coverage is how much of the tree an
answer accounts for, and after the re-scan it accounts for all of it. What moved is how far
the stream between then and now can be trusted.

TESTS. `losing_watch_precision_is_an_issue_rather_than_a_reset` applies a `WatchOverflow`
escalation directly and asserts not-reset, the typed issue naming its subtree, the re-scan's
change still delivered, and the aggregates still named. Mutation-checked. And
`a_reset_batch_carries_nothing_a_consumer_could_apply` pins both invariants, with the two
ways of getting them wrong spelled out so the checker has teeth.

A DRIFT THIS EXPOSED, now checked rather than remembered. `IssueKind::ObservationGap` was
added to the engine and not to the Python `StrEnum`, so the first batch carrying it raised
`ValueError` from inside `_operation_error`. The parity harness caught it -- but only
because a watch capture happens to provoke a `WatchSetupRace`, and only after a ten-minute
gate. `scripts/check-vocabularies.mjs` now compares the two declarations of `IssueKind` and
`Phase` directly and fails in a second; it is in `make test`. Mutation-checked in both
directions.

STILL OPEN: dirty query kinds. fdu can say which *paths* moved but not which of the
contract's nine *projections* are stale, so a consumer holding a `recent` list re-derives
whether its own view is affected.
