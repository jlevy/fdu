---
type: is
id: is-01m0rw7a5ref8h8b8b17kxccbs
title: "Coverage reason: partial must say why, not only that"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0rw7d4h3t49rwvk11cmk5xb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:01.418Z
updated_at: 2026-08-24T03:16:02.656Z
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
