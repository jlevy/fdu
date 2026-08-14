---
title: Breadth-first traversal order
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-012
  title: Breadth-first traversal order
  date: "2026-08-11"
  hypotheses:
    - H48
  subject:
    tree_label: metabrowser-clone
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: c631fbf39d7c7adace225d5c9935aaf991176d05da800abd7a69c56ceb0f3b0e
    tree_entries: 60067
    tree_directories: 7350
    tree_files: 52695
    tree_symlinks: 22
    tree_apparent_bytes: 1085083672
    tree_allocated_bytes: 1230073856
    tree_max_depth: 19
    tree_mutated_during_run: false
    host_cpu: Apple M1 Pro
    host_arch: arm64
    host_cores: 10
    host_performance_cores: 8
    host_efficiency_cores: 2
    host_memory_bytes: 34359738368
    host_system: Darwin 25.5.0
    filesystem: apfs
    os_cache: warm-steady
  method:
    trials: 16
    warmups: 4
    interleaved: true
    control: the previous hardcoded LIFO walk
    candidate: "breadth-first, the new default, so partial results are monotone lower bounds"
    control_binary:
      name: depth_first
      sha256: a8b5b81f1b22e11bda07a04153e461c60d01220d24df9a6529007da41c700fec
      size_bytes: 519344
      args:
        - "--order"
        - depth-first
    candidate_binary:
      name: breadth_first
      sha256: a8b5b81f1b22e11bda07a04153e461c60d01220d24df9a6529007da41c700fec
      size_bytes: 519344
      args:
        - "--order"
        - breadth-first
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp012-traversal-order.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 337925083.0
          candidate_median: 337040083.5
          change_pct: -0.58
          ci95_low_pct: -2.502
          ci95_high_pct: 1.195
          significant: false
          pairs: 16
        component_ns:
          control_median: 216259437.5
          candidate_median: 211240313.0
          change_pct: -0.074
          ci95_low_pct: -3.486
          ci95_high_pct: 1.169
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 1178190000.0
          candidate_median: 1205359500.0
          change_pct: 0.492
          ci95_low_pct: -0.709
          ci95_high_pct: 3.487
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 242736500.0
          candidate_median: 245665500.0
          change_pct: 0.489
          ci95_low_pct: -1.435
          ci95_high_pct: 3.124
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 943520000.0
          candidate_median: 955696000.0
          change_pct: 0.681
          ci95_low_pct: -1.116
          ci95_high_pct: 4.302
          significant: false
          pairs: 16
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 34668544.0
          candidate_median: 35373056.0
          change_pct: 1.512
          ci95_low_pct: 0.845
          ci95_high_pct: 2.881
          significant: false
          pairs: 16
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 491046437.5
          candidate_median: 503156687.0
          change_pct: 1.5
          ci95_low_pct: -3.5
          ci95_high_pct: 3.128
          significant: false
          pairs: 16
        component_ns:
          control_median: 199747479.0
          candidate_median: 198001729.5
          change_pct: -1.564
          ci95_low_pct: -3.825
          ci95_high_pct: 1.62
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 2025009000.0
          candidate_median: 2095731500.0
          change_pct: 2.497
          ci95_low_pct: 1.478
          ci95_high_pct: 4.039
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 310684000.0
          candidate_median: 315451000.0
          change_pct: 2.061
          ci95_low_pct: 0.157
          ci95_high_pct: 3.78
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 1720861000.0
          candidate_median: 1778906000.0
          change_pct: 2.319
          ci95_low_pct: 1.258
          ci95_high_pct: 4.135
          significant: false
          pairs: 16
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 34652160.0
          candidate_median: 35774464.0
          change_pct: 3.656
          ci95_low_pct: 2.469
          ci95_high_pct: 4.723
          significant: false
          pairs: 16
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 700830021.0
          candidate_median: 700746291.0
          change_pct: 0.027
          ci95_low_pct: -3.834
          ci95_high_pct: 2.868
          significant: false
          pairs: 16
        component_ns:
          control_median: 471870229.0
          candidate_median: 482795145.5
          change_pct: 1.279
          ci95_low_pct: -1.355
          ci95_high_pct: 7.16
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 672442000.0
          candidate_median: 682560500.0
          change_pct: 2.311
          ci95_low_pct: 0.45
          ci95_high_pct: 5.101
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 253313000.0
          candidate_median: 253584500.0
          change_pct: 0.575
          ci95_low_pct: -0.793
          ci95_high_pct: 1.846
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 417302500.0
          candidate_median: 429174500.0
          change_pct: 3.209
          ci95_low_pct: 0.243
          ci95_high_pct: 6.982
          significant: false
          pairs: 16
        blocked_ns:
          control_median: 25822854.0
          candidate_median: 13686728.5
          change_pct: -36.677
          ci95_low_pct: -60.108
          ci95_high_pct: -4.03
          significant: true
          pairs: 16
        peak_rss_bytes:
          control_median: 32251904.0
          candidate_median: 32710656.0
          change_pct: 1.171
          ci95_low_pct: 0.357
          ci95_high_pct: 3.768
          significant: false
          pairs: 16
  reference_tools: []
  complexity:
    lines_changed: 153
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Accepted on cost, not on speed: the value is that partial results become monotone lower bounds instead of a confidently wrong ranking, and this experiment establishes that the property is cheap - not that it is free. It also corrects an earlier six-sample median comparison that had suggested ~8% and was quoted in the plan before going through the accept rule; sixteen paired trials say the wall-time difference is not measurable. Peak RSS and producer CPU did rise measurably and are recorded above. NOTE: the reference tree was reclaimed mid-session (disk at 100%) and re-cloned, so it is now 60,067 entries against the 59,654 used by exp-000..011; comparisons across that boundary are not valid."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -0.58
    reason: "No detectable wall-time change on a complete scan (-0.58%, interval [-2.50%, +1.20%] - straddles zero, so this is 'not measurably different', not 'free'). Peak RSS rose measurably on all three jobs (+1.51% [+0.85, +2.88] cold-scan-index, +3.66% [+2.47, +4.72] cold-scan-producer, +1.17% [+0.36, +3.77] warm-revalidate), as did producer CPU (+2.50% [+1.48, +4.04]). Accepted on those costs, not on speed: monotone partial results are worth ~1 MiB of frontier and ~2% producer CPU on a 60k tree"
    commit: fbc28f4
---
# Breadth-first traversal order

## Hypothesis

H48: switching the walker’s claim order from LIFO to FIFO will not change wall time on a
complete scan, because the same directories are read either way and only their order
differs — but it will raise peak memory, because a breadth-first frontier holds a whole
level of directories where a depth-first stack holds one path from the root.

The metric to watch is therefore `peak_rss_bytes`, not `wall_ns`. Wall time is the
metric that decides whether the change is affordable; RSS is the one that decides
whether it stays affordable on a tree with a very wide level.

## What was tried

One policy enum threaded through the four places that pop pending work — the serial
scan, the parallel `claim`, revalidate, and reconcile — selecting `pop_front` instead of
`pop_back`. No new data structure: `DirectoryQueue` already held a `VecDeque`, and it
holds directories rather than entries, which is why the frontier cost is bounded by the
number of directories in the widest level rather than by the tree size.

## What the numbers said

**Wall time did not move, and the interval is wide enough that “did not move” is the
whole claim.** `cold-scan-index` is -0.58% with a 95% interval of [-2.50%, +1.20%]. That
interval straddles zero, so it is consistent with a 2.5% win and with a 1.2% loss alike.
It does not license the word “free”.

**Memory moved, and the intervals are clear of zero.** Peak RSS rose +1.51%
[+0.85%, +2.88%] on `cold-scan-index`, +3.66% [+2.47%, +4.72%] on `cold-scan-producer`,
and +1.17% [+0.36%, +3.77%] on `warm-revalidate` — about 34.7 MB to 35.4 MB on the
primary job. Producer CPU rose +2.50% [+1.48%, +4.04%]. These are small, but they are
measured, and the hypothesis predicted the direction.

The surprise was procedural rather than technical.
An earlier six-sample median comparison had suggested breadth-first cost about 8% of
wall time, and that number reached the plan before it went through the accept rule;
sixteen paired trials say the wall-time difference is not measurable at all.
The reporting then made the opposite error in the other direction — the harness rendered
every metric failing the one-sided accept rule as “n.s.”, so these RSS regressions
printed as statistical silence and the first write-up of this experiment called the
change “free” and “unchanged in memory”.
Both readings were wrong, and the second was the more dangerous one, because it
converted a real cost into a claim of no cost.
The harness now reports evidence direction separately from the accept rule.

## Limitations

This is one warm tree of 60,067 entries on one host, and the two conditions that would
make breadth-first genuinely expensive are both absent from it:

- **Frontier width.** A 60k tree has no level wide enough to stress the queue.
  A home folder with a million directories has not been measured for peak queue size,
  and that is where a +1.5% RSS delta could become a different number entirely.
- **Cold I/O locality.** On a warm tree the metadata cache absorbs ordering effects.
  Ordering interacts with readahead and seek locality differently on a genuinely cold
  tree.

Neither is a reason to reject the change; both are reasons not to quote this result as
general. The wide-tree experiment is tracked separately.

## Verdict

**ACCEPTED, on cost rather than on speed.** Wall time is not measurably different; peak
RSS and producer CPU are measurably but slightly worse.
The change buys monotone partial results — every number a lower bound that only grows,
instead of a confident and wrong early ranking — and roughly 1 MiB of frontier plus ~2%
producer CPU on a 60k tree is a fair price for it.
