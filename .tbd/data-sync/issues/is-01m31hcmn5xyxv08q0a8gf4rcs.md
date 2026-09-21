---
type: is
id: is-01m31hcmn5xyxv08q0a8gf4rcs
title: "Linux follow-ups after the Darwin #91/#92 campaign"
kind: epic
status: open
priority: 1
version: 6
labels: []
dependencies: []
parent_id: is-01m31hc3p7wv76jeq5dhgv3bd8
child_order_hints:
  - is-01kzysa79temyc45zjn2v98kpw
  - is-01kzy2qv7fkcwjcn3g8gas7g4m
  - is-01m00ktvaqdprr245qetsacwbr
  - is-01kzy2qt789svdbes8g3656788
created_at: 2026-09-21T08:30:15.204Z
updated_at: 2026-09-21T08:32:10.780Z
---
Parent handoff: fdu-82h4. Recent verification has been macOS-heavy; this is the Linux half.

## Why this matters

The Darwin campaign's measured effects are warm M1 Pro / APFS, exploratory, and recorded as **not additive, not Linux, not a general one-shot speedup**. The CHANGELOG and release notes carry those caveats. PR #94 is the Linux counterpart and has already recorded H139-H143 — **those ids are minted, not reserved.** Any document still saying "H139-H142 reserved, do not mint" is stale; one such local copy was preserved on branch `preserve/linux-campaign-local-2026-09-20` rather than acted on.

A constant tuned in one regime is inherited, not proven, in the others. Record platform, host (bare metal or virtualized), and cache state with every cell; they decide what a result is evidence about.

## Work, in priority order

1. `fdu-78q6` — content sidecar load is the layer-3 warm cost on Linux. Highest value: this is the same subsystem #91 and #92 changed, so Linux evidence here is directly informative about what just landed. Note `fdu-2pct` first — the restore timers do not currently span the whole load, so stage percentages are not comparable with exp-109's baseline.
2. `fdu-tk1b` — cold-regime worker sweep and adaptive-calibration retune.
3. `fdu-c65j` — adopt samply as the cross-platform profiler, closing the Linux and Windows gap.
4. `fdu-cckr` (deferred) — mimalloc as global allocator; a summary plan measured -30% wall on Linux, never reproduced claim-grade.

## Before measuring anything

- Instrument before optimizing; counters are compiled in and off by default, so `FDU_COUNTERS=1` costs a variable rather than a rebuild.
- Profile before changing anything. The ledger has the rejected experiments to prove intuition about where a walker spends its time is unreliable.
- Measure with `make perf-compare` against a real tree, interleaved and paired; record every experiment including failures with `make perf-record`, then republish with `make perf-ledger` and `make perf-report`. `make check` fails if either generated file has drifted, so an unpublished experiment is caught before merge.
- None of this is in `make check`. A timing gate on a shared CI runner measures the runner.

## Also wanted from a Linux host

- Parity artifacts are recorded by CI on Linux per `AGENTS.md`; a genuine Linux host is the right place to confirm them rather than trust a local recording.
- PR #103 (directory filters) is green on all 19 CI checks but has had no full `make check` anywhere. A Linux host without a nested `.claude/worktrees/` checkout can run the gate that macOS currently cannot (`fdu-vjf2`).

## Notes

Same-day update: PR #105 already starts fdu-78q6 (content sidecar load as the layer-3 warm cost on Linux), stacked on cursor/linux-perf-iterate-de1b, and depends on the apply-timer expansion in PR #104. Review those before opening new Linux measurement work on the sidecar path. fdu-tk1b, fdu-c65j and fdu-cckr remain untouched.
