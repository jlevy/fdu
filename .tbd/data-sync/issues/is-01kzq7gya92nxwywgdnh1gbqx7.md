---
type: is
id: is-01kzq7gya92nxwywgdnh1gbqx7
title: "PR #3 review R2: make extension roll-ups self-describing and reclaim IDs"
kind: bug
status: closed
priority: 1
version: 6
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzqk48vnyz79nvkv2keacw45
  - type: blocks
    target: is-01kzqk493tkcy6nwws6vf9md7f
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T01:38:20.615Z
updated_at: 2026-08-11T05:12:14.621Z
closed_at: 2026-08-11T05:12:14.621Z
close_reason: Fixed by restoring self-describing named RollUp values at every public Index and IndexHandle boundary, keeping InternedRollUp and ExtId private, reclaiming and reusing extension slots by live-file refcount, and testing cross-index semantics, handle queries, serial/parallel equivalence, snapshot round trips, and churn.
---
CI red at branch tip (run 31449536186, commit 0644f37): scan::tests::parallel_and_serial_walks_produce_the_same_index fails in MSRV (1.85) and Test (ubuntu-latest); passes 5/5 locally on a 10-core M1 Pro, so it is scheduling-dependent, not deterministic.

Root cause: index.rs intern_ext() assigns ids sequentially in first-seen order (id = ext_names.len()). With more than one producer thread the consumer sees extensions in nondeterministic order, so the same tree yields different id assignments run to run. The failure shows identical data with swapped keys: left {0: 12 files/48 bytes, 1: 504/19656}, right {0: 504/19656, 1: 12/48}.

Introduced by bb1529d (perf: intern extension tallies to integer ids, H18).

Scope beyond the test: RollUp.by_ext is public API keyed by ExtId, so any consumer comparing two indexes directly (perf-harness oracle digests, snapshot round-trip comparisons) inherits the nondeterminism. index.rs:71 documents ids as session-local by construction and provides a name-resolving path, so the fix is a choice: (a) compare name-resolved tallies rather than raw ids wherever two indexes are compared, or (b) make id assignment order-independent, e.g. assign after the walk sorted by name. Choose based on whether snapshot serialization or any digest depends on id stability.

Owned by the concurrent perf-loop session working in scan.rs/index.rs; not fixed here to avoid colliding with in-flight work.

## Notes

FDU-PR3-R2 in https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288 broadens the existing nondeterminism bug: keep interned roll-ups private; expose named values through Index and IndexHandle; reject foreign-owner misuse by construction; reclaim unused extension names; test serial/parallel semantics, two-index safety, handle queries, and churn.
