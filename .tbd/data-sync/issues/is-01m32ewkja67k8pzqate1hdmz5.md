---
type: is
id: is-01m32ewkja67k8pzqate1hdmz5
title: "Serves is not load-bearing: one call site collapses it to a bool, and its projection lives elsewhere"
kind: bug
status: open
priority: 0
version: 1
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:05:47.082Z
updated_at: 2026-09-21T17:05:47.082Z
---
Verified by execution in an adversarial review, 2026-09-21.

`crates/fdu-core/src/lib.rs:430-448` — `snapshot_scope_serves(...) -> bool` is the ONLY call site of `serves_snapshot` (`lib.rs:436`), and it immediately collapses the enum with `serves_snapshot(..) == Serves::Exact`. Lines 441-447 then decide a second serving relation inline as a bool: controls-on serving controls-off, when `ReportOnly && !policy.scans() && the scope differs only in ignore rules && the wanted fingerprint is 0`.

The projection that relation implies is applied in a different file, keyed on a different fact: `execution.rs:335-337` overwrites `answer.scope` and calls `forget_ignore_classification` when `!config.scan.control_identity().is_observed()` — on the request alone, not on whether the projection was taken.

That is exactly the "predicate separable from its projection, rule stated in two places" shape the Serves enum is supposed to prevent.

Proof: with `serves_snapshot` hard-wired to `Refuse` for every input, `fdu <root> --cache only` refuses, but `fdu <root> --cache only --no-gitignore` still answers with `source: cache_only`, served through the inline branch.

Also: `Serves::Exact` carries no value, so a future `ProjectControlsOff` variant is a tag every call site must handle. The one existing site uses `== Serves::Exact`, which silently treats any new variant as a miss — reproducing `projection-route`: served on the route that was updated, refused on the one that was not.

Fix direction from the review: `serves_*` returns `Option<Projection>` where `Projection` carries `fn apply(self, Index) -> Index`, applied inside `snapshot::load` so no route can obtain an index without the projection; `#[must_use]`; no `== Serves::Exact` comparisons.
