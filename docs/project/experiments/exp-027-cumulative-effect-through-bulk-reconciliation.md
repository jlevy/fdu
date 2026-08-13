---
title: Cumulative effect through bulk reconciliation
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-027
  title: Cumulative effect through bulk reconciliation
  date: 2026-08-12
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
    - H53
  subject:
    tree_label: metabrowser-20260812
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
    candidate: "current code through exp-026, including accepted traversal, index, snapshot, BFS scheduling, adaptive-depth, and macOS bulk cold/warm changes"
    control_binary:
      name: control
      sha256: 713a7db449084172489d1e4fd3bc1c8b9f40cf3c352eb65f4af505e127b917d4
      size_bytes: 468832
      args: []
    candidate_binary:
      name: candidate
      sha256: 35198f0525f9501b71bd6764362f35723c925a3689b99c587bfbc457da896019
      size_bytes: 569104
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp027-cumulative-through-bulk-reconciliation.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 586921499.5
          candidate_median: 275416520.5
          change_pct: -52.835
          ci95_low_pct: -53.433
          ci95_high_pct: -52.605
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 475825229.0
          candidate_median: 165353458.5
          change_pct: -65.421
          ci95_low_pct: -66.219
          ci95_high_pct: -64.61
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 582742500.0
          candidate_median: 1113555500.0
          change_pct: 91.361
          ci95_low_pct: 89.211
          ci95_high_pct: 92.772
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 228890500.0
          candidate_median: 195326500.0
          change_pct: -14.534
          ci95_low_pct: -15.897
          ci95_high_pct: -6.569
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 352747000.0
          candidate_median: 916970500.0
          change_pct: 160.209
          ci95_low_pct: 154.237
          ci95_high_pct: 163.661
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 4281771.0
          candidate_median: 0.0
          change_pct: -100.0
          ci95_low_pct: -100.0
          ci95_high_pct: -100.0
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 33521664.0
          candidate_median: 34504704.0
          change_pct: 2.622
          ci95_low_pct: 2.031
          ci95_high_pct: 3.425
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
          control_median: 903298646.0
          candidate_median: 376447750.0
          change_pct: -58.287
          ci95_low_pct: -58.546
          ci95_high_pct: -58.144
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 377156708.5
          candidate_median: 155887416.0
          change_pct: -58.607
          ci95_low_pct: -59.457
          ci95_high_pct: -57.852
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 897917500.0
          candidate_median: 1717197500.0
          change_pct: 90.476
          ci95_low_pct: 88.791
          ci95_high_pct: 92.107
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 259567000.0
          candidate_median: 225213000.0
          change_pct: -13.478
          ci95_low_pct: -14.122
          ci95_high_pct: -12.019
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 639512500.0
          candidate_median: 1491450000.0
          change_pct: 132.731
          ci95_low_pct: 130.691
          ci95_high_pct: 135.013
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 5072229.5
          candidate_median: 0.0
          change_pct: -100.0
          ci95_low_pct: -100.0
          ci95_high_pct: -100.0
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 33546240.0
          candidate_median: 34275328.0
          change_pct: 1.999
          ci95_low_pct: 1.558
          ci95_high_pct: 4.009
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
          control_median: 610143312.5
          candidate_median: 296389896.0
          change_pct: -51.128
          ci95_low_pct: -51.829
          ci95_high_pct: -50.679
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 23914458.5
          candidate_median: 23809479.0
          change_pct: 0.216
          ci95_low_pct: -10.55
          ci95_high_pct: 5.532
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 599282500.0
          candidate_median: 1111567500.0
          change_pct: 85.793
          ci95_low_pct: 84.688
          ci95_high_pct: 89.018
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 243436000.0
          candidate_median: 207802000.0
          change_pct: -14.846
          ci95_low_pct: -15.676
          ci95_high_pct: -12.427
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 354732000.0
          candidate_median: 903765500.0
          change_pct: 154.194
          ci95_low_pct: 153.43
          ci95_high_pct: 160.329
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 11413104.0
          candidate_median: 0.0
          change_pct: -100.0
          ci95_low_pct: -100.0
          ci95_high_pct: -100.0
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 43016192.0
          candidate_median: 43270144.0
          change_pct: 1.048
          ci95_low_pct: -0.038
          ci95_high_pct: 2.407
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 825682250.0
          candidate_median: 536512333.0
          change_pct: -34.776
          ci95_low_pct: -37.161
          ci95_high_pct: -31.029
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 489717166.5
          candidate_median: 329178479.0
          change_pct: -33.822
          ci95_low_pct: -35.817
          ci95_high_pct: -29.873
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 806419000.0
          candidate_median: 530143500.0
          change_pct: -34.849
          ci95_low_pct: -36.42
          ci95_high_pct: -31.875
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 434082000.0
          candidate_median: 235649000.0
          change_pct: -45.291
          ci95_low_pct: -46.601
          ci95_high_pct: -43.728
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 375955500.0
          candidate_median: 296302500.0
          change_pct: -22.489
          ci95_low_pct: -25.042
          ci95_high_pct: -18.28
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 12285167.0
          candidate_median: 6968145.5
          change_pct: -20.94
          ci95_low_pct: -46.874
          ci95_high_pct: 74.438
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 33882112.0
          candidate_median: 32636928.0
          change_pct: -3.7
          ci95_low_pct: -4.464
          ci95_high_pct: -2.51
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
          control_median: 383783521.0
          candidate_median: 261126854.5
          change_pct: -32.687
          ci95_low_pct: -38.765
          ci95_high_pct: -30.772
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 261953083.5
          candidate_median: 134852271.0
          change_pct: -51.921
          ci95_low_pct: -54.957
          ci95_high_pct: -43.956
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 360583500.0
          candidate_median: 232072500.0
          change_pct: -35.83
          ci95_low_pct: -38.702
          ci95_high_pct: -34.442
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 345767000.0
          candidate_median: 218126500.0
          change_pct: -36.666
          ci95_low_pct: -39.79
          ci95_high_pct: -34.692
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 13757000.0
          candidate_median: 10369500.0
          change_pct: -13.734
          ci95_low_pct: -25.095
          ci95_high_pct: -6.994
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 25720854.0
          candidate_median: 23512583.0
          change_pct: 2.737
          ci95_low_pct: -50.443
          ci95_high_pct: 47.713
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 32604160.0
          candidate_median: 30834688.0
          change_pct: -5.517
          ci95_low_pct: -6.949
          ci95_high_pct: -5.047
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
    change_pct: -52.835
    reason: "Against the original baseline, current code improved cold index 52.84%, producer 58.29%, snapshot save 51.13%, warm revalidation 34.78%, and snapshot load 32.69%; all oracle checks passed"
    commit: null
---
# Cumulative effect through bulk reconciliation

## Purpose

This measurement replaces exp-023 as the current cumulative anchor after exp-026 added
bulk metadata to full macOS reconciliation.
It asks one question only: what does the complete accepted stack now do relative to the
exact `b565882` pre-campaign binary?
Complexity and causal attribution remain in the individual experiments.

## Method

The exact original release probe and the exact release probe from `824f2c4` ran twelve
interleaved pairs after three warmups across all five loop jobs.
The subject was freshly fingerprinted before the run: 60,067 entries on APFS, with the
same engine digest before and after every job.
All samples passed the independent oracle.
The candidate binary is the same hash used for exp-026’s final 60k and 720k gates.

## Results

Against the original build, the current stack improves cold indexed wall 52.84%
[-53.43%, -52.60%], producer wall 58.29%, and cold scan plus snapshot save 51.13%. Warm
revalidation now improves 34.78% [-37.16%, -31.03%], up from exp-023’s 20.60% cumulative
result before bulk reconciliation.
Snapshot load improves 32.69%.

The indexed candidate retained one 675-ms host-load outlier among otherwise roughly
267–282-ms samples.
It remains in the evidence; the paired interval still clears the gate
by a wide margin. These are warm-steady operating-system-cache results, not
controlled-cold claims.

## Verdict

**Accepted as the new cumulative anchor.** The accepted traversal, index, snapshot,
breadth-first scheduling, adaptive-depth, macOS cold bulk-metadata, and macOS bulk-
reconciliation changes compose across every measured job.
This record introduces no code and must not be used to double-count the individual
experiment gains.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
