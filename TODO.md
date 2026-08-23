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

Thirteen open, ordered by how much is outstanding under each.

| Epic | Open | What remains | Spec |
| --- | ---: | --- | --- |
| `fdu-qfz6` — fdu phase 1: fastest walker with full stats, proven by benchmark | 15 | The original delivery epic. Its benchmark gate is the definition of “done” for the walker. | [phase-1](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md) |
| `fdu-0myw` — Linux performance validation and optimization | 13 | Linux is measured but thinly: the ledger’s regime coverage is almost entirely macOS/APFS. | [end-to-end performance](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) |
| `fdu-wpa0` — progressive results: order, provenance, sessions, lazy open | 11 | Values labelled per-value with provenance, so an approximate answer can be shown and then converge. Blocks the prefer-cache tier (`fdu-wu6w`). | [progressive results](docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md) |
| `fdu-d5e1` — reproducible end-to-end performance evidence | 8 | The generated-corpus harness this project’s loop borrows from. | [end-to-end performance](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) |
| `fdu-xde5` — H86: consumer representation as one structural experiment | 5 | The largest measured headroom in the record: an oracle-checked spike does the tree view’s work in ~199 ms where the CLI spends ~849 ms on the 450k Linux subject. Must be measured as one experiment, not piecemeal — intermediate forms pay conversion costs the end state deletes. | [structural headroom](docs/project/research/research-2026-08-15-consumer-structural-headroom.md) |
| `fdu-pxeb` — composable CLI and query surface | 5 | Follow-ups after the five-axis surface shipped. | [composable CLI](docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md) |
| `fdu-x746` — post-phase-1 extensions and integrations | 4 | Deliberately deferred scope. | [post-phase-1 roadmap](docs/project/specs/future/plan-2026-08-09-fdu-post-phase-1-roadmap.md) |
| `fdu-j2ka` — iteratively profile and optimize real-world traversal | 3 | The standing optimization loop itself. | [end-to-end performance](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) |
| `fdu-dxee` — harden fdu against the Rust engineering quality audit | 2 | CLI stack hardening in follow-up validation. | [rust engineering quality](docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md) |
| `fdu-ktyl` — spec: composable CLI and query surface | 2 | Spec-side remainder of the surface work. | [composable CLI](docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md) |
| `fdu-5bne` — cache layers and defaults | 1 | Phases 1–3 landed; Phase 2 closed by measurement rather than implemented. The remainder is `fdu-wu6w`, a prefer-cache tier for progressive UIs, which is gated on `fdu-wpa0`. | [cache layers](docs/project/specs/active/plan-2026-08-15-fdu-cache-layers-and-defaults.md) |
| `fdu-5e17` — salvage the still-valid fixes from PR #4 | 1 | Housekeeping. | — |
| `fdu-j5k6` — complete the performance record and generate the technical report | — | Phase A and Phase B’s harness landed. Phases C and D outstanding; see below. | [performance record](docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md) |

## Plan specs not complete

| Spec | Status | What remains |
| --- | --- | --- |
| [phase-1](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md) | Active | The delivery spec behind `fdu-qfz6`. |
| [end-to-end performance testing](docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md) | Active | Owns the generated-corpus evidence harness; three epics point at it. |
| [rust engineering quality](docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md) | Active | PR #1 merged; CLI stack hardening still in follow-up validation. |
| [fsevents-scoped revalidation](docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md) | Draft | Unstarted. Would let revalidation be narrowed to changed subtrees, which is the one thing that could invert the cache-read cost model — see the cache-layers open question. |
| [progressive results](docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md) | Draft | Epic `fdu-wpa0`. |
| [release packaging and Python API polish](docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md) | Partly implemented | Done for the non-publishing release-engineering scope; registry publication remains. |
| [cache layers and defaults](docs/project/specs/active/plan-2026-08-15-fdu-cache-layers-and-defaults.md) | Phases 1–3 landed | Only `fdu-wu6w` remains, and it is gated on progressive results. |
| [performance record and report](docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md) | Phases A–B(harness) landed | **Phase B (artifacts)**: promote session-scale findings into artifacts — blocked on a quiet host, not on the harness. **Phase C**: fill the cross-platform matrix. **Phase D**: emit the generated technical report. |

Two specs in `active/` are finished and are listed in the archive instead: CLI UX and
agent skill, and composable CLI surface.
They stay in `active/` because open follow-up epics still cite them.

## Notable loose ends outside any epic

Sixty open beads have no parent.
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
- `fdu-5yjk` — `FDU_SCAN_DIAGNOSTICS` cannot instrument the FullIndex plan, which is the
  default one users run.
- `fdu-rjqx` / `fdu-tgsx` — the controlled-cold macOS protocol.
  `purge` only approximates boot conditions, so every macOS cold claim is currently
  diagnostic.
- `fdu-tt49` — lint and typecheck `benchmarks/`. The 8,000-line harness that decides
  accept/reject and validates every artifact is unit-tested but never linted.
- `fdu-f8ni` — reserve experiment and hypothesis ids at registration time.
  The duplicate-id half is now enforced by `make perf-ledger`; the reservation
  convention is not.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
