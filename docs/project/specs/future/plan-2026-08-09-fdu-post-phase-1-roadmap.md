# Plan: fdu Post-Phase 1 Extensions and Integrations

**Date:** 2026-08-09

**Status:** Future

## Overview

Keep useful but nonessential extensions out of the Phase 1 critical path.
Three workstreams retain reserved architectural seams and require evidence or a stable
Phase 1 surface that does not exist yet.
Content-tier metrics are complete under the
[file-content metrics plan](../done/plan-2026-08-12-fdu-file-content-metrics.md).

The active [Phase 1 plan](../active/plan-2026-08-08-fdu-phase-1.md) owns the optimized
stat-tier engine, product surfaces, performance proof, and first release.
This document owns only work that begins after its stated activation gate.

## Activation Policy

- A future bead becomes active only when every blocker in the bead graph is closed and
  the Phase 1 evidence supports its premise.
- Future work does not delay the remaining correctness hardening or Phase 1 unless new
  evidence promotes a concrete correctness or security defect.
- An activated workstream receives its own active plan when its open design questions
  are material enough to affect user-visible behavior or persistent formats.
- No dependency, public API, performance claim, or downstream migration is introduced
  merely to reserve optional future work.

## Deferred Workstreams

### Durable Cross-Restart Delta Journal

`fdu-3dtq` adds a sidecar journal after the block snapshot and optimized revalidation
contracts are stable.
It must settle compaction, recovery, durability, and whether `since(clock)` survives
process restart without weakening the rule that reconciliation is the correctness
backstop.

Activation gate: block snapshot `fdu-xihx`, optimized revalidation `fdu-wbis`, and the
first release gate `fdu-9cf0` are complete.

### Completed Content-Tier Metrics

`fdu-3n8c` delivers bounded, opt-in line, source-code, and document metrics with
analyzer-versioned cache identity.
The sparse `ContentIndex` and separate content sidecar preserved the reducer and block
snapshot extension boundaries, so the work did not require those future storage changes.
The completed contract and remaining analyzer ideas are in the
[file-content metrics plan](../done/plan-2026-08-12-fdu-file-content-metrics.md).

### Metabrowser Integration

`fdu-p02b` replaces metabrowser’s Python inventory hot path only after fdu has a tested,
published Rust/Python contract.
The integration plan must choose watcher ownership and define a rollback and parity test
before replacing the existing path.

Activation gate: first publication `fdu-9cf0` is complete.

### Optional io_uring Acceleration

`fdu-ktka` remains an optional Linux accelerator.
It starts only if the complete Phase 1 report shows that synchronous `openat`, `close`,
or `statx` calls remain a dominant bottleneck after the syscall walker and parallel
scheduler are complete.

Activation gate: final performance evidence `fdu-ywu0` contains the profile that
justifies the work and the first release gate `fdu-9cf0` is complete.

## Sequence

These are independent tracks after their activation gates, not one forced serial chain:

| Order | Bead | Work | Activation blockers |
| --- | --- | --- | --- |
| 1 | `fdu-3dtq` | Snapshot plus append-only delta journal | `fdu-xihx`, `fdu-wbis`, `fdu-9cf0` |
| 1 | `fdu-p02b` | Metabrowser integration | `fdu-9cf0` |
| 2 | `fdu-ktka` | Evidence-gated io_uring accelerator | `fdu-ywu0`, `fdu-9cf0` |

The order number groups the earliest plausible activation point.
It does not imply a dependency between workstreams in the same or later group.

## Validation Expectations

- Journal work reuses the snapshot fault-state tests and proves replay, compaction,
  corruption, and crash-boundary behavior.
- Metabrowser integration uses installed artifacts and end-to-end parity over scan,
  refresh, error, identity, and watcher transitions.
- io_uring keeps synchronous per-operation fallbacks and must demonstrate an end-to-end
  improvement rather than only a syscall microbenchmark win.

## Beads

Epic: **fdu-x746** — Post-Phase 1 extensions and integrations.
The epic depends on the first-release gate `fdu-9cf0`, so it cannot enter the active
queue before Phase 1 is complete.

| Bead | Priority | Work | Direct blockers |
| --- | --- | --- | --- |
| `fdu-p02b` | P2 | Replace metabrowser’s Python inventory hot path | `fdu-9cf0` |
| `fdu-3dtq` | P3 | Add a compacting cross-restart delta journal | `fdu-xihx`, `fdu-wbis`, `fdu-9cf0` |
| `fdu-ktka` | P3 | Add io_uring only when Phase 1 profiles justify it | `fdu-ywu0`, `fdu-9cf0` |

The three open beads are children of `fdu-x746`, carry the `future` label, and link back
to this plan. Because parent blockers do not propagate to children in tbd, every child
also names `fdu-9cf0` directly.
None is ready while its activation dependencies remain open.
The former fourth child, `fdu-3n8c`, is complete and links to the done content-metrics
plan.

## References

- [fdu Phase 1 plan](../active/plan-2026-08-08-fdu-phase-1.md)
- [End-to-end performance plan](../active/plan-2026-08-09-fdu-end-to-end-performance-testing.md)
- [Rust engineering quality plan](../active/plan-2026-08-09-fdu-rust-engineering-quality.md)
- [File-content metrics plan](../done/plan-2026-08-12-fdu-file-content-metrics.md)
- [File-content metrics implementation report](../../reports/report-2026-08-13-file-content-metrics-implementation.md)
- [File roll-up engine research](../../research/research-2026-08-06-file-rollup-engine.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
