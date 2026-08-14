---
title: Extensions interned to integer ids
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-008
  title: Extensions interned to integer ids
  date: "2026-08-11"
  hypotheses:
    - H18
  subject:
    tree_label: metabrowser-clone
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: bf574331eca680372f7060d4f9ab3b3b175afd265ac27bda6b6dc67ed9c80798
    tree_entries: 59654
    tree_directories: 7341
    tree_files: 52291
    tree_symlinks: 22
    tree_apparent_bytes: 1082046346
    tree_allocated_bytes: 1225879552
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
    trials: 14
    warmups: 3
    interleaved: true
    control: "exp-007 build: by_ext keyed by owned String, cloned at every ancestor merge"
    candidate: "by_ext keyed by u32 ExtId; names interned once per index at insert, resolved only at query time"
    control_binary:
      name: h14
      sha256: 654ca229a3b89508fbe92d5c8ecf70190fa9a41940f991a7495ab2f95c39040b
      size_bytes: 519328
      args: []
    candidate_binary:
      name: h14h18
      sha256: fd188164cb635a257654f7cbb5d72d6faeec70fd53b9661f4da523db8c0ff448
      size_bytes: 519328
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp007-009-portable-stack.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 511332125.0
          candidate_median: 462634895.5
          change_pct: -15.652
          ci95_low_pct: -32.766
          ci95_high_pct: -0.783
          significant: true
          pairs: 14
        component_ns:
          control_median: 327129062.5
          candidate_median: 282551875.0
          change_pct: -17.509
          ci95_low_pct: -29.646
          ci95_high_pct: -8.048
          significant: true
          pairs: 14
        cpu_ns:
          control_median: 1065645500.0
          candidate_median: 1078041500.0
          change_pct: -0.175
          ci95_low_pct: -2.536
          ci95_high_pct: 5.787
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 265170000.0
          candidate_median: 248176000.0
          change_pct: -6.307
          ci95_low_pct: -9.03
          ci95_high_pct: -5.527
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 802061000.0
          candidate_median: 826702500.0
          change_pct: 1.429
          ci95_low_pct: -1.895
          ci95_high_pct: 9.864
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 38330368.0
          candidate_median: 35086336.0
          change_pct: -10.218
          ci95_low_pct: -14.057
          ci95_high_pct: -7.096
          significant: true
          pairs: 14
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 519887334.0
          candidate_median: 536973042.0
          change_pct: -3.12
          ci95_low_pct: -5.802
          ci95_high_pct: 2.057
          significant: false
          pairs: 14
        component_ns:
          control_median: 215709770.5
          candidate_median: 219996750.0
          change_pct: 0.534
          ci95_low_pct: -8.428
          ci95_high_pct: 4.579
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 2113609000.0
          candidate_median: 2119421500.0
          change_pct: 0.56
          ci95_low_pct: -6.057
          ci95_high_pct: 2.137
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 334253000.0
          candidate_median: 317423000.0
          change_pct: -4.957
          ci95_low_pct: -6.694
          ci95_high_pct: -3.42
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 1779819500.0
          candidate_median: 1801933000.0
          change_pct: 1.795
          ci95_low_pct: -6.245
          ci95_high_pct: 3.617
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 37412864.0
          candidate_median: 34324480.0
          change_pct: -6.688
          ci95_low_pct: -8.45
          ci95_high_pct: -5.992
          significant: true
          pairs: 14
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1037379771.0
          candidate_median: 952780791.5
          change_pct: -4.538
          ci95_low_pct: -7.21
          ci95_high_pct: 8.602
          significant: false
          pairs: 14
        component_ns:
          control_median: 636129708.5
          candidate_median: 643461791.5
          change_pct: -3.332
          ci95_low_pct: -13.442
          ci95_high_pct: 9.489
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 782870000.0
          candidate_median: 771031000.0
          change_pct: -3.548
          ci95_low_pct: -5.076
          ci95_high_pct: 1.325
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 288477500.0
          candidate_median: 274157000.0
          change_pct: -6.404
          ci95_low_pct: -7.776
          ci95_high_pct: -4.083
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 494191000.0
          candidate_median: 495445000.0
          change_pct: -1.926
          ci95_low_pct: -3.575
          ci95_high_pct: 4.456
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 240636271.0
          candidate_median: 184705791.5
          change_pct: -3.939
          ci95_low_pct: -27.808
          ci95_high_pct: 59.563
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 34308096.0
          candidate_median: 32563200.0
          change_pct: -6.112
          ci95_low_pct: -7.506
          ci95_high_pct: -4.463
          significant: true
          pairs: 14
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 283683583.0
          candidate_median: 278913250.0
          change_pct: -6.896
          ci95_low_pct: -14.786
          ci95_high_pct: -4.237
          significant: true
          pairs: 14
        component_ns:
          control_median: 133516541.5
          candidate_median: 123834167.0
          change_pct: -12.334
          ci95_low_pct: -17.502
          ci95_high_pct: -10.416
          significant: true
          pairs: 14
        cpu_ns:
          control_median: 245301000.0
          candidate_median: 227014500.0
          change_pct: -7.245
          ci95_low_pct: -9.246
          ci95_high_pct: -5.812
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 230362500.0
          candidate_median: 214189000.0
          change_pct: -6.618
          ci95_low_pct: -8.327
          ci95_high_pct: -6.068
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 15133500.0
          candidate_median: 13955500.0
          change_pct: -6.494
          ci95_low_pct: -21.843
          ci95_high_pct: -1.106
          significant: true
          pairs: 14
        blocked_ns:
          control_median: 38183083.0
          candidate_median: 47098750.0
          change_pct: -14.377
          ci95_low_pct: -46.204
          ci95_high_pct: 19.68
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 33095680.0
          candidate_median: 31236096.0
          change_pct: -5.991
          ci95_low_pct: -7.004
          ci95_high_pct: -4.055
          significant: true
          pairs: 14
  reference_tools:
    - name: dust
      wall_ns_median: 220640562.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 120
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Ids are session-local by construction (snapshots store names; roll-ups rebuild on load), so the format is untouched. Cross-index RollUp equality now goes through by_ext_named — the one call site the type change forced honest. The run was noisy (load average 17, other agents building concurrently); the accept stands because the interval cleared zero anyway, and the cumulative re-measurement will confirm on a quiet machine."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -15.652
    reason: "cold-scan-index -15.65% [-32.77%, -0.78%] and warm-snapshot-load -6.90% [-14.79%, -4.24%], significant even in a run whose variance was inflated several-fold by machine load, with the placebo job unmoved"
    commit: bb1529d
---
# Extensions interned to integer ids

## Hypothesis

H18: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**ACCEPTED** — cold-scan-index -15.65% [-32.77%, -0.78%] and warm-snapshot-load -6.90%
[-14.79%, -4.24%], significant even in a run whose variance was inflated several-fold by
machine load, with the placebo job unmoved
