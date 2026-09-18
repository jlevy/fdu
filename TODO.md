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

Twenty-one open, ordered by how many direct children are still open under each.

| Epic | Open | What remains | Spec |
| --- | ---: | --- | --- |
| `fdu-qfz6` — fdu phase 1: fastest walker with full stats, proven by benchmark | 16 | The original delivery epic. Its benchmark gate is the definition of “done” for the walker. | [phase-1](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md) |
| `fdu-0myw` — Linux performance validation and optimization | 12 | Linux is measured but thinly: the ledger’s regime coverage is almost entirely macOS/APFS. | [end-to-end performance](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) |
| `fdu-wpa0` — warm progressive results: lazy open and per-value provenance | 11 | Narrowed to persisted roll-ups, lazy warm open, prefer-cache policy, and honest mixed-source provenance. The opened-root rewrite now owns cold streaming and the live session lifecycle. | [progressive results](docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md) |
| `fdu-d5e1` — reproducible end-to-end performance evidence | 10 | The generated-corpus harness this project’s loop borrows from. | [end-to-end performance](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) |
| `fdu-snej` — implement the opened-root inventory engine rewrite | 9 | The engine merged in PR #48, with lifecycle follow-ups in #56 and contract decisions in #57, and Python has the synchronous `fdu.opened.OpenedIndex`. Measured native indexes and continuations, the revised MetaBrowser provider contract, the thin MetaBrowser backend, cross-provider conformance, composed integration, and final performance acceptance remain. | [opened-root inventory](docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md) |
| `fdu-u7vo` — fdu for interactive clients: the MetaBrowser contract | 9 | The umbrella over `fdu-snej`, which holds the active design. Session integration shape, progress mode, progressive goldens, a two-engine agreement oracle, and handle lifecycle remain outside it. | [opened-root inventory](docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md) |
| `fdu-7yx4` — extract the experiment loop as a reusable framework | 8 | The contract, statistics, generated views, and protocol as something other campaigns can adopt. | [framework extraction](docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md) |
| `fdu-pxeb` — composable CLI and query surface | 5 | Follow-ups after the axis surface shipped. | [composable CLI](docs/project/specs/done/plan-2026-08-10-fdu-composable-cli-surface.md) |
| `fdu-748k` — streaming performance parity without one-shot overhead | 4 | Delivered in PR #52. The parity proof and regression guards (`fdu-lj4h`), final validation (`fdu-rx0d`), and removing ordered path-map work from mutation preflight (`fdu-0q6w`) remain. | [streaming performance parity](docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md) |
| `fdu-xde5` — H86: consumer representation as one structural experiment | 4 | Campaign 2’s centerpiece. The spike measures **1.06× the parallel syscall floor** where the index tier runs 2.68×, and the ~15-point real-tree tax lands in the code it deletes. One experiment, floor-anchored targets, not piecemeal. | [campaign 2](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) |
| `fdu-x746` — post-phase-1 extensions and integrations | 4 | Deliberately deferred scope. | [post-phase-1 roadmap](docs/project/specs/future/plan-2026-08-09-fdu-post-phase-1-roadmap.md) |
| `fdu-2lkf` — control state does not scale to a real home directory | 4 | PR #63 made the `.gitignore` bounds degrade instead of aborting. The slowdown attribution (`fdu-pro1`), a `~/Library` scan killed for memory (`fdu-6o5o`), peak memory against dust (`fdu-syyl`), and snapshots keyed by scan scope (`fdu-w3l5`) remain. | [opened-root inventory](docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md) |
| `fdu-j2ka` — iteratively profile and optimize real-world traversal | 3 | The standing optimization loop itself, now directed by campaign 2 rather than by the harness spec. | [campaign 2](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) |
| `fdu-ktyl` — spec: composable CLI and query surface | 2 | Spec-side remainder of the surface work. | [composable CLI](docs/project/specs/done/plan-2026-08-10-fdu-composable-cli-surface.md) |
| `fdu-gjc2` — release readiness: land the PR stack and cut the first stable release | 1 | The stack has merged. The 0.1.0 CHANGELOG section and release notes (`fdu-qy8e`) remain, and the user cuts the release. | [release process](docs/project/guides/release-process.md) |
| `fdu-yov0` — split the files view; `--view all` becomes `--view full` | 1 | The vocabulary shipped in PR #39: `largest`, `recent`, a complete `files`, `--view full`, stated bounds, and a YAML parse check. Parsing each JSONL report line with a real JSON parser (`fdu-c2ml`) remains. | [view vocabulary](docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md) |
| `fdu-dxee` — harden fdu against the Rust engineering quality audit | 1 | CLI stack hardening landed; exercising snapshot parsing and commit failures as a state machine (`fdu-471a`) remains. | [rust engineering quality](docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md) |
| `fdu-d4kg` — overnight research loop: the macOS agenda for campaign 2 | 1 | The macOS ordering an unattended agent follows. The macOS floor instrument (`fdu-9hdc`) remains. | [campaign 2](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) |
| `fdu-ives` — output design system: written layout rules every renderer follows | 1 | The rules a renderer is checked against, rather than per-view convention. | — |
| `fdu-5e17` — salvage the still-valid fixes from PR #4 | 1 | Housekeeping. | — |
| `fdu-j5k6` — complete the performance record and generate the technical report | 1 | Phase A and Phase B’s harness landed. Phases C and D outstanding; see below. | [performance record](docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md) |

## Plan specs not complete

| Spec | Status | What remains |
| --- | --- | --- |
| [phase-1](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md) | Active | The delivery spec behind `fdu-qfz6`. |
| [end-to-end performance testing](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) | Active, narrowed | Owns the evidence harness — corpus contract, probe modes, comparator adapters, regression governance. No longer owns which experiment runs next; that moved to campaign 2. |
| [rust engineering quality](docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md) | Active | PR #1 merged and CLI stack hardening landed; `fdu-471a` remains. |
| [fsevents-scoped revalidation](docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md) | Draft, scheduled | Unstarted, but no longer speculative: campaign 2 places its Phase 0 spike in Phase D, because a warm revalidation stats every entry regardless of the snapshot — measured twice — so a journal is the only mechanism that goes under the stat floor. |
| [progressive results](docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md) | Active, narrowed | Epic `fdu-wpa0`: warm persisted roll-ups, lazy open, prefer-cache policy, and per-value mixed-source provenance. |
| [opened-root inventory engine rewrite](docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md) | Active; merged in PR #48 | Epics `fdu-snej` and `fdu-2lkf`: measured native indexes and continuations, MetaBrowser adoption and composed acceptance, and control state at home-directory scale. |
| [streaming performance parity](docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md) | Merged in PR #52; acceptance open | Epic `fdu-748k`: the parity proof and regression guards, and final validation. |
| [release packaging and Python API polish](docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md) | Partly implemented | Done for the non-publishing release-engineering scope; registry publication remains. |
| [view vocabulary and output contract](docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md) | Implemented in PR #39 | Epic `fdu-yov0` keeps one open bead, `fdu-c2ml` (a real JSON parser for JSONL report lines); the move to `done/` is `fdu-747k`. |
| [experiment loop framework extraction](docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md) | Draft | Epic `fdu-7yx4`: the loop’s contract, statistics, and generated views as a reusable framework. |
| [experiment evidence scope](docs/project/specs/active/plan-2026-08-23-experiment-evidence-scope.md) | Draft | PR #38 landed the first slice of Phase 1. The rest of Phase 1 (subject profiles, `verdict.scope`, provenance) and all of Phase 2 (the accept rule’s scope clause) remain. |
| [performance record and report](docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md) | Phases A–B(harness) landed | **Phase B (artifacts)**: promote session-scale findings into artifacts — blocked on a quiet host, not on the harness. **Phase C**: fill the cross-platform matrix. **Phase D**: emit the generated technical report. |
| [performance campaign 2](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) | Active | The current performance strategy: floor-normalized priorities, one structural experiment (H86) as the centerpiece, and per-tier termination criteria. Owns the queue ordering the older research docs used to carry. |
| [disk-usage checkpoints](docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md) | Proposed | All five delivery slices, from the replay probe (`fdu-uwhl`) and checkpoint store (`fdu-8ybz`) to the large-home workflow. |

View vocabulary has shipped but stays in `active/`, listed above, while its one bead is
open; moving it to `done/` is `fdu-747k`. CLI UX and agent skill, and composable CLI
surface, moved to `done/` on 2026-09-16 and are listed in the archive.

[Cache layers and defaults](docs/project/specs/done/plan-2026-08-15-fdu-cache-layers-and-defaults.md)
moved to `done/` on 2026-08-23: all three phases are resolved, and the one bead still
citing it belongs to progressive results.
Its cost model — a snapshot earns its keep when it avoids expensive work, not when it
mirrors a walk that still has to happen — is what campaign 2’s warm posture rests on.

## Notable loose ends outside any epic

Of the open beads, 109 have no parent.
These are the ones a reader of this page should know about:

- `fdu-ow8y` — the inconclusive quiet-host release cell.
  **Blocks every positive peer-comparison claim**, including the current fdu-versus-dust
  and fdu-versus-dumac results, which are ties and decisive-loss respectively on an
  uncontrolled host.
- `fdu-f6n7` — narrow the `getattrlistbulk` attribute set to what the plan consumes.
  The registered path to the scalar class: fdu requests ctime, inode, and flags per
  entry for a cache fingerprint the transient summary provably never uses, and measures
  17% more kernel time than `dumac` at identical enumeration counts.
- `fdu-9tul` — the adaptive worker threshold is marginal on macOS, not inert.
  The calibration lands at 20.6–41.0 µs/entry against a 30 µs trigger, so the worker
  count flips with host load: ~45% of aggregate kernel time riding on a decision a
  thread sweep cannot distinguish on wall time.
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
