---
title: Cumulative effect of every accepted change
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-006
  title: Cumulative effect of every accepted change
  date: "2026-08-11"
  hypotheses:
    - H1
    - H5
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
    trials: 16
    warmups: 4
    interleaved: true
    control: "main @ b565882, before any of this work"
    candidate: "HEAD 954d27b: exp-001, exp-004 and exp-005 together"
    control_binary:
      name: baseline
      sha256: dceca2d5de68e4a063f3afbedb53d27940e14462e9ce49a62e8a319454eea387
      size_bytes: 468832
      args: []
    candidate_binary:
      name: optimized
      sha256: 43a2a043da5e20b1ae5cd6c8cf80acdbbf728f78b75174d412c88806c88e688a
      size_bytes: 519328
      args: []
    toolchain: ""
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp006-cumulative.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 630688291.5
          candidate_median: 320869395.5
          change_pct: -48.909
          ci95_low_pct: -51.135
          ci95_high_pct: -47.516
          significant: true
          pairs: 16
        component_ns:
          control_median: 507400021.0
          candidate_median: 202122000.0
          change_pct: -61.113
          ci95_low_pct: -61.902
          ci95_high_pct: -59.293
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 612460500.0
          candidate_median: 1266600500.0
          change_pct: 104.134
          ci95_low_pct: 100.929
          ci95_high_pct: 108.559
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 231736500.0
          candidate_median: 253196000.0
          change_pct: 8.809
          ci95_low_pct: 7.82
          ci95_high_pct: 10.427
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 379903000.0
          candidate_median: 1006397000.0
          change_pct: 161.31
          ci95_low_pct: 157.209
          ci95_high_pct: 166.314
          significant: false
          pairs: 16
        peak_rss_bytes:
          control_median: 33406976.0
          candidate_median: 36519936.0
          change_pct: 9.211
          ci95_low_pct: 8.848
          ci95_high_pct: 9.559
          significant: false
          pairs: 16
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 958829125.0
          candidate_median: 464549520.5
          change_pct: -51.631
          ci95_low_pct: -52.298
          ci95_high_pct: -50.193
          significant: true
          pairs: 16
        component_ns:
          control_median: 396697874.5
          candidate_median: 187262375.5
          change_pct: -53.379
          ci95_low_pct: -54.936
          ci95_high_pct: -50.773
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 944144500.0
          candidate_median: 2084853000.0
          change_pct: 122.599
          ci95_low_pct: 113.206
          ci95_high_pct: 127.796
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 262278000.0
          candidate_median: 321830000.0
          change_pct: 22.768
          ci95_low_pct: 21.076
          ci95_high_pct: 25.539
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 682232500.0
          candidate_median: 1768911500.0
          change_pct: 161.822
          ci95_low_pct: 147.594
          ci95_high_pct: 168.583
          significant: false
          pairs: 16
        peak_rss_bytes:
          control_median: 33464320.0
          candidate_median: 36503552.0
          change_pct: 9.227
          ci95_low_pct: 8.528
          ci95_high_pct: 9.456
          significant: false
          pairs: 16
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 647260666.5
          candidate_median: 351505417.0
          change_pct: -46.163
          ci95_low_pct: -48.638
          ci95_high_pct: -45.064
          significant: true
          pairs: 16
        component_ns:
          control_median: 41286312.5
          candidate_median: 39641979.5
          change_pct: -8.182
          ci95_low_pct: -16.522
          ci95_high_pct: 6.567
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 627680000.0
          candidate_median: 1255873000.0
          change_pct: 96.58
          ci95_low_pct: 93.165
          ci95_high_pct: 107.136
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 244842500.0
          candidate_median: 262469500.0
          change_pct: 7.827
          ci95_low_pct: 6.493
          ci95_high_pct: 9.842
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 383862500.0
          candidate_median: 987664500.0
          change_pct: 152.304
          ci95_low_pct: 145.793
          ci95_high_pct: 171.891
          significant: false
          pairs: 16
        peak_rss_bytes:
          control_median: 41918464.0
          candidate_median: 44564480.0
          change_pct: 6.464
          ci95_low_pct: 5.463
          ci95_high_pct: 7.686
          significant: false
          pairs: 16
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 804105937.5
          candidate_median: 687997500.0
          change_pct: -14.726
          ci95_low_pct: -15.766
          ci95_high_pct: -12.67
          significant: true
          pairs: 16
        component_ns:
          control_median: 482402666.5
          candidate_median: 461987312.5
          change_pct: -4.391
          ci95_low_pct: -6.501
          ci95_high_pct: -0.606
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 795556000.0
          candidate_median: 681054500.0
          change_pct: -14.727
          ci95_low_pct: -15.623
          ci95_high_pct: -13.588
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 417707000.0
          candidate_median: 304768500.0
          change_pct: -27.092
          ci95_low_pct: -27.506
          ci95_high_pct: -26.815
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 378679000.0
          candidate_median: 375925000.0
          change_pct: -0.954
          ci95_low_pct: -3.133
          ci95_high_pct: 1.446
          significant: false
          pairs: 16
        blocked_ns:
          control_median: 9176896.0
          candidate_median: 6937104.5
          change_pct: -4.262
          ci95_low_pct: -35.991
          ci95_high_pct: 87.578
          significant: false
          pairs: 16
        peak_rss_bytes:
          control_median: 34365440.0
          candidate_median: 33964032.0
          change_pct: -0.314
          ci95_low_pct: -2.08
          ci95_high_pct: 0.425
          significant: false
          pairs: 16
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 324284896.0
          candidate_median: 229665604.0
          change_pct: -29.106
          ci95_low_pct: -29.589
          ci95_high_pct: -28.054
          significant: true
          pairs: 16
        component_ns:
          control_median: 213129187.5
          candidate_median: 117926667.0
          change_pct: -44.918
          ci95_low_pct: -45.494
          ci95_high_pct: -44.067
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 319829000.0
          candidate_median: 226431500.0
          change_pct: -29.351
          ci95_low_pct: -29.893
          ci95_high_pct: -28.77
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 307902500.0
          candidate_median: 215859500.0
          change_pct: -30.043
          ci95_low_pct: -30.586
          ci95_high_pct: -29.829
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 11766000.0
          candidate_median: 10815000.0
          change_pct: -7.78
          ci95_low_pct: -14.22
          ci95_high_pct: 3.206
          significant: false
          pairs: 16
        blocked_ns:
          control_median: 5047666.5
          candidate_median: 4308583.0
          change_pct: -19.915
          ci95_low_pct: -37.949
          ci95_high_pct: 1.691
          significant: false
          pairs: 16
        peak_rss_bytes:
          control_median: 32456704.0
          candidate_median: 32530432.0
          change_pct: 0.354
          ci95_low_pct: -0.051
          ci95_high_pct: 0.809
          significant: false
          pairs: 16
  reference_tools:
    - name: du
      wall_ns_median: 350447771.0
      argv:
        - "{binary}"
        - "-s"
        - "-k"
        - "{root}"
    - name: dust
      wall_ns_median: 219553291.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 282
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Not a change of its own: this is the three accepted changes measured together against the pre-work baseline, which is the only comparison that can honestly be called a total. Summing the individual experiments would not be, since each had a different control on a differently loaded machine."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -48.909
    reason: "Every measured job improved significantly in one interleaved run against the original baseline, with no sample rejected by the oracle"
    commit: 954d27b
---
# Cumulative effect of every accepted change

## Hypothesis

H1, H5, H10: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**ACCEPTED** — Every measured job improved significantly in one interleaved run against
the original baseline, with no sample rejected by the oracle
