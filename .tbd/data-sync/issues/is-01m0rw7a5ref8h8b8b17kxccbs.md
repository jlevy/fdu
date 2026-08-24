---
type: is
id: is-01m0rw7a5ref8h8b8b17kxccbs
title: "Coverage reason: partial must say why, not only that"
kind: feature
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0rw7d4h3t49rwvk11cmk5xb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:01.418Z
updated_at: 2026-08-24T16:29:13.494Z
closed_at: 2026-08-24T16:29:13.494Z
close_reason: |
  Coverage now says why, and the shape enforces that it cannot say why when it is complete.

  Status::Partial carries a CoverageReason rather than a reason sitting beside the status.
  Two fields could disagree -- a complete value with a reason, a partial one without --
  and this makes both unspellable. It also keeps the derived Ord correct for
  Provenance::combine: Complete sorts below every Partial, two partials sort by reason,
  so taking the maximum still yields the least trustworthy contributor, and now surfaces
  the reason a consumer most needs to act on.

  THE BEAD'S ESTIMATE WAS WRONG AND CHECKING IS WHAT FOUND IT. It said four of six reasons
  were already engine state. Two are:
    Inaccessible -- a scan or reconciliation that completed around errors
    Failed       -- one that returned an error instead
  The other four are declared and deliberately unreachable, each documented with what would
  make it real:
    Building, Cancelled -- need the session (fdu-4o0m); an in-progress reconciliation marks
                           Freshness::Reconciling, which is coverage-COMPLETE by design
    Budget              -- fdu has no walk budget
    WatcherGap          -- and this is the interesting one. An InvalidateSubtree marks
                           Freshness::Stale, a statement about TRUST, not coverage: the
                           totals still account for every entry, they may simply be wrong.
                           Reporting WatcherGap there would tell a consumer part of the
                           subtree is missing about a subtree that is entirely present.
  All six ship as vocabulary so a consumer matches exhaustively once, not twice.

  WHERE THE REASON COMES FROM. FreshnessMark carries Option<CoverageReason>, set at the
  site that marks a subtree -- the only place that still knows why. ScanReport::coverage()
  and ReconcileReport::coverage() derive it once so two callers cannot name different
  reasons for the same condition; the failed-pass branch names Failed explicitly.
  Index::coverage_at resolves it over overlapping marks by the same worst-wins rule
  freshness_at uses, and status_of now reads it instead of re-deriving from a Freshness
  that has forgotten the cause.

  set_initial_freshness(bool) -> set_initial_coverage(Status), and finish_reconcile's
  `complete: bool` -> `coverage: Status`. A bool cannot carry a reason, and threading one
  beside it would have left the two free to disagree at every call site.

  SURFACED: Provenance.reason in Python, separate from the status label so a consumer that
  only branches on complete-or-not needs no change; fdu.CoverageReason exported.

  TESTS. Three in the engine plus a Python smoke check. The load-bearing one is
  a_dropped_watch_queue_costs_trust_and_not_coverage, which asserts an ABSENCE -- so it was
  mutation-checked by implementing the plausible-but-wrong version (InvalidateSubtree
  marking Partial/WatcherGap) and confirming it fails. The others pin propagation to
  ancestors through the real provenance read path, and worst-reason-wins on combine.

  make check passes. 446 engine tests.
resolution: null
duplicate_of: null
---
MetaBrowser's implemented provider contract (arch-inventory-provider.md, PR #44 comment
2026-08-24) requires every result to carry coverage as "either complete with no reason or
partial with one of `building`, `budget`, `cancelled`, `inaccessible`, `watcher_gap`, or
`failed`".

fdu has the bit and not the reason: engine_contract::Status is a bare two-variant enum,
Complete | Partial. A consumer is told a subtree is incomplete and cannot tell a walk
still running from a directory it could not read from a dropped watch queue -- three
situations with three different correct UI responses.

Every reason already exists as engine state, which is what makes this small:
  building     -> a walk in progress (Freshness::Reconciling under an additive scan)
  inaccessible -> reconciliation errors already collected per path
  watcher_gap  -> InvalidateReason::{WatchOverflow, UnpairedRename, WatchSetupRace}
  failed       -> a scan or apply error
  cancelled    -> needs the session (fdu-4o0m)
  budget       -> needs a walk budget, which fdu does not have yet

So this lands as a reason carried alongside Status, populated from the invalidation and
error paths that already know it. Two reasons cannot be filled until the session exists;
they should be declared and left unreachable rather than omitted, so the vocabulary
matches the contract and the gap is visible.

Watch the freshness-vs-coverage distinction Index::status_of already documents: Stale and
Reconciling describe TRUST, not coverage, and only Freshness::Partial is genuinely missing
coverage. The reason attaches to the latter.
