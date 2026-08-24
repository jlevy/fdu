# fdu TODO

Outstanding work at the top level: one entry per open epic and per incomplete plan spec.
Finished work moves to [TODO.archive.md](TODO.archive.md).

This page is a map, not the tracker.
The tracker is tbd — every entry below names its bead, and the bead holds the current
detail, dependencies, and evidence.
Where an entry names a spec, that spec owns the design and the phase list.
Ask the agent for status rather than reading bead counts here as authoritative; the
counts are a snapshot of when this page was last edited.

## Epics

Sixteen open, ordered by how much is outstanding under each.

| Epic | Open | What remains | Spec |
| --- | ---: | --- | --- |
| `fdu-qfz6` — fdu phase 1: fastest walker with full stats, proven by benchmark | 15 | The original delivery epic. Its benchmark gate is the definition of “done” for the walker. | [phase-1](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md) |
| `fdu-0myw` — Linux performance validation and optimization | 13 | Linux is measured but thinly: the ledger’s regime coverage is almost entirely macOS/APFS. | [end-to-end performance](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) |
| `fdu-u7vo` — fdu for interactive clients: the metabrowser contract | 17 | Shared reads during a write (a measured drop-in blocker), partitioned (gitignore/hidden) tallies, a customizable roll-up taxonomy with a browsing group level, the embedder watch contract, the session integration shape, and the adoption proof. | [interactive client integration](docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md) |
| `fdu-wpa0` — progressive results: order, provenance, sessions, lazy open | 11 | Values labelled per-value with provenance, so an approximate answer can be shown and then converge. Also owns the prefer-cache tier (`fdu-wu6w`), which the completed cache-layers plan first named. | [progressive results](docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md) |
| `fdu-d5e1` — reproducible end-to-end performance evidence | 8 | The generated-corpus harness this project’s loop borrows from. | [end-to-end performance](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) |
| `fdu-xde5` — H86: consumer representation as one structural experiment | 5 | Campaign 2’s centerpiece. The spike measures **1.06× the parallel syscall floor** where the index tier runs 2.68×, and the ~15-point real-tree tax lands in the code it deletes. One experiment, floor-anchored targets, not piecemeal. | [campaign 2](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) |
| `fdu-7yx4` — extract the experiment loop as a reusable framework | 8 | The contract, statistics, generated views, and protocol as something other campaigns can adopt. | [framework extraction](docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md) |
| `fdu-yov0` — split the files view; `--view all` becomes a named set | 7 | Output-contract follow-ups after the view vocabulary landed. | [view vocabulary](docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md) |
| `fdu-pxeb` — composable CLI and query surface | 5 | Follow-ups after the five-axis surface shipped. | [composable CLI](docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md) |
| `fdu-x746` — post-phase-1 extensions and integrations | 4 | Deliberately deferred scope. | [post-phase-1 roadmap](docs/project/specs/future/plan-2026-08-09-fdu-post-phase-1-roadmap.md) |
| `fdu-j2ka` — iteratively profile and optimize real-world traversal | 3 | The standing optimization loop itself, now directed by campaign 2 rather than by the harness spec. | [campaign 2](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) |
| `fdu-dxee` — harden fdu against the Rust engineering quality audit | 2 | CLI stack hardening in follow-up validation. | [rust engineering quality](docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md) |
| `fdu-ktyl` — spec: composable CLI and query surface | 2 | Spec-side remainder of the surface work. | [composable CLI](docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md) |
| `fdu-ives` — output design system: written layout rules every renderer follows | 1 | The rules a renderer is checked against, rather than per-view convention. | — |
| `fdu-5e17` — salvage the still-valid fixes from PR #4 | 1 | Housekeeping. | — |
| `fdu-j5k6` — complete the performance record and generate the technical report | — | Phase A and Phase B’s harness landed. Phases C and D outstanding; see below. | [performance record](docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md) |

## Plan specs not complete

| Spec | Status | What remains |
| --- | --- | --- |
| [phase-1](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md) | Active | The delivery spec behind `fdu-qfz6`. |
| [end-to-end performance testing](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) | Active, narrowed | Owns the evidence harness — corpus contract, probe modes, comparator adapters, regression governance. No longer owns which experiment runs next; that moved to campaign 2. |
| [rust engineering quality](docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md) | Active | PR #1 merged; CLI stack hardening still in follow-up validation. |
| [fsevents-scoped revalidation](docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md) | Draft, scheduled | Unstarted, but no longer speculative: campaign 2 places its Phase 0 spike in Phase D, because a warm revalidation stats every entry regardless of the snapshot — measured twice — so a journal is the only mechanism that goes under the stat floor. |
| [progressive results](docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md) | Draft | Epic `fdu-wpa0`. |
| [release packaging and Python API polish](docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md) | Partly implemented | Done for the non-publishing release-engineering scope; registry publication remains. |
| [view vocabulary and output contract](docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md) | Draft | Epic `fdu-yov0`: split the files view, and make `--view all` a named set. |
| [experiment loop framework extraction](docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md) | Draft | Epic `fdu-7yx4`: the loop’s contract, statistics, and generated views as a reusable framework. |
| [performance record and report](docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md) | Phases A–B(harness) landed | **Phase B (artifacts)**: promote session-scale findings into artifacts — blocked on a quiet host, not on the harness. **Phase C**: fill the cross-platform matrix. **Phase D**: emit the generated technical report. |
| [performance campaign 2](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) | Active | The current performance strategy: floor-normalized priorities, one structural experiment (H86) as the centerpiece, and per-tier termination criteria. Owns the queue ordering the older research docs used to carry. |
| [experiment evidence scope](docs/project/specs/active/plan-2026-08-23-experiment-evidence-scope.md) | Draft | Arrived with PR #38 and had no row here. Eight open beads: make a measurement’s subject scope an enforced property, so a number taken on one tree stops travelling as a general claim — which the record shows happening three times. |
| [interactive client integration](docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md) | Draft | Epic `fdu-u7vo`: the metabrowser contract. Phase 0 is a measured binding-layer defect — a reader raises while `refresh()` runs — then partitioned tallies, a runtime type registry with a browsing group level, the embedder watch contract, the session shape, and the adoption proof. Consumes the progressive-results session rather than redesigning it; its registry work converts PR #38’s indexed tiers rather than competing with them. |

Two specs in `active/` are finished and are listed in the archive instead: CLI UX and
agent skill, and composable CLI surface.
They stay in `active/` because open follow-up epics still cite them.

[Cache layers and defaults](docs/project/specs/done/plan-2026-08-15-fdu-cache-layers-and-defaults.md)
moved to `done/` on 2026-08-23: all three phases are resolved, and the one bead still
citing it belongs to progressive results.
Its cost model — a snapshot earns its keep when it avoids expensive work, not when it
mirrors a walk that still has to happen — is what campaign 2’s warm posture rests on.

## Notable loose ends outside any epic

Sixty open beads have no parent.
These are the ones a reader of this page should know about:

- `fdu-ow8y` — the inconclusive quiet-host release cell.
  **Blocks every positive peer-comparison claim**, including the current fdu-versus-dust
  and fdu-versus-dumac results, which are ties and decisive-loss respectively on an
  uncontrolled host.
- `fdu-lk9u` — the second blocker on peer claims, and a larger one: the corpus.
  The Linux walker comparison inverts with the subject — fdu leads ripgrep’s `ignore` by
  12–26% on four generated trees and trails by about 12% on `/usr`, the only real tree
  measured. A quiet host is not sufficient if the tree is generated.
- `fdu-f6n7` — narrow the `getattrlistbulk` attribute set to what the plan consumes.
  The registered path to the scalar class: fdu requests ctime, inode, and flags per
  entry for a cache fingerprint the transient summary provably never uses, and measures
  17% more kernel time than `dumac` at identical enumeration counts.
- `fdu-9tul` — the adaptive worker threshold is marginal on macOS, not inert.
  The calibration lands at 20.6–41.0 µs/entry against a 30 µs trigger, so the worker
  count flips with host load: ~45% of aggregate kernel time riding on a decision a
  thread sweep cannot distinguish on wall time.
- `fdu-5yjk` — `FDU_SCAN_DIAGNOSTICS` cannot instrument the FullIndex plan, which is the
  default one users run.
- `fdu-rjqx` / `fdu-tgsx` — the controlled-cold macOS protocol.
  `purge` only approximates boot conditions, so every macOS cold claim is currently
  diagnostic.
- `fdu-tt49` — lint and typecheck `explorations/benchmarks/`. The 8,000-line harness
  that decides accept/reject and validates every artifact is unit-tested but never
  linted.
- `fdu-f8ni` — reserve experiment and hypothesis ids at registration time.
  The duplicate-id half is now enforced by `make perf-ledger`; the reservation
  convention is not.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
