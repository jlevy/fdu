---
title: Per-layer counters cost less than the measurement can see
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-052
  title: Per-layer counters cost less than the measurement can see
  date: "2026-08-14"
  hypotheses: []
  subject:
    tree_label: meta450k
    tree_root_id: 49fd7bea88ee0b90566f01fb140e86707e2dd6979dd01af97f30cdb13c535aee
    tree_engine_digest: 5b894066affe6de8c7f0ba44d00eccd9bc3b7e5d0cf146786f3ae860f4b0c3bc
    tree_entries: 450463
    tree_directories: 28630
    tree_files: 421690
    tree_symlinks: 143
    tree_apparent_bytes: 3000524491
    tree_allocated_bytes: 747966464
    tree_max_depth: 20
    tree_mutated_during_run: false
    host_cpu: Linux
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 0
    host_system: Linux 6.18.5-fc-v20
    filesystem: ""
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: perf probe without perf-counters
    candidate: same probe with perf-counters and a counting global allocator
    control_binary:
      name: control
      sha256: a49b22e0a10dba1a50bdb5b19a1e982b73595211be6698953d084329fd5d179d
      size_bytes: 1444752
      args: []
    candidate_binary:
      name: candidate
      sha256: 02d6451f118bac836e4ab6ed45996ff06d51f9b51b815220a484582364c53c96
      size_bytes: 1472336
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: /tmp/fdu-perf/results/run-counter-overhead.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1891335181.5
          candidate_median: 1870126871.0
          change_pct: 0.026
          ci95_low_pct: -3.306
          ci95_high_pct: 3.764
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        component_ns:
          control_median: 787336473.0
          candidate_median: 779342953.0
          change_pct: 0.739
          ci95_low_pct: -4.607
          ci95_high_pct: 5.53
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        cpu_ns:
          control_median: 2826085500.0
          candidate_median: 2862239000.0
          change_pct: 0.175
          ci95_low_pct: -1.456
          ci95_high_pct: 2.554
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        user_cpu_ns:
          control_median: 1901062000.0
          candidate_median: 1902414000.0
          change_pct: -0.832
          ci95_low_pct: -1.62
          ci95_high_pct: 3.685
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        system_cpu_ns:
          control_median: 931127500.0
          candidate_median: 944207500.0
          change_pct: 0.546
          ci95_low_pct: -1.909
          ci95_high_pct: 4.094
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        peak_rss_bytes:
          control_median: 284760064.0
          candidate_median: 287041536.0
          change_pct: -0.53
          ci95_low_pct: -5.062
          ci95_high_pct: 5.442
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1728655277.5
          candidate_median: 1712330259.0
          change_pct: -1.057
          ci95_low_pct: -1.963
          ci95_high_pct: 0.312
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        component_ns:
          control_median: 329211080.5
          candidate_median: 332608010.0
          change_pct: 0.577
          ci95_low_pct: -3.058
          ci95_high_pct: 4.795
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        cpu_ns:
          control_median: 2604687000.0
          candidate_median: 2631196000.0
          change_pct: 0.034
          ci95_low_pct: -1.454
          ci95_high_pct: 2.218
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        user_cpu_ns:
          control_median: 1693671500.0
          candidate_median: 1659413500.0
          change_pct: -1.497
          ci95_low_pct: -4.133
          ci95_high_pct: 2.013
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        system_cpu_ns:
          control_median: 925671000.0
          candidate_median: 953916500.0
          change_pct: 3.319
          ci95_low_pct: -2.378
          ci95_high_pct: 7.009
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        peak_rss_bytes:
          control_median: 199471104.0
          candidate_median: 199401472.0
          change_pct: -0.015
          ci95_low_pct: -0.035
          ci95_high_pct: 0.014
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
  reference_tools: []
  complexity:
    lines_changed: 420
    new_dependencies:
      - "none"
    new_unsafe_blocks: 1
    new_failure_modes: []
    notes: "One unsafe impl GlobalAlloc, kept in the probe rather than the library so the engine's unsafe-free guarantee stands; it forwards every argument unchanged and only adds to a Copy struct, which cannot unwind or re-enter the allocator."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 0.026
    reason: "Overhead is below the noise floor on both jobs: +0.03% [-3.31%, +3.76%] cold and -1.06% [-1.96%, +0.31%] warm, with roughly 13 million counter increments per cold run. Bounds the cost below about 3.3% rather than establishing zero."
    commit: null
---
## What was measured

The cost of leaving the per-layer counters on, which is the only thing that decides
whether they can be trusted.
Instrumentation that distorts what it measures is worse than none, because it is
believed.

The candidate is the probe built with `--features perf-counters`, which turns on
thread-local tallies at every syscall, index and allocation site *and* installs a
counting global allocator wrapping `System`. The control is the same probe with the
feature off, where the module is absent rather than silenced.
A 450k-entry cold scan performs roughly 13 million counter increments across those
sites, so this is close to the worst case the design allows.

| Job | Median | 95% CI | Verdict |
| --- | ---: | --- | --- |
| `cold-scan-index` | +0.03% | [−3.31%, +3.76%] | no detectable cost |
| `warm-revalidate` | −1.06% | [−1.96%, +0.31%] | no detectable cost |

Both intervals span zero.
Stated precisely, this bounds the overhead **below about 3.3%** at twenty trials; it
does not establish that the cost is exactly zero, and a tighter bound would need more
trials than the question is worth.

## Why it is this cheap

Three choices, in descending order of how much they matter:

1. **Thread-local, non-atomic counters.** Incrementing touches one thread’s own cache
   line. A shared atomic in the walk would have measured the counter rather than the
   walk, and the walk is the parallel part.
2. **Counts, not timers, on per-entry paths.** A clock read costs an order of magnitude
   more than a `u64` add, so per-entry work is counted and only per-phase spans are
   timed — the same amortization rule `WalkAttribution` already follows.
3. **Allocation counting rides on an expensive operation.** An allocation costs tens of
   nanoseconds; counting it costs one or two.
   The ratio is what makes always-on defensible here and would not hold for counting
   path components.

## What it immediately showed

A 450k-entry cold scan, per entry:

| Quantity | Per entry |
| --- | ---: |
| Allocations | **15.4** |
| Reallocations | **11.0** |
| Bytes allocated | 2,456 |
| Roll-up merges | 11.9 |
| Parent memo hit rate | **93.6%** |

Three of these change what is worth doing next.

**The memo hit rate confirms `exp-051` for free.** That experiment needed a callgrind
run to show an 89% drop in `normalize` instructions.
The counter says 93.6% directly, on an ordinary build, in the time the scan already
takes.

**Allocation is producer-side, not index-side.** `scan-producer`, which walks without
building an index, allocates *more* than `scan-index` does — 8.8M against 6.9M. The two
jobs differ in what they retain, so this is a direction rather than a clean subtraction,
but it points away from the consumer.
The walk allocates an `OsString` for each name, a `PathBuf` for each joined path, and
clones that `PathBuf` into the op, which is at least three per entry before the batch
vector.

**Eleven reallocations per entry is unexplained.** It tracks roll-up merges almost
one-to-one, which suggested `InternedRollUp::merge`, but `by_ext` is a `BTreeMap` and
those allocate nodes rather than reallocating.
The counters localize the cost without attributing it, which is the honest limit of a
counter that does not sample stacks.
Filed rather than guessed at.

## What this does not do

It does not sample stacks, attribute allocations to call sites, or track live bytes.
Each needs either a profiler or a shadow map, and both cost enough to change what they
measure. Counts and totals are what a paired A/B needs — they answer whether a change
moved allocation *volume*, which is the question H51, H62, H74 and H85 all turn on.

The `perf-counters` build feature described here is the historical experiment control.
`exp-053` replaced it with the current `FDU_COUNTERS=1` runtime toggle and removed the
feature after measuring both its idle and recording costs.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
