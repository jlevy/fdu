---
title: Snapshot load resolves through the parent
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-005
  title: Snapshot load resolves through the parent
  date: "2026-08-11"
  hypotheses:
    - H10
  checkpoint:
    profile: index-core-v1
    kept_variant: candidate
    source_revision: 954d27be986ef7a0862036efc8bbf2b8b11b7ea1
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
    control: "HEAD bf7a05a, each record's path rebuilt and resolved from the root"
    candidate: "parent path memoized, child id read from the parent's children, duplicate check folded into the insert"
    control_binary:
      name: control
      sha256: 599ed76f10cccb36935cdeef06baea9971615a7976558c0c68cca37cef804dcc
      size_bytes: 519328
      args: []
    candidate_binary:
      name: candidate
      sha256: 43a2a043da5e20b1ae5cd6c8cf80acdbbf728f78b75174d412c88806c88e688a
      size_bytes: 519328
      args: []
    toolchain: ""
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp005-snapshot-load.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 424306958.0
          candidate_median: 436121958.5
          change_pct: -3.851
          ci95_low_pct: -10.869
          ci95_high_pct: 5.099
          significant: false
          pairs: 14
        component_ns:
          control_median: 250343896.0
          candidate_median: 239519667.0
          change_pct: -3.965
          ci95_low_pct: -12.066
          ci95_high_pct: 2.301
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 1078294500.0
          candidate_median: 1062209500.0
          change_pct: 3.474
          ci95_low_pct: -3.827
          ci95_high_pct: 4.774
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 257232500.0
          candidate_median: 259668000.0
          change_pct: 0.868
          ci95_low_pct: -0.117
          ci95_high_pct: 2.319
          significant: false
          pairs: 14
        system_cpu_ns:
          control_median: 821062000.0
          candidate_median: 802447500.0
          change_pct: 3.313
          ci95_low_pct: -5.325
          ci95_high_pct: 6.308
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 37699584.0
          candidate_median: 37134336.0
          change_pct: -0.708
          ci95_low_pct: -4.797
          ci95_high_pct: 1.374
          significant: false
          pairs: 14
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1060451208.5
          candidate_median: 1138260041.5
          change_pct: -1.644
          ci95_low_pct: -8.88
          ci95_high_pct: 13.741
          significant: false
          pairs: 14
        component_ns:
          control_median: 684334833.5
          candidate_median: 745338000.0
          change_pct: 4.237
          ci95_low_pct: -5.832
          ci95_high_pct: 24.25
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 858873000.0
          candidate_median: 817483500.0
          change_pct: -5.979
          ci95_low_pct: -7.297
          ci95_high_pct: -1.498
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 387661000.0
          candidate_median: 343705500.0
          change_pct: -12.055
          ci95_low_pct: -13.677
          ci95_high_pct: -10.516
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 469900500.0
          candidate_median: 473215500.0
          change_pct: -0.687
          ci95_low_pct: -1.987
          ci95_high_pct: 5.28
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 200814708.5
          candidate_median: 256735604.5
          change_pct: -5.53
          ci95_low_pct: -37.673
          ci95_high_pct: 70.518
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 33865728.0
          candidate_median: 33906688.0
          change_pct: 0.383
          ci95_low_pct: -0.944
          ci95_high_pct: 2.021
          significant: false
          pairs: 14
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 386325666.5
          candidate_median: 323610937.5
          change_pct: -18.597
          ci95_low_pct: -23.475
          ci95_high_pct: -12.516
          significant: true
          pairs: 14
        component_ns:
          control_median: 235639208.5
          candidate_median: 164680688.0
          change_pct: -31.081
          ci95_low_pct: -34.767
          ci95_high_pct: -25.335
          significant: true
          pairs: 14
        cpu_ns:
          control_median: 309776500.0
          candidate_median: 256432000.0
          change_pct: -17.164
          ci95_low_pct: -18.32
          ci95_high_pct: -16.404
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 290582000.0
          candidate_median: 238612000.0
          change_pct: -17.86
          ci95_low_pct: -19.244
          ci95_high_pct: -17.239
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 19295500.0
          candidate_median: 17820000.0
          change_pct: -7.099
          ci95_low_pct: -12.045
          ci95_high_pct: -3.387
          significant: true
          pairs: 14
        blocked_ns:
          control_median: 74106041.5
          candidate_median: 65401437.5
          change_pct: -25.204
          ci95_low_pct: -37.634
          ci95_high_pct: 0.43
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 32538624.0
          candidate_median: 32645120.0
          change_pct: 0.201
          ci95_low_pct: -0.849
          ci95_high_pct: 1.563
          significant: false
          pairs: 14
  reference_tools:
    - name: dust
      wall_ns_median: 327408021.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 34
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "The machine was busier during this run than the previous one: every absolute number moved by roughly 30%, including the third-party reference tool's. Paired interleaving is what keeps the comparison usable, and the jobs that should not have moved correctly reported no change with intervals wide enough to say so."
  verdict:
    decision: accepted
    primary_job: warm-snapshot-load
    primary_metric: wall_ns
    change_pct: -18.597
    reason: "Snapshot load 18.6% faster with an interval entirely below zero, by using the parent id the format already provided instead of four ancestor walks per entry"
    commit: 954d27b
---
# Snapshot load resolves through the parent

## Hypothesis

H10: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**ACCEPTED** — Snapshot load 18.6% faster with an interval entirely below zero, by using
the parent id the format already provided instead of four ancestor walks per entry
