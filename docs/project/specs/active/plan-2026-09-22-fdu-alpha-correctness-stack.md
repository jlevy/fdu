# Plan: Finish the Alpha Correctness Stack

**Date:** 2026-09-22

**Status:** In Progress

**Tracking:** `fdu-yi1a`; core-model acceptance `fdu-xgjx` under `fdu-h7xy`.

## Overview

The alpha PR audit found reviewed changes that can advance, three held PRs, and
correctness fixes implemented locally but absent from every open PR. This plan connects
the existing specifications and beads to a reviewable delivery stack.
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
| Truthful state, reconciliation, and watch handoff | `fdu-awjm`, `fdu-szll`, `fdu-yfb7`, `fdu-ems3`, `fdu-f9fv`, `fdu-sb82`, `fdu-08aj`, `fdu-0ywm`, `fdu-aach`, `fdu-jott`, `fdu-4239`; P1.4 children | Core-model Phase 1 item 4 and engine commit boundaries. Unverified descendants are dropped; new observations supersede only their verified scope; unrelated errors remain visible; watch capture covers registration and drain. |
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

The native GitHub stack is #111. Its current lower layers are
[#99](https://github.com/jlevy/fdu/pull/99),
[#98](https://github.com/jlevy/fdu/pull/98),
[this plan (#110)](https://github.com/jlevy/fdu/pull/110),
[measured values (#112)](https://github.com/jlevy/fdu/pull/112),
[typed answers (#113)](https://github.com/jlevy/fdu/pull/113), and
[serving state (#114)](https://github.com/jlevy/fdu/pull/114). Execution planning,
harness acceptance, and surface composition extend that stack.
The core-model layers form one dependent merge group: the measured-value layer alone
does not provide the final wire contract.

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
Old worktrees remain intact, including their uncommitted patches.

Recovery is not validation.
In particular:

- A metric-total check cannot detect row errors that cancel each other.
- A parser check for reports does not validate change records or cache status.
- A second native Python dictionary writer can disagree with the shared wire model even
  when the public wrapper happens not to use it.
- A newer child verification must not invalidate an older ancestor pass’s evidence
  outside that child. Error arbitration and entry arbitration need the same scope rule.
- Existing rendering and directory-query changes must compose without dropping either
  contract. Document the combined unreleased report shape under the repository’s accepted
  pre-1.0 schema policy.

The execution-plan model is still required; recovering serializers and projection alone
does not finish the core-model specification.

## Implementation Plan

### Recover and Complete the Owning Layers

- [x] Finish and independently review the Windows oracle correction on PR #98.
- [x] Preserve PR #99’s reviewed ignore and test-precondition fixes as a prerequisite.
- [ ] Recover independent metric records and complete row-level acceptance checks.
- [ ] Recover typed status, provenance, and shared writers; finish all document kinds
  and public Python consumers.
- [ ] Finish reconciliation arbitration, watch transitions, and all-route projection.
- [ ] Implement the remaining execution-plan and persistence-policy work.
- [ ] Fix opened Python diagnostics on PR #103 and compose its list contract with the
  shared answer model.
- [ ] Strengthen positive serving and subset history checks, and require an empty
  registry in the conformance gate.
- [x] Correct the raw-extension documentation (`fdu-tp2p`) with the architecture update:
  only a valid-Unicode extension has a string bucket, while Unix and Windows can extract
  it from a native stem that is not valid Unicode.

The dependency order is validity and request prerequisites, measured values, typed
answers and provenance, serving-route state and projection, execution planning, then
final surface composition.
A layer may include tightly coupled migrations needed to compile independently; the PR
description must state that boundary.

### Publish and Verify the Stack

- [ ] Publish dependent PRs with each base set to the branch directly below it and
  verify the GitHub stack object.
- [ ] Give every implementation layer an independent Astra review.
  Sol may perform mechanical recovery and conflict resolution; semantic resolutions
  receive Astra review before acceptance.
- [ ] Carry current main’s fixes forward without restoring obsolete test baselines,
  deleting newer guards, or rewriting commits cited by performance evidence.
- [ ] Resolve the recorded composition work (`fdu-qx0e`, `fdu-8fax`) if the performance
  branches are incorporated into the candidate.
- [ ] Run the full local handoff gate and platform checks on the resulting candidate;
  finish CI for every published layer.
- [ ] Update the core-model specification, work index, and beads from actual evidence.

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
