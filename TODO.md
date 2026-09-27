# fdu TODO

Current work is tracked in tbd.
This page maps workstreams to their plans; bead status, dependencies, and evidence are
authoritative. Completed work is in [TODO.archive.md](TODO.archive.md).
The latest
[tracking review](docs/project/reviews/review-2026-09-27-tracking-consistency.md)
records the status reconciliation and remaining evidence gaps.

## Delivery and Release

fdu 0.1.0 is published on crates.io, PyPI, and GitHub.
Publication does not complete every first-user verification exercise.
The analysis and inventory changes are implemented and verified in PRs #133 and #135 but
await merge; research is in #130.

| Workstream | Owner | Remaining work / governing document |
| --- | --- | --- |
| Current PR stack | `fdu-0gqc` | Ordered merge and post-merge verification of #130, #133, #135 and the tracking follow-up; [release process](docs/project/guides/release-process.md). No next release is implied. |
| Published 0.1.0 verification | `fdu-gjc2`, `fdu-yfej`, `fdu-vxvm` | Record the remaining [published-channel first-user checks](docs/project/specs/active/plan-2026-09-18-fdu-first-release-verification.md). |
| Release automation | `fdu-zr73` | [Packaging follow-ups](docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md), including registry propagation retry (`fdu-zx9y`). |
| Stabilization review residue | `fdu-82h4`, `fdu-6wvb`, `fdu-fhde` | Operational review/handoff parents retain their open findings; their bead descriptions own that scope. A merged PR alone does not close a finding. |

Current behavior is documented in the [usage guide](docs/usage.md),
[machine-output reference](docs/machine-output.md), and emitted `fdu --skill`. Research
and completed plans preserve decisions; their older examples do not override these
references.

## Engine and Product Follow-Ups

| Workstream | Owner | Remaining work / plan |
| --- | --- | --- |
| Core contracts | `fdu-h7xy`, `fdu-azz3` | [Explicit core models](docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md): per-analyzer records and remaining model decisions. |
| Alpha acceptance evidence | `fdu-yi1a` | [Alpha correctness stack](docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md): final matrix/evidence reconciliation; bounded long-line memory remains `fdu-1zb6`. |
| Opened-root engine and integration | `fdu-snej`, `fdu-u7vo`, `fdu-2lkf` | [Opened-root inventory](docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md): measured indexes/continuations, MetaBrowser adoption, composed acceptance, home-directory scale. |
| Streaming performance acceptance | `fdu-748k` | [Streaming parity](docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md): remaining parity proof and regression gates after #52. |
| Warm progressive serving | `fdu-wpa0` | [Progressive results](docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md): post-0.1 lazy warm open, persisted roll-ups and mixed-source provenance. |
| Directory queries | `fdu-65x1` | [Directory formats](docs/project/specs/active/plan-2026-09-20-directory-query-formats.md): predicate decisions (`fdu-2udc`), flat-read cost (`fdu-uea9`), benchmark naming (`fdu-afc4`). |
| CLI and skill distribution | `fdu-pxeb`, `fdu-8e1b` | [Follow-up plan](docs/project/specs/active/plan-2026-09-27-cli-and-skill-followups.md): cache retention, Windows coverage, output/clock decisions, benchmark/tracking gaps, committed skill and foreign-repository sdist verification. |
| Progress indicator | `fdu-vngp`, `fdu-2e8o` | [Progress plan](docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md): shipped indicator; remaining acceptance is recorded in the plan and linked beads. |
| Rust quality | `fdu-dxee` | [Quality plan](docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md): parser/commit-failure state-machine coverage (`fdu-471a`). |
| Disk checkpoints and allocation | `fdu-uwhl`, `fdu-8ybz`, `fdu-579b` | [Disk checkpoints](docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md): probe and store design; unique hard-link accounting remains separate. Current allocation counts paths, not reclaimable bytes. |
| Deferred extensions | `fdu-x746` | [Post-phase-1 roadmap](docs/project/specs/future/plan-2026-08-09-fdu-post-phase-1-roadmap.md). |

## Performance and Evidence

| Workstream | Owner | Remaining work / plan |
| --- | --- | --- |
| Original walker acceptance | `fdu-qfz6` | [Phase 1](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md): shipped increments do not satisfy every original benchmark exit criterion. |
| Evidence harness and Linux coverage | `fdu-d5e1`, `fdu-0myw` | [End-to-end evidence](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md): reproducible regimes, comparator contracts and coverage. |
| Campaign 2 | `fdu-j2ka`, `fdu-xde5`, `fdu-d4kg` | [Campaign plan](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md): consumer representation, measured floor and platform priorities; PGO adoption (`fdu-pdne`) remains distinct from the completed screen. |
| Remaining hypotheses | `fdu-8ya1`, `fdu-4sg3` | [Post-H115 headroom](docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md) and Linux handoff. `fdu-4sg3` is an undecomposed handoff referencing existing tasks, not a completed empty epic. |
| Performance record | `fdu-j5k6` | [Record/report plan](docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md): artifact backfill (`fdu-1dtd`), cross-platform matrix (`fdu-uxl0`), per-platform report sections (`fdu-72bn`), harness lint (`fdu-tt49`). |
| Evidence scope | `fdu-ug4y` | [Scope enforcement](docs/project/specs/active/plan-2026-08-23-experiment-evidence-scope.md): partially implemented, with transfer/provenance enforcement outstanding. |
| FSEvents scoped revalidation | `fdu-7w9a` | [Replay/revalidation plan](docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md): #131 is an open probe/design PR, not production journal serving. |
| Reusable experiment framework | `fdu-7yx4` | [Framework extraction](docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md): reusable skill landed; remaining framework scope stays open. |

The merged #132 comparisons are qualified exploratory measurements on an uncontrolled
host. The quiet native/wheel release cell (`fdu-ow8y`) remains unresolved.
`fdu-s234` owns reconciling the claim policy across these two evidence classes; the
tracking cleanup does not waive a measurement gate.

Other standalone findings remain discoverable through `tbd ready` and `tbd list`. They
need not acquire an invented spec or parent: the bead itself can own a narrow bug,
decision, or operational task.
Closed parents can retain open review follow-ups when the child’s scope is explicitly
separate from the parent’s completed increment.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
