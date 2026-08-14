---
title: Cumulative effect through adaptive scanning and macOS bulk metadata
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-023
  title: Cumulative effect through adaptive scanning and macOS bulk metadata
  date: "2026-08-12"
  hypotheses:
    - H1
    - H5
    - H10
    - H14
    - H18
    - H32
    - H48
    - H49
    - H31
    - H3
    - H26
  subject:
    tree_label: metabrowser
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: ce5a7430e152412a519ee9f9776c2fec73e59c58fa553aa3e9c2f8c085d26619
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
    trials: 12
    warmups: 3
    interleaved: true
    control: b565882 before the iterative performance work
    candidate: "current code through exp-022: accepted traversal, index, snapshot, BFS scheduling, adaptive-depth, and macOS bulk-metadata changes"
    control_binary:
      name: control
      sha256: 713a7db449084172489d1e4fd3bc1c8b9f40cf3c352eb65f4af505e127b917d4
      size_bytes: 468832
      args: []
    candidate_binary:
      name: candidate
      sha256: 52e0b303402ac0eafa11b06013b731126d81bef482acc962cca3ad9fa2ebc879
      size_bytes: 552576
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp023-cumulative-current.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 625207833.5
          candidate_median: 295511938.0
          change_pct: -53.495
          ci95_low_pct: -55.22
          ci95_high_pct: -52.223
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 506068166.5
          candidate_median: 175306562.5
          change_pct: -65.661
          ci95_low_pct: -67.039
          ci95_high_pct: -64.886
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 617312000.0
          candidate_median: 1068869000.0
          change_pct: 64.871
          ci95_low_pct: 62.599
          ci95_high_pct: 92.642
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 240384000.0
          candidate_median: 228528000.0
          change_pct: -4.218
          ci95_low_pct: -7.596
          ci95_high_pct: -0.528
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 376013000.0
          candidate_median: 839705000.0
          change_pct: 111.314
          ci95_low_pct: 102.849
          ci95_high_pct: 149.982
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 33579008.0
          candidate_median: 35201024.0
          change_pct: 4.657
          ci95_low_pct: 3.404
          ci95_high_pct: 6.494
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 990935937.5
          candidate_median: 412880895.5
          change_pct: -58.199
          ci95_low_pct: -59.051
          ci95_high_pct: -56.462
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 399656979.5
          candidate_median: 172290624.5
          change_pct: -57.435
          ci95_low_pct: -58.19
          ci95_high_pct: -53.901
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 961367000.0
          candidate_median: 1741867000.0
          change_pct: 82.819
          ci95_low_pct: 72.116
          ci95_high_pct: 87.496
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 273107500.0
          candidate_median: 263405000.0
          change_pct: -4.188
          ci95_low_pct: -5.069
          ci95_high_pct: -1.641
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 688886500.0
          candidate_median: 1491114500.0
          change_pct: 118.232
          ci95_low_pct: 100.19
          ci95_high_pct: 123.942
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 33718272.0
          candidate_median: 35504128.0
          change_pct: 4.655
          ci95_low_pct: 3.738
          ci95_high_pct: 6.137
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 645422229.0
          candidate_median: 315360541.5
          change_pct: -51.318
          ci95_low_pct: -52.337
          ci95_high_pct: -50.228
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 25178062.0
          candidate_median: 25551770.5
          change_pct: 1.225
          ci95_low_pct: -1.791
          ci95_high_pct: 6.573
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 628778500.0
          candidate_median: 1114464500.0
          change_pct: 75.098
          ci95_low_pct: 71.502
          ci95_high_pct: 82.901
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 253686500.0
          candidate_median: 247738500.0
          change_pct: -2.305
          ci95_low_pct: -3.835
          ci95_high_pct: -1.39
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 374161500.0
          candidate_median: 861231000.0
          change_pct: 128.114
          ci95_low_pct: 122.569
          ci95_high_pct: 142.692
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 42164224.0
          candidate_median: 43024384.0
          change_pct: 0.924
          ci95_low_pct: 0.014
          ci95_high_pct: 4.678
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 796896854.0
          candidate_median: 631613333.5
          change_pct: -20.6
          ci95_low_pct: -21.124
          ci95_high_pct: -20.215
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 472734500.5
          candidate_median: 423992979.0
          change_pct: -10.035
          ci95_low_pct: -11.318
          ci95_high_pct: -8.94
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 791700500.0
          candidate_median: 626608000.0
          change_pct: -20.978
          ci95_low_pct: -21.24
          ci95_high_pct: -20.337
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 424194500.0
          candidate_median: 245300500.0
          change_pct: -42.143
          ci95_low_pct: -42.566
          ci95_high_pct: -41.505
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 364820000.0
          candidate_median: 379884500.0
          change_pct: 4.32
          ci95_low_pct: 1.095
          ci95_high_pct: 5.101
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 5495062.5
          candidate_median: 5353000.5
          change_pct: -4.604
          ci95_low_pct: -24.485
          ci95_high_pct: 43.528
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 33808384.0
          candidate_median: 32505856.0
          change_pct: -3.985
          ci95_low_pct: -4.201
          ci95_high_pct: -3.675
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 315255083.0
          candidate_median: 201158833.5
          change_pct: -36.077
          ci95_low_pct: -36.394
          ci95_high_pct: -35.779
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 206444833.0
          candidate_median: 94543479.0
          change_pct: -54.011
          ci95_low_pct: -54.582
          ci95_high_pct: -53.839
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 312266000.0
          candidate_median: 198809000.0
          change_pct: -36.212
          ci95_low_pct: -36.483
          ci95_high_pct: -35.917
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 304392500.0
          candidate_median: 192820500.0
          change_pct: -36.573
          ci95_low_pct: -36.868
          ci95_high_pct: -36.255
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 7993500.0
          candidate_median: 6227500.0
          change_pct: -22.957
          ci95_low_pct: -25.205
          ci95_high_pct: -18.806
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 2806563.0
          candidate_median: 2334666.5
          change_pct: -15.004
          ci95_low_pct: -31.305
          ci95_high_pct: -4.871
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 32546816.0
          candidate_median: 30998528.0
          change_pct: -5.086
          ci95_low_pct: -5.714
          ci95_high_pct: -4.034
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: measurement-only cumulative anchor; complexity belongs to the individual accepted experiments
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -53.495
    reason: "Against the original pre-work baseline, current code improved cold-index wall 53.49%, producer wall 58.20%, snapshot-save wall 51.32%, warm revalidation 20.60%, and snapshot load 36.08%; all oracle checks passed"
    commit: null
---
# Cumulative effect through adaptive scanning and macOS bulk metadata

## Hypothesis

The accepted experiments deliberately attacked different layers: bounded production,
path normalization, reconciliation lookup, extension aggregation, snapshot parsing,
breadth-first scheduling, scale-adaptive queue depth, and finally macOS metadata
syscalls. Their percentages cannot be added because each experiment used the preceding
accepted build as its control.
Only a fresh comparison between the current stack and the original pre-work binary can
say what users gained cumulatively.

## What was tried

The exact original `b565882` release probe and the exact current release probe were run
against the same immutable 60,067-entry APFS tree.
The harness interleaved twelve paired trials after three warmups for all five
established jobs, checked every engine digest against the independent oracle, and
fingerprinted the tree before and after the set.
This experiment changes no code; the individual experiment records own the complexity
and tradeoffs of each optimization.

## What the numbers said

Cold indexed scans are 53.49% faster [−55.22%, −52.22%], producer-only scans 58.20%
faster [−59.05%, −56.46%], and a cold scan followed by snapshot save 51.32% faster
[−52.34%, −50.23%]. The snapshot save component itself is neutral; the end-to-end gain
comes from the scan that precedes it.

The warm jobs also retain substantial cumulative improvements even though macOS bulk
metadata is not yet wired into reconciliation: full warm revalidation is 20.60% faster
[−21.12%, −20.22%], and snapshot load is 36.08% faster [−36.39%, −35.78%]. All samples
were valid and the tree stayed unchanged.

Cold total CPU remains higher than the original serial baseline—64.87% for indexed scans
and 82.82% for producer-only scans—because bounded parallelism buys latency by
overlapping metadata operations.
The latest bulk experiment materially paid that cost back relative to the adaptive
parallel control, but did not erase it relative to the serial starting point.
Warm CPU, by contrast, fell 20.98% for revalidation and 36.21% for snapshot load.

## Verdict

**Accepted as the cumulative anchor.** The current implementation is roughly twice as
fast as the pre-work build on every live cold path measured here, while preserving the
independent result oracle.
The resource record also keeps the important qualification: the wall-time win is not a
claim that cold work became cheaper in aggregate CPU.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
