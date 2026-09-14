---
type: is
id: is-01m2esgn21pdr6pafwfvmx7kds
title: Decide whether a projection error fails its projection or the whole ReadRequest
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T01:46:41.088Z
updated_at: 2026-09-14T01:46:41.088Z
---
Open decision from PR #48 review READ-3 (fixed as fdu-q2oj in fa033c2), recorded by the fixer. Related to READ-8 on fdu-91ru.

**What changed.** `Tree` and `RollUp` on a retained non-directory now return the typed `Error::NotADirectory` (Python: `InvalidArgumentError`) instead of `Absent` / `Unknown { Building }`. That fixed the contradiction the review found. But the error is returned from `opened::read::read` itself, so it fails the whole `ReadRequest`, not just that projection. A mixed read of, say, `Lookup(a)`, `Tree(dir)`, and `RollUp(README.md)` loses every result because one projection named a file.

**Evidence (codex/opened-root-inventory-rewrite at f917cb7).**
- `crates/fdu-core/src/opened/read.rs:73`: the RollUp arm returns `Err(Error::NotADirectory(path))` from `read()`.
- `crates/fdu-core/src/opened/read.rs:426`: `tree_projection` does the same.
- A per-projection typed outcome already exists (`ProjectionResult::Limit`), so the envelope can carry a refusal for one projection.
- READ-8 (deferred on fdu-91ru) is the same question from the other side: an oversized continuation record fails the whole read, and the review's fix is "refuse that page, typed and per projection, rather than the whole read".

**Decision to make.** Which failures are request-level (validation, lifecycle, version pin) and which are projection-level (a path of the wrong kind, a continuation record over its bound)? The options:
1. Per-projection typed results: add a variant such as `ProjectionResult::Refused { reason }` and keep request-level errors for things that invalidate every projection.
2. Keep whole-request failure, and say so in the read contract (`engine_contract.rs`) and the MetaBrowser provider contract, so an adapter never batches a path it has not proved is a directory.

**Acceptance.** The decision is written in the plan's read-envelope section and in the provider contract. READ-3's `NotADirectory` and READ-8's `ContinuationRecordLimit` follow the same rule. A mixed-request test pins it on both the Rust and Python surfaces.

Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101. Disposition: https://github.com/jlevy/fdu/pull/48#issuecomment-5657097329
