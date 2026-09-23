# Plan: Finish the Alpha Correctness Stack

**Date:** 2026-09-22

**Status:** In Progress

**Tracking:** `fdu-yi1a`; core-model acceptance `fdu-xgjx` under `fdu-h7xy`.

## Overview

An audit of the open alpha PRs on 2026-09-22 found reviewed changes that can advance,
three held PRs, and correctness fixes implemented locally but absent from every open PR.
This plan connects the existing specifications and beads to a reviewable delivery stack.
The [explicit core models plan](plan-2026-09-17-fdu-explicit-core-models.md) continues
to own the behavioral contracts and implementation details.

The work is complete when the remaining correctness contracts are implemented,
independently reviewed, published as dependent PRs, and validated on their final
commits. A passing gate with registered semantic violations does not close conformance.

## Goals

- Preserve filenames and values across every machine format and Python model.
- Make metrics independent of the requested analyzer combination and cache history.
- Compute completeness, errors, coverage, and provenance on every serving route.
- Make snapshot projection and persistence follow one engine-owned policy.
- Finish the execution-plan model required by the core-model specification.
- Resolve the Windows oracle and opened-Python diagnostics findings from the audit.
- Preserve existing work and recorded performance commits while composing the fixes.

## Non-Goals

This work does not claim performance acceptance or publish a release.
The performance campaign retains its measurement obligations.
Only the explicit correctness-preserving deferrals in the core-model plan remain
deferred; an incomplete implementation is not reclassified as a deferral to make the
stack pass.

## Existing Specifications and Beads

| Contract | Owning Beads | Specification and Acceptance |
| --- | --- | --- |
| Per-analyzer values, coverage, and cache reuse | `fdu-azz3`, `fdu-ky5m`, `fdu-7dj6`, `fdu-ugom`, `fdu-h14g`, `fdu-4vbi`, `fdu-82vm`; P2.1 children | Core-model Phase 2 item 1. Every requested metric agrees in each row and total across analyzer combinations; unrequested metrics are absent; operational failures are retried. |
| Typed answers and lossless writers | `fdu-fft9`, `fdu-bqb7`, `fdu-c2ml`, `fdu-up8j`, `fdu-b6iu`; P2.2 children | Core-model Phase 2 item 2. JSON, reconstructed JSONL, strict YAML 1.1/1.2, and public Python agree for reports, changes, and cache status, including adversarial and native paths. |
| Truthful state, reconciliation, and watch handoff | `fdu-awjm`, `fdu-szll`, `fdu-yfb7`, `fdu-ems3`, `fdu-f9fv`, `fdu-bwo2`, `fdu-sb82`, `fdu-08aj`, `fdu-0ywm`, `fdu-aach`, `fdu-jott`, `fdu-4239`; P1.4 children | Core-model Phase 1 item 4 and engine commit boundaries. Unverified descendants are dropped; new observations supersede only their verified scope; unrelated errors remain visible; watch capture covers registration and drain. |
| Controls-off projection on every route | `fdu-ssyf`, `fdu-qsos`; P2.4 children | Core-model Phase 2 item 4. Projected cold-equivalent answers work on retained, opened, report, and initial-watch routes without overwriting the stronger snapshot. |
| One execution plan and persistence policy | `fdu-838z`, `fdu-2o2r`, `fdu-kuev`; P2.3 children | Core-model Phase 2 item 3. All routes consume `Delivery` and `Plan`; read admission, writes, partial outcomes, refresh, and watch persistence have one owner. |
| Windows validity and independent oracle | `fdu-6act`, `fdu-ns3o` | PR #98 and the stored-state validity contract. Oracle timestamps saturate correctly, zero remains zero, locked metadata follows the same documented fallback, and full file identity participates in validation. |
| Opened Python diagnostic parity | `fdu-zjjt` | [Directory query formats](https://github.com/jlevy/fdu/blob/c3aeed8a0a04cecfc18c5719d93e61dbbe4ba449/docs/project/specs/active/plan-2026-09-20-directory-query-formats.md). Bounded Paths and Long reports, including incomplete discovery, expose the same diagnostics through opened and retained Python. |
| Conformance harness integrity | `fdu-j7go`, `fdu-laeo`, `fdu-8whh`, `fdu-k3ca`, `fdu-0ssl` | Positive exact-cache serving controls reject a cache that never serves; the subset includes code-warmed mutations; registered exceptions cannot pass the gate. `fdu-8whh` (artifact identity after changing worktrees) is addressed only by AGENTS.md guidance, with no mechanical check; `fdu-k3ca` has no change in this stack. |
| Final conformance | `fdu-xgjx`, `fdu-fjh1`, `fdu-tyvq` | Empty known-violation registry, independent metric and writer checks, all-platform validation, and a final packaged-artifact rehearsal. |

The implementation beads already exist.
Recovery and publication must update those beads with the actual branch, PR, reviewed
commit, and evidence instead of creating a second set of implementation tickets.

## Published Delivery Order

The native GitHub stack #111 contains, in order,
[#99](https://github.com/jlevy/fdu/pull/99),
[#98](https://github.com/jlevy/fdu/pull/98),
[this plan (#110)](https://github.com/jlevy/fdu/pull/110),
[measured values (#112)](https://github.com/jlevy/fdu/pull/112),
[typed answers (#113)](https://github.com/jlevy/fdu/pull/113), and
[serving state (#114)](https://github.com/jlevy/fdu/pull/114),
[execution planning (#115)](https://github.com/jlevy/fdu/pull/115), and
[harness acceptance (#116)](https://github.com/jlevy/fdu/pull/116), and
[composed directory queries (#117)](https://github.com/jlevy/fdu/pull/117). The
core-model layers form one dependent merge group: the measured-value layer alone does
not provide the final wire contract.
The published bases match the branch directly below each of the nine layers.
Validation below is anchored at PR #117 commit `ff2b07da`.

PR #103 carries the opened-Python diagnostics fix.
Its functional directory-query changes are ported into the final surface composition
without importing unrelated performance ancestry; the source commits and merge
resolutions are recorded for review.
A published PR or a passing focused test is not a completed alpha gate.
The checklists below stay open until their full acceptance evidence is available.

## Design and Recovery Boundaries

The existing local correctness work contains per-analyzer outcomes, shared serializers,
typed status and provenance, watch fixes, and controls-off projection.
Recover each concern into a new branch based on the current reviewed prerequisites.

Recovery is not validation.
In particular:

- A metric-total check cannot detect row errors that cancel each other.
- A parser check for reports does not validate change records or cache status.
- A second native Python dictionary writer can disagree with the shared wire model even
  when the public wrapper happens not to use it.
- A newer child verification must not invalidate an older ancestor pass’s evidence
  outside that child. Error arbitration and entry arbitration need the same scope rule.
- Existing rendering and directory-query changes must compose without dropping either
  contract. Document the combined unreleased report shape under the draft-schema rule in
  [the release process](../../guides/release-process.md).

The composed candidate implements the execution-plan model, including refresh and watch
persistence.
Its implementation status is separate from final conformance and publication
acceptance.

## Implementation Plan

### Implemented Contracts

The 2026-09-22 audit of the composed surface through `a920b393` found the following
implementations present and independently reviewed.
Checked items describe implemented behavior and focused evidence; they do not close the
owning beads or final gates.
Beads such as `fdu-0ssl`, `fdu-bwo2`, `fdu-5w7f`, `fdu-c22r`, `fdu-93e8`, `fdu-6act`,
and `fdu-ns3o` close when their layer merges.

- [x] Finish and independently review the Windows oracle correction on PR #98.
- [x] Preserve PR #99’s reviewed ignore and test-precondition fixes as a prerequisite.
- [x] Implement P1.4’s six children: typed tree status and provenance, bounded path
  errors, source transitions, writing-pass timing, removal of unverified descendants,
  and content-tier provenance.
- [x] Implement P2.1’s five children: per-analyzer records and coverage, name-based
  grouping, requested metric presence, and row-level metric independence.
- [x] Implement P2.2’s eight children: shared answer walks and scalar policy, every
  machine document kind, text, native path identity, and Python wire-model consumers.
- [x] Implement P2.3’s eight children: typed delivery, plan admission and writes,
  refresh, session start and persistence, partial outcomes, and opened roots.
- [x] Implement P2.4’s five children: controls-off projection at shared load boundaries
  and protection of the stronger stored snapshot.
- [x] Compose directory selection and list formats with typed answers, including
  opened-Python diagnostics, unknown ages, and complete healthy siblings in partial
  scans.
- [x] Add positive cache-serving controls, code-warmed mutation histories, and an
  empty-registry requirement to the production conformance gate.
- [x] Correct the raw-extension documentation (`fdu-tp2p`) with the architecture update:
  only a valid-Unicode extension has a string bucket, while Unix and Windows can extract
  it from a native stem that is not valid Unicode.

The follow-up fixes attach to those contracts as follows:

| Contract | Implemented Follow-Ups | Focused Evidence |
| --- | --- | --- |
| Content identity and results | `fdu-ugom`, `fdu-4vbi`, `fdu-h14g`, `fdu-82vm`, `fdu-2gkh` | Operational failures retry; proof-bearing admission governs all content consumers; encoding refusals and per-unit digests are distinct; cold values match cache-history values in rows and totals. |
| Reconciliation and watch | `fdu-yfb7`, `fdu-ems3`, `fdu-sb82`, `fdu-08aj`, `fdu-0ywm`, `fdu-aach`, `fdu-jott`, `fdu-4239` | Overlapping passes preserve newer facts and unrelated issues; overflow is captured before flush acknowledgement; registration, membership, ignored-state transitions, and finite intervals have regressions. |
| Partial directories | `fdu-f9fv`, `fdu-bwo2` | Canonical own-listing evidence agrees across cold, serial warm, and parallel warm routes; failed boundaries publish `DirectoryIncomplete`, preserve newer verification and healthy siblings, and recover on a later successful listing. Public rows test actual permission refusal, cache-route identity, unknown ages, and excluded failed subtrees. |
| Admission and persistence | `fdu-qsos`, `fdu-2o2r`, `fdu-kuev` | Snapshot projection carries its proof; refusals identify cache location, root, and type-rule mismatch; refresh rejects another root and reseeds incompatible stored baselines. |
| Surface parity | `fdu-up8j`, `fdu-b6iu`, `fdu-zjjt`, `fdu-ns3o` | Native paths and bounded content errors survive shared writers; opened Python retains flat diagnostics; Windows oracle follows validity semantics. |

Content analyzer-set containment remains a
[scope deferral](plan-2026-09-17-fdu-explicit-core-models.md#scope-deferrals).
Per-analyzer storage and request-based projection are implemented, but admission still
requires equality of analyzer sets.
A wider sidecar therefore misses for a narrower request; `fdu-7dj6` must not be closed
with a claim of cross-set reuse.

### Remaining Acceptance

- [x] Finish `fdu-0ssl`: distinct golden keys and corpus checks cover all three
  projection refusals, including an actual oversized continuation beside a successful
  lookup (`bdc1c7b2`, published through `3a3ad5c8`).
- [x] Implement and independently review `fdu-bwo2`: the composed full matrix found 12
  cases on each Unix platform where warm reconciliation retained directory-listing
  completeness after an unreadable boundary.
  Canonical fixes `7e6e8acf` and `8decb3bf` withdraw only the affected own-listing
  evidence, publish the withdrawal even when aggregate state is unchanged, and preserve
  newer verification. Public regression `1ad8b8fd` covers exclusions, healthy siblings,
  and cache routes; core regressions also cover recovery.
  Final composed matrix acceptance remains below; no exception was added.
- [x] Install the reviewed, pinned `uv` before release-plan tests (`fdu-5w7f`,
  `c2542f55`); rehearsal run `35812633721` passes planning, all five wheel builds,
  native smoke tests where available, source distribution, crate packaging, and
  immutable artifact inspection.
  This run precedes the final listing-state fix and must be repeated for final
  packaged-artifact acceptance.
- [x] Correct cache-only directory completeness (`fdu-c22r`, `801bf7a7`) and qualify the
  machine-format depth exemption to flat projections (`fdu-93e8`, `ea7baf50`).
- [ ] Run one uninterrupted `make check` and `make cross-lint` on the exact merge
  candidate (`fdu-n2ok`). Earlier evidence below is partial and does not satisfy this.
  On the composed candidate at `ff2b07da` the initial full run stopped when the
  opened-root golden fixture was integrated during the run; the fixture passed on rerun.
  The remaining targets then passed on the `ff2b07da` tracked tree, including
  featureless and watch core tests, Python wheel and source-distribution smoke, parity,
  path-independence subset, and release tests.
  Both macOS and Windows cross-lint targets passed.
  Commit `8edd9b21` changes only numeric-literal formatting in a golden-support test;
  its exact watch-gated normalization test passed.
  This evidence spans scoped runs, not one uninterrupted `make check` invocation.
- [ ] Complete the full Linux, macOS, and Windows path-independence matrix on the final
  merge commit. An earlier run at PR #117 commit `ff2b07da` passed:
  [Run 35815707617](https://github.com/jlevy/fdu/actions/runs/35815707617) passed 16,272
  cases each on Linux and macOS and 14,382 on Windows, with zero recorded known
  violations on all three platforms.
  Metric independence and parser-backed equality for machine documents and public Python
  models are covered by `make check`, so they wait on the item above.
- [ ] Repeat the packaged-artifact rehearsal on the final merge commit.
  The earlier rehearsal on `ff2b07da`
  [run 35815753312](https://github.com/jlevy/fdu/actions/runs/35815753312) passed all
  nine jobs, including five wheels, source distribution, crate packaging, and artifact
  inspection.
- [ ] Complete release end-to-end verification required by `fdu-tyvq` on the final
  commits. Current-head CI still needs the prepared fixture fixes and a green rerun.

The focused audit preceded the composed matrix finding `fdu-bwo2` above.
Accepted scope deferrals and performance work remain separate; passing focused tests or
clearing a review does not establish these final acceptance results.

The dependency order is validity and request prerequisites, measured values, typed
answers and provenance, serving-route state and projection, execution planning, then
final surface composition.
A layer may include tightly coupled migrations needed to compile independently; the PR
description must state that boundary.

### Published Review Fixes

The layer reviews published on 2026-09-22 found no blockers.
The fixes, each with its own delta review, are:

- Windows walks read the root’s volume once instead of demanding a consistent
  observation of a directory whose children are changing, retry a torn entry
  observation, and treat an unavailable device as no filesystem boundary (#98,
  `fdu-39m3`); this was the cause of the stalled Windows churn test.
- YAML quotes letter-initial YAML 1.1 exponent forms such as `e3` (`fdu-ju6w`); text
  output states omitted errors and names a missing path (`fdu-peil`) (#113).
- A failure retained by a pass is stamped at its own Partial marks’ epoch, so a
  concurrent later-finishing pass cannot drop it (`fdu-goge`, #114).
- Opened roots refuse delivery fields they cannot honor (`fdu-yonh`); a refresh repeats
  an owed metadata write (`fdu-9kk8`); `Plan::admit` is the one admission decision
  (`fdu-ftsh`) (#115).
- A serving control fails when both runs fail (`fdu-gzd1`, #116).
- The draft-schema rule, CHANGELOG notes, and examples (`fdu-9vkc`), and this plan’s
  evidence (`fdu-9mn0`) (#117).

### Publish and Verify the Stack

- [x] Publish the final surface layer, verify every PR base against the branch directly
  below it, and verify the nine-PR GitHub stack object (#111).
- [x] Complete independent review of every layer.
  Private reviews preceded publication; the published layer reviews and their
  dispositions are on each PR, and every fix commit made in answer to them received a
  separate delta review before merge.
- [x] Carry current main’s fixes forward without restoring obsolete test baselines,
  deleting newer guards, or rewriting commits cited by performance evidence.
  Main commit `11a6dc31` is an ancestor of the composed candidate.
- [ ] Finish current-head CI and the final platform checks for every published layer
  after the prepared fixture fixes.
- [ ] Update the core-model specification, work index, and beads from actual evidence.

The performance composition beads `fdu-qx0e` and `fdu-8fax` concern separate performance
branches. This isolated correctness stack does not incorporate those branches, so their
composition does not gate correctness acceptance.

### Merge Order After Correctness

Merge the correctness layers #99 → #98 → #110 → #112 → #113 → #114 → #115 → #116 → #117
first. Integrate the separate performance layers #94 → #97 → #105 only after semantic
review of their composition and resolution of #105’s held documentation findings.
Integrate #109 last with its evidence-kept-arm migration.
The functional work from #96 and #103 is already ported into #117; merging either again
would duplicate it.

A rehearsal found #117 plus #94 clean.
Adding #97 and #105 produces conflicts in `execution.rs` and `scan.rs`. PR #109 composes
cleanly with #117 alone, but following the performance layers requires three more
performance-harness and documentation reconciliations.
Merging #103 directly into #117 produces broad conflicts across core, Python, renderers,
and goldens. The future integration work belongs to `fdu-qx0e` and `fdu-8fax`.

## Testing Strategy

Focused regressions establish each behavioral fix before the broad gates run.
Use deterministic synchronization for concurrent state transitions and real parser
oracles for serialization.
Preserve named patterns when updating goldens and inspect every semantic change.

The final candidate must pass `make check`, `make cross-lint`, the full Linux/macOS/
Windows path-independence matrix with an empty known-violation registry, metric
independence, and equality of every machine document.
Native Windows tests supply the evidence unavailable on a macOS host.
A cross-check that compiles Windows code is not a Windows runtime test.

Run one full local build gate at a time.
Retain logs with the tested commit, and rerun the affected evidence after a fix or
semantic merge resolution.
Do not reinterpret a missing, skipped, or unfinished check as a pass.

## Rollout Plan

PRs remain unmerged during implementation and review.
Ready-to-merge means the layer’s findings are resolved, its final commit is reviewed,
and its required checks pass.
Implementation beads retain a publication and validation status until their layers
merge; release acceptance remains open until the composed candidate meets its contract.
Release publication remains a separate maintainer action.

## Open Questions

No product decision blocks the confirmed fixes.
Resolve implementation choices against the owning core-model and directory-query
specifications, and record any newly found correctness defect as a bead before handoff.

## References

- [Design principles](../../architecture/fdu-design-principles.md)
- [Engine architecture](../../architecture/fdu-engine-architecture.md)
- [Surface architecture](../../architecture/fdu-surface-architecture.md)
- [First-release verification](plan-2026-09-18-fdu-first-release-verification.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
