---
type: is
id: is-01m1b444cnk1qttdgzms5zz013
title: "PR #48 branch is 3.6-10x slower than main: allocator churn, not I/O"
kind: bug
status: in_progress
priority: 0
version: 9
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - regression
  - stack-followup
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-31T05:19:25.577Z
updated_at: 2026-09-16T17:39:56.157Z
---
The opened-root-inventory-rewrite branch has an unreported whole-scan performance regression against main that is larger and broader than the control-table cap this epic started from. It affects trees with NO .gitignore files, so it is not control-file I/O.

Measured, 5 runs each, warm, medians, same host, release builds, 'fdu --color never TREE':

| Tree | .gitignore files | main (b75bf85) | branch tip (4ce1539) | installed (27aeed0) | slowdown |
|---|---|---|---|---|---|
| ~/.rustup/toolchains | 0 | 0.44 s | 1.58 s | 1.57 s | 3.6x |
| ~/wrk/github/thinking-scratchpad | 34 | 0.21 s | 1.82 s | 1.78 s | 8.7x |
| ~/wrk/github/metabrowser | 304 | 1.08 s | 10.81 s | 10.64 s | 10.0x |

A clean release build of the branch tip reproduces the stale installed binary almost exactly, so this is in the code, not in one bad binary, and 20+ commits after 27aeed0 nothing has fixed it.

FDU_COUNTERS=1 on ~/.rustup/toolchains (zero .gitignore, isolating the baseline regression) shows the cause is allocation, not work:

IDENTICAL between main and branch:
  directory opens 3775; entries enumerated 119367; metadata stats 119368;
  file opens 0; bytes read 0; upserts applied 119367; roll-up merges 1111246;
  index entries allocated 119367

DIVERGENT:
  allocations     983,424 -> 4,171,260   (4.24x)
  reallocations   138,299 -> 2,785,222   (20.1x)
  frees           983,418 -> 4,171,244   (4.24x)
  bytes allocated 182,126,386 -> 676,493,704 (3.71x)
  page faults     4,563 -> 7,124         (1.56x)

Same syscalls, same index work, ~4.2x the allocations and ~20x the reallocations. Normalised: about 26.7 extra allocations and 22 extra reallocations PER ENTRY (119,367 entries).

A 20x realloc ratio is the signature of a buffer grown by repeated push without reserve, on a per-entry path. Strong suspects on this branch, not yet confirmed by bisect: '13fe8b4 feat: every path has a portable name' and 'c0fb6de refactor: make a portable path a type, not a String'. Both introduce per-entry path construction.

This also explains the field reports better than the cap does: the agent's 1m17s on ~/wrk and 3m37s on ~ were the branch's regression, not fdu-versus-dust. Against main, fdu beats dust; against this branch, dust wins comfortably.

Acceptance: bisect the branch to the commit that introduces the allocation growth; per-entry allocations and reallocations return to main's order; the three trees above land within noise of main; a counters-based regression check exists so the next such change is caught before merge.

## Notes

PR #51 partially removes the regression but does not meet this P0 acceptance boundary. Independent review at e8f1bed measured the head at about 2.4x main wall time and 4.7x main engine-component time on the same 119,368-entry subject; a 100,001-op public batch remained about 7.7x main. Disposable counter and timing ladders corrected the residual attribution: path-keyed StructuralOverlay ancestry preflight dominates CPU, per-batch impact publication is next, and prepare/effect/AppliedDelta path copies dominate residual allocations. The correctness-first redesign, formal profile protocol, and parity thresholds are now owned by plan-2026-08-31-fdu-streaming-performance-parity.md and epic fdu-748k. This bead remains open until fdu-lj4h proves parity and the allocation guard lands.

2026-09-13 (stack-followup audit): PR #51's description still quotes this bead's current numbers:
- toolchains 1.58 s → 0.70 s, against main's 0.37 s;
- the 304-gitignore tree 11.49 s → 3.92 s, against main's 1.32 s;
- 2.23M / 422k allocations / reallocations.

All were measured before COMMIT-4's port (50e6ca5) replaced the canonical-path copy lane with one pre-sized canonicalizing pass, and they have not been re-measured. Tracked as fdu-wdqf, which fdu-lj4h's quiet-host run on the merged engine supersedes if it lands first.

When closing, also record whether #52's detached builder (no commits, impacts, or journals for detached cold scans) settles #51's "Open for review and redesign" question: should effect recording be lifecycle-gated? Record too where the counters-based per-entry allocation guard landed.

2026-09-14 (triage at c0511e9): not re-measured at the combined head; the acceptance remains `plan-2026-08-31:419-437`. Two of this bead's closing questions are answerable now: the counters-based per-entry allocation guard is `crates/fdu/tests/detached_performance_invariants.rs:66@c0511e9` (both routes, slope between two fixture sizes); and detached cold scans record no effects, impacts, or journal clones by construction (`scan.rs:3530-3536` -> builder under `NoConsequences`, `index.rs:768-792, 1658`), which settles #51's lifecycle-gating question for one-shot. Timing evidence still needs fdu-lj4h's quiet-host or Linux run.

2026-09-16, sanity check on the 0.1.0 release candidate, NOT the plan's parity verdict. Release CLIs `fdu 0.1.0-dev+gb75bf85a3` (pre-rewrite control) and `fdu 0.1.0-dev+g16efcd0ad` (release candidate), `fdu --cache off --color never ~/.rustup/toolchains`, the candidate with `--no-gitignore` so both do the same work (b75bf85 reads no .gitignore; the subject has none). 10 interleaved pairs after 2 warm-ups each, order alternating, bootstrap 95% interval on the median pair ratio. Result: 0.315 s control vs 0.292 s candidate, median pair ratio 0.941 (95% CI 0.813-1.023); peak RSS 49.6 -> 29.5 MiB; totals identical. No sign of the 3.6x whole-scan regression this bead was filed for. Regime: M1 Pro, macOS/APFS, warm cache, host uncontrolled at load average 14.6-16.0 with the user's other Codex and Claude sessions active, and another agent's build lock held during the run. Why this is not the acceptance at plan-2026-08-31:419-437: one subject rather than a control-free and a control-rich tree, CLI wall time only with no component time or allocation counters, 10 pairs, and a loaded host, which the plan says cannot produce a parity verdict. Left open.
