---
type: is
id: is-01m01cm1sb8xyw9ag3pabb5s3h
title: Stabilize adaptive scan scaling on heterogeneous macOS trees
kind: bug
status: open
priority: 1
version: 5
labels:
  - perf
  - macos
dependencies: []
created_at: 2026-08-15T00:19:49.615Z
updated_at: 2026-08-15T00:37:28.455Z
---

## Notes

Observed with installed fdu 0.1.0-dev+ge53f70802.dirty on /Users/levy/Library/Application Support (live, partial tree; about 396,900 entries and 12 macOS TCC permission errors). User sample: dust 2.284 s versus fdu 4.008 s. Total CPU was nearly equal (dust 11.797 s, fdu 11.515 s), but CPU/wall was about 5.17 for dust versus 2.87 for fdu. Fixed-thread and automatic perf_probe runs showed automatic mode is bimodal on the same tree: some runs make the one-shot decision below the 30,000 ns/entry threshold and remain at 6 workers (2.27-2.35 s); others cross it and jump to about 15 effective workers (1.62-1.75 s). Source diagnosis: hardware already bounds the pool (10 available CPUs -> initial 6, maximum 16), but calibration aggregates the first 16,384 successfully completed entries in concurrent completion order, not a representative tree sample. Fast subtrees can finish while slow chunks remain in flight, biasing the sample downward. The measured work time also includes path/batch CPU work rather than isolating filesystem wait. Once the threshold is reached, calibration is discarded permanently on either decision, so later slow regions cannot correct a false negative. Exp-021 explicitly recorded this known failure mode and accepted it only on an immutable 720k homogeneous cache-pressure corpus with a neutral 120k boundary. Proposed next hypothesis: retain hardware-based initial/maximum bounds but replace the one-shot binary threshold with an instrumented, monotonic step controller (6 -> available parallelism -> bounded I/O reserve) evaluated over rolling aggregate throughput, ready-directory backlog, and consumer backlog/backpressure. Add runtime-gated counters for initial/max workers, calibration entries/work/ns-per-entry, decision, scale point, pending/outstanding directories, and macOS portable fallbacks; measure counter overhead first. Do not switch directly to a fixed hardware worker count: prior 60k evidence found 8 workers about 4% worse, and 16 workers on the 720k tree improved wall while increasing aggregate CPU 42-51%. Validate on immutable 60k, 120k, 720k, and a reproducible heterogeneous topology.
