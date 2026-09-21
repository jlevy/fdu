---
type: is
id: is-01m31hw4f865xyaj8kvtz2n88c
title: "PR #105: H138 sharing allocation guard fails on #97's engine"
kind: bug
status: closed
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T08:38:42.919Z
updated_at: 2026-09-21T16:50:48.899Z
closed_at: 2026-09-21T16:50:48.898Z
close_reason: "Not a #97 regression. The racy counter window explained it; #105 is 19/19 green on its newer tip. The underlying test defects belong to #104 and are fixed there in e38afecc."
resolution: null
duplicate_of: null
---
`Test (ubuntu-latest)` fails at `crates/fdu-core/tests/query_allocations.rs:118`:

    test unfiltered_metric_views_share_one_every_entry_walk ... FAILED
    H138 shares one every_entry walk: [Types, Families] allocated 8214, [Types] allocated 966

The guard is the one PR #104 restores under `fdu-iajs` — it exists because flipping `row_consumers > 1` to never share previously passed every test. It is green on `main` (#104, 19/19) and red once #105 cherry-picks it onto #97, whose engine carries the H147 recycle and H72 d_type keeps.

So either #97 regressed H138 sharing, or the guard's numeric bound does not survive #97's allocation profile. These have opposite remedies, which is why this is not a 'loosen the assertion' fix. Under deep review.

## Notes

Resolved as a flake, not a #97 regression. Deep review verdict, independently corroborated by CI.

PR #105's newer tip (docs-only change above the tested engine) is now 19/19 green: `Test (ubuntu-latest)` job 106267353474 succeeded on the same engine code that failed on `065175ee`. Same engine, opposite outcomes, which is the signature of a race rather than a regression.

Mechanism: `crates/fdu-core/tests/query_allocations.rs` toggles the process-global `ENABLED` flag in `counters.rs` while reading a thread-local count, and libtest runs the file's two tests on parallel threads in one process. The sibling test's `enable(false)` lands mid-measurement and truncates the guard's reading. `reset()` can only clear the calling thread's local, so the race can only truncate, never inflate — which matches every captured failure: `types` came in at 966, 1099, 2053, 2059, 3779 and 3866 against a clean 5134, while `both` was always the clean 8214. A reading of 966 is below the physical floor: `every_entry` does a `path.join` and a `clone` per entry, so 1024 files cannot cost under 2048 allocations.

#97's engine is not on the tested path at all — its diff touches only `scan.rs` and `execution.rs`, and this test never scans. The cherry-pick is byte-identical to #104's commit by `git range-diff`.

A SECOND, more serious defect surfaced in the same review and belongs to #104, not #105: the guard does not catch the flip it names. See fdu-iajs.
