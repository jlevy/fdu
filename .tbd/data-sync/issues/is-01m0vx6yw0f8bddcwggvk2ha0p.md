---
type: is
id: is-01m0vx6yw0f8bddcwggvk2ha0p
title: "A native walk budget: stop discovery at the cap, and say so"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T07:30:01.728Z
updated_at: 2026-08-25T08:59:08.587Z
closed_at: 2026-08-25T08:59:08.587Z
close_reason: "Native walk budget shipped: --max-files / ScanOptions.max_files, fingerprinted into ScanScope, snapshot format 4, coverage Partial(Budget) with a typed ResourceStop issue. Eight engine tests plus a Python smoke check, mutation-checked. Reconcile-side budgeting is recorded in the notes as deliberately out of scope; watching a capped index is refused."
resolution: null
duplicate_of: null
---
The interactive-client contract declares a file budget as *scope* -- validated as positive,
a component of the scope fingerprint, and enforced by the reference provider, which stops
discovery at it and reports partial coverage with reason `budget` plus a typed
resource-budget issue.

fdu had none. `CoverageReason::Budget` was declared and documented as unreachable, and
`ScanScope` carried depth, symlink, filesystem and hidden-admission facts with no entry cap.
Two providers given the same scope fingerprint therefore returned different inventories:
complete where the other was truncated.

DECIDED (MB74-D2, answered on PR #47 at head `eaae030`): fdu implements the native budget.
The cap stays fingerprinted semantic scope on the consumer side because the shipped
resource stop, partial coverage and typed issue must be preserved, and projection limits
are explicitly not a substitute.

The bound has to stop the *walk*, not truncate an *answer*. A budget that only bounds a
projection leaves the tree read anyway and saves nothing -- the cost it exists to avoid is
the discovery, not the serialization.

What this touches:
- `ScanConfig` gains the cap; `ScanScope` reports it and folds it into the scope
  fingerprint, so a snapshot taken under one cap cannot be reused under another.
- Snapshot `FORMAT_VERSION` and `SERIALIZED_SCOPE_BYTES` move for the new scope field.
- Discovery stops at the cap and marks coverage `Partial(Budget)` from the stopping point
  upward, which is what finally makes that variant reachable.
- A typed `IssueKind::ResourceStop` rides on the run facts, so a consumer sees why rather
  than inferring it from a count.
- `--max-files` on the SCOPE axis and `ScanOptions.max_files`, both surfaces, one judge.

Acceptance: a tree larger than the cap stops reading at it -- proven by counters, not by
the row count, since a truncating projection would pass a row-count assertion; coverage is
partial with reason `budget`; the issue is typed; the fingerprint differs between two caps
over the same tree; and a `--cache only` open under a different cap refuses the snapshot.

## Notes

SHIPPED. `ScanConfig::max_files` / `ScanScope::max_files`, `--max-files`,
`ScanOptions.max_files`, snapshot format 4.

Three things this turned out to require that the plan did not say:

**The budget cannot ride on `should_descend`.** That looked like the one shared decision
point both walkers already consult -- and reconciliation consults it too, where a `false`
for a directory means "out of scope, so remove what the index holds below it". Correct for
depth and filesystem bounds, which are fixed for the life of an index; catastrophic for a
bound that flips partway through a walk, which would have deleted subtrees for the crime of
being discovered late. The budget is asked at the two discovery sites instead, where the
only consequence of `false` is that a directory is not enqueued.

**Checking only at enqueue time does nothing on a wide tree.** The root holds no files of
its own, so nothing is charged before all of its children are queued. The check that
matters is at *take* time, and the first version of this shipped with only the enqueue
gate and a test that passed because the fixture was deep enough to hide it.

**A directory is read whole or not at all.** The cap is checked between directories, never
inside one: a half-listed directory reports its own tallies as complete, silently, which is
the one thing a bound must never be. The cost is an overshoot bounded by the directories
already in flight, and that is the right trade.

Two defects the budget exposed in existing code, both the same shape -- a second copy of
something the engine already assembles:

- `PyIndex.status` built its own four-field envelope, so the standalone accessor reported
  no lifecycle phase and no coverage reason while a bundled read of the same index reported
  both. Now routed through `Index::engine_state()`.
- The PyO3 open path rebuilt run facts from `report.errors()` plus the analysis failure,
  which silently dropped every typed condition that is not a per-path I/O error. A
  budget-stopped walk reported partial with an empty reason list, because nothing had
  failed and there was nothing to reconstruct it from. `OpenReport::issues()` is now the
  one assembler and the binding asks for it.

`Error::ScanScopeMismatch` and `Error::SubtreeOutsideScanScope` now box their `ScanScope`s:
the added field pushed the variant past clippy's large-Err threshold, and that variant sets
the size of every `Result` the crate returns.

Tests: `crates/fdu-core/tests/walk_budget.rs`, eight cases, plus
`public_smoke.py:check_a_file_cap_stops_the_walk_rather_than_the_answer`. The load-bearing
assertion is on **directories read**, because every softer assertion a capped result invites
-- fewer rows, partial coverage, a typed issue -- passes just as well against a projection
limit that reads the whole tree and trims the answer.

Mutation-checked, and one mutation *passed* and forced the test to be rewritten: the
per-worker-budget mutation survived `files_walked < whole tree`, because at a cap of two
directories the shared bound and the per-worker bound are the same number. The test now
states the invariant -- `files_walked <= cap + workers * PER_DIR` -- at a cap where the two
hypotheses give 56 and 80.

Also pinned: a budget-stopped walk is never saved as a warm baseline. It would be a stable
answer to reread and a wrong one to build on, since a later reconcile against it would treat
"never discovered" and "since deleted" as the same thing.

NOT DONE, and deliberate: the cap governs discovery. Reconciliation walks from the index and
does not consult it, so a refresh of a capped index can grow it past the cap. Doing that
correctly needs a budget seeded from the index's current file count and an additive-only
rule, and it must never reach the removal branch above. Watching a capped index is refused
outright for the adjacent reason -- a creation inside an unread subtree would be admitted
while its siblings stayed missing, assembling a subtree nobody walked one event at a time.
