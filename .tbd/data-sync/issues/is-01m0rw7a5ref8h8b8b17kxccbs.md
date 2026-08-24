---
type: is
id: is-01m0rw7a5ref8h8b8b17kxccbs
title: "Coverage reason: partial must say why, not only that"
kind: feature
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0rw7d4h3t49rwvk11cmk5xb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:01.418Z
updated_at: 2026-08-24T23:34:02.744Z
closed_at: 2026-08-24T23:31:10.622Z
close_reason: |
  Shipped earlier in this branch (commit a07fa17, "engine: partial coverage says why, not
  only that"). Re-closed 2026-08-25 after a `tbd sync` reverted the status.

  `Status::Partial` carries a `CoverageReason`: `Building`, `Budget`, `Cancelled`,
  `Inaccessible`, `WatcherGap`, `Failed`. Ordered least-to-most-alarming so
  `Provenance::combine`'s `max` surfaces the worst reason in a subtree rather than an
  arbitrary one -- a consumer asking "why is this partial" gets the answer that matters
  rather than whichever child happened to be visited last.

  The bead's own estimate was corrected during implementation: it claimed four of the six
  reasons were reachable engine state, and only two are. `WatcherGap` in particular is not,
  because `InvalidateSubtree` marks `Freshness::Stale` -- a statement about *trust* rather
  than about coverage. That distinction is now the subject of MB74-C3 on metabrowser PR #74,
  which asks the two projects to pick one axis before the semantic digest compares them.
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

## Notes

REOPENED at `a3960fb`; I closed it again by mistake and have reopened it.

MetaBrowser removed `watcher_gap` from the coverage-reason vocabulary, which settles the
question I raised as MB74-C3 on their PR #74 -- and settles it the way I recommended:
coverage and freshness are independent axes, observation loss is a *freshness* fact.

So `CoverageReason::WatcherGap` is now a dead variant that FDU still exports through both
Rust and Python, even though the implementation correctly never produces it. Exporting a
reason nothing can return invites a consumer to branch on it forever.

FIX. Remove the variant from the enum and from every surface that names it. Observation
loss makes freshness stale and emits a typed issue; coverage becomes partial only when the
answer actually omits scope. Update the ordering comment on the enum, which currently
explains the least-to-most-alarming ordering including this variant.
