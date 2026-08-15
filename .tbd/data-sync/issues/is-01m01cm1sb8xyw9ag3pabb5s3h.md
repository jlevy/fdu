---
type: is
id: is-01m01cm1sb8xyw9ag3pabb5s3h
title: Stabilize adaptive scan scaling on heterogeneous macOS trees
kind: bug
status: open
priority: 1
version: 8
labels:
  - perf
  - macos
dependencies: []
created_at: 2026-08-15T00:19:49.615Z
updated_at: 2026-08-15T00:45:47.667Z
---

## Notes

Observed with installed fdu 0.1.0-dev+ge53f70802.dirty on /Users/levy/Library/Application Support (live, partial tree; about 396,900 entries and 12 macOS TCC permission errors). User sample: dust 2.284 s versus fdu 4.008 s. Total CPU was nearly equal (dust 11.797 s, fdu 11.515 s), but CPU/wall was about 5.17 for dust versus 2.87 for fdu. Both tools encounter the same protected directories when dust is run with --print-errors; fdu intentionally reports each error and exits 2 for a partial result (or 0 with --allow-partial), so the warnings are not the performance cause.

Direct diagnostics showed automatic mode is bimodal on the same tree: some runs make the one-shot decision below the 30,000 ns/entry threshold and remain at about 6 effective workers (2.27-2.35 s); others cross it and jump to about 15 effective workers (1.62-1.75 s). Four diagnostic auto/fixed-16 pairs had median auto 2.010 s and fixed-16 1.902 s (paired median -6.85% for fixed-16), but this live partial-tree sample is not claim-grade.

Research-loop audit found a concrete validation gap. Exp-015 through exp-021 tuned the one-shot threshold before the macOS getattrlistbulk backend landed, using an immutable 720,805-entry corpus made from twelve clones of one 60k tree plus a 120k boundary. Exp-021 explicitly recorded the failure mode: a noisy first 16k entries can choose the wrong fixed pool for the remainder of a scan. Exp-022 then changed the timing unit from per-entry metadata calls to directory-level bulk work. Exp-025 rechecked fixed 6/8/12/16 after bulk and invoked automatic mode only once; exp-036 ran auto repeatedly on one 1,007,659-entry live workspace, where auto happened to match fixed six. Neither run artifact records the calibration value, decision, activation point, or active-worker history. The published fdu-vs-dust matrix covered one self-contained 901,963-entry, error-free, warm-steady tree. Although the performance plan says one corpus is not a workload model and already defines balanced/wide/deep/mixed/partial recipes, the worker policy was not qualified across those topology families, phase orderings, or partial/error-bearing natural trees. Unit tests pin only the arithmetic fast/slow sides and the deliberate one-shot behavior.

Source diagnosis: hardware already bounds the pool (10 available CPUs -> initial 6, maximum 16). Calibration aggregates the first 16,384 successfully completed entries in concurrent completion order. Slow in-flight chunks are censored while fast subtrees complete, and the measured work includes path/batch CPU work rather than isolated filesystem wait. The calibration is discarded permanently after either result; a later slow phase cannot correct a false negative. A threshold retune or a fixed available-CPU count would overfit: post-bulk exp-025 found fixed 16 regressed wall 19.19%; exp-036 found fixed 10 neutral on wall with +70.12% CPU and fixed 16 +10.65% wall/+110.60% CPU, while the current Application Support diagnostic sometimes favors the deeper pool.

Required next loop:
1. Instrument before changing policy. Record available/initial/max workers, calibration windows and ns/entry, decision and entry ordinal, active/peak workers, pending/outstanding directory work, consumer backpressure, and macOS bulk/fallback directories in probe artifacts. Measure overhead.
2. Add deterministic mixed-phase subjects (fast-prefix/slow-suffix, slow-prefix/fast-suffix, alternating regions) to the existing balanced, wide, deep, mixed-metadata, 60k, 120k, 720k, and near-1M regimes. Keep partial/permission fixtures as correctness and UX diagnostics; claim-grade speed samples remain exact, complete, immutable, paired, and interleaved.
3. Compare the current one-shot control, fixed 6/available/16 diagnostics, and a minimal continuous-window step controller. Hardware supplies lower/upper bounds; runtime evidence chooses within them. Start conservatively, keep observing after a fast window, scale in bounded stages only with enough ready work, and avoid the irreversible 6->16 jump. Do not select an implementation until profiling and the matrix settle it.
4. Pre-register product gates: on each quiet immutable regime, auto decision/outcome must not be bimodal; auto wall must stay within 3% of the best fixed arm with the paired 95% interval; no existing regime may regress by 3%; CPU/RSS/context-switch costs must be reported and rejected when they buy no wall gain. Add actual release-CLI fdu-vs-dust pairs, not probe-only comparisons, and treat a statistically significant >3% fdu loss on any supported representative fixture as a release-blocking investigation.
5. Re-run on a normal interactive-host regime as a separate label because prior fdu/dumac evidence already showed concurrency rankings can reverse under host pressure. Verify the installed binary/hash and single PATH resolution in the release smoke test.
