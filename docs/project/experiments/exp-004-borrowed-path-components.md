---
title: Borrowed path components
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-004
  title: Borrowed path components
  date: "2026-08-11"
  hypotheses:
    - H5
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
    control: "HEAD a0cc981, components copied into owned OsStrings twice per operation"
    candidate: "components borrowed from the caller's path; validation allocates nothing"
    control_binary:
      name: control
      sha256: 80e6049b1259dc6c7a6438a6c684a668c096fdd298d9aaf6bd1417ae6c0b820c
      size_bytes: 519328
      args: []
    candidate_binary:
      name: candidate
      sha256: 599ed76f10cccb36935cdeef06baea9971615a7976558c0c68cca37cef804dcc
      size_bytes: 519328
      args: []
    toolchain: ""
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp004-borrowed-path-components.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 318251812.5
          candidate_median: 312819000.0
          change_pct: -1.137
          ci95_low_pct: -5.331
          ci95_high_pct: 2.266
          significant: false
          pairs: 14
        component_ns:
          control_median: 198220542.0
          candidate_median: 196947041.5
          change_pct: -0.497
          ci95_low_pct: -6.371
          ci95_high_pct: 4.248
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 1263677500.0
          candidate_median: 1214263000.0
          change_pct: -4.064
          ci95_low_pct: -8.981
          ci95_high_pct: 2.278
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 285559500.0
          candidate_median: 250866500.0
          change_pct: -12.536
          ci95_low_pct: -13.654
          ci95_high_pct: -11.216
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 972388500.0
          candidate_median: 960834500.0
          change_pct: -1.267
          ci95_low_pct: -7.323
          ci95_high_pct: 6.322
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 36790272.0
          candidate_median: 36331520.0
          change_pct: -1.815
          ci95_low_pct: -2.685
          ci95_high_pct: -0.896
          significant: true
          pairs: 14
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 353364791.5
          candidate_median: 369835062.5
          change_pct: -0.731
          ci95_low_pct: -9.198
          ci95_high_pct: 7.593
          significant: false
          pairs: 14
        component_ns:
          control_median: 29663770.5
          candidate_median: 30280541.5
          change_pct: 2.514
          ci95_low_pct: -3.537
          ci95_high_pct: 10.194
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 1296292500.0
          candidate_median: 1203788000.0
          change_pct: -8.103
          ci95_low_pct: -9.921
          ci95_high_pct: 1.76
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 308929000.0
          candidate_median: 275755000.0
          change_pct: -11.622
          ci95_low_pct: -12.142
          ci95_high_pct: -9.653
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 991853000.0
          candidate_median: 938464000.0
          change_pct: -6.924
          ci95_low_pct: -9.282
          ci95_high_pct: 6.317
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 45686784.0
          candidate_median: 44916736.0
          change_pct: -2.224
          ci95_low_pct: -2.888
          ci95_high_pct: 0.086
          significant: false
          pairs: 14
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 817707479.0
          candidate_median: 764745375.5
          change_pct: -9.4
          ci95_low_pct: -10.639
          ci95_high_pct: -4.867
          significant: true
          pairs: 14
        component_ns:
          control_median: 487284021.0
          candidate_median: 479216604.0
          change_pct: -4.557
          ci95_low_pct: -7.253
          ci95_high_pct: 3.263
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 810489000.0
          candidate_median: 747297000.0
          change_pct: -10.01
          ci95_low_pct: -11.065
          ci95_high_pct: -8.695
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 431864500.0
          candidate_median: 356068500.0
          change_pct: -18.556
          ci95_low_pct: -18.859
          ci95_high_pct: -17.942
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 380346000.0
          candidate_median: 390331000.0
          change_pct: 0.044
          ci95_low_pct: -2.145
          ci95_high_pct: 2.369
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 7826000.0
          candidate_median: 16205604.5
          change_pct: 50.131
          ci95_low_pct: 4.937
          ci95_high_pct: 129.747
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 34086912.0
          candidate_median: 34627584.0
          change_pct: 0.913
          ci95_low_pct: -0.218
          ci95_high_pct: 1.635
          significant: false
          pairs: 14
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 591348812.5
          candidate_median: 556162583.5
          change_pct: -17.804
          ci95_low_pct: -22.486
          ci95_high_pct: -15.562
          significant: true
          pairs: 14
        component_ns:
          control_median: 355697604.5
          candidate_median: 309833041.0
          change_pct: -25.907
          ci95_low_pct: -34.044
          ci95_high_pct: -23.313
          significant: true
          pairs: 14
        cpu_ns:
          control_median: 398295500.0
          candidate_median: 340600000.0
          change_pct: -18.124
          ci95_low_pct: -22.593
          ci95_high_pct: -16.938
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 365092500.0
          candidate_median: 305023000.0
          change_pct: -18.21
          ci95_low_pct: -19.194
          ci95_high_pct: -17.352
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 30267000.0
          candidate_median: 31278000.0
          change_pct: -11.024
          ci95_low_pct: -16.274
          ci95_high_pct: -5.151
          significant: true
          pairs: 14
        blocked_ns:
          control_median: 193053312.5
          candidate_median: 215562583.5
          change_pct: -20.446
          ci95_low_pct: -32.538
          ci95_high_pct: 16.56
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 32555008.0
          candidate_median: 32350208.0
          change_pct: -0.554
          ci95_low_pct: -1.255
          ci95_high_pct: -0.252
          significant: true
          pairs: 14
  reference_tools:
    - name: dust
      wall_ns_median: 209730187.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 38
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "A net simplification: one function returns borrowed slices, and one new predicate replaces an allocating check. The cold path did not move, which is the right result and a check on the story — after exp-001 it is bound by open and stat, so consumer work cannot show up there."
  verdict:
    decision: accepted
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -9.4
    reason: "Warm revalidation 9.4% faster and snapshot load 17.8% faster, both with intervals entirely below zero, by deleting work rather than adding machinery"
    commit: bf7a05a
---
# Borrowed path components

## Hypothesis

H5: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**ACCEPTED** — Warm revalidation 9.4% faster and snapshot load 17.8% faster, both with
intervals entirely below zero, by deleting work rather than adding machinery
