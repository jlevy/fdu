---
title: Cumulative effect through bounded parallel reconciliation
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-032
  title: Cumulative effect through bounded parallel reconciliation
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
    - H12
    - H9
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
    control: b565882 before the iterative performance campaign
    candidate: "final Rust-1.85-compatible code through exp-030, including accepted cold, snapshot, BFS, bulk metadata, and bounded parallel reconciliation changes"
    control_binary:
      name: control
      sha256: 713a7db449084172489d1e4fd3bc1c8b9f40cf3c352eb65f4af505e127b917d4
      size_bytes: 468832
      args: []
    candidate_binary:
      name: candidate
      sha256: 3ac1e1b2ef50a06fcea4779a06e3f62faebdfe8722daef03163130ec95ccf165
      size_bytes: 585680
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp032-cumulative-through-parallel-reconciliation-msrv-final.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 635371833.5
          candidate_median: 289610104.0
          change_pct: -54.532
          ci95_low_pct: -55.328
          ci95_high_pct: -53.715
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 520890854.0
          candidate_median: 173437729.5
          change_pct: -66.736
          ci95_low_pct: -67.452
          ci95_high_pct: -65.702
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 629746500.0
          candidate_median: 1142684000.0
          change_pct: 82.427
          ci95_low_pct: 73.391
          ci95_high_pct: 86.302
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 236291500.0
          candidate_median: 223656500.0
          change_pct: -5.081
          ci95_low_pct: -8.161
          ci95_high_pct: -3.409
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 391319000.0
          candidate_median: 933176000.0
          change_pct: 133.838
          ci95_low_pct: 122.041
          ci95_high_pct: 142.88
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 6925354.0
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
          control_median: 33570816.0
          candidate_median: 34996224.0
          change_pct: 4.043
          ci95_low_pct: 3.417
          ci95_high_pct: 6.796
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
          control_median: 1218711125.0
          candidate_median: 457088521.0
          change_pct: -60.049
          ci95_low_pct: -62.189
          ci95_high_pct: -58.742
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 475107250.0
          candidate_median: 178588395.5
          change_pct: -62.601
          ci95_low_pct: -63.347
          ci95_high_pct: -61.832
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 1084271500.0
          candidate_median: 1615102500.0
          change_pct: 52.575
          ci95_low_pct: 42.074
          ci95_high_pct: 56.491
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 299419500.0
          candidate_median: 265614000.0
          change_pct: -8.479
          ci95_low_pct: -13.765
          ci95_high_pct: -6.82
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 783329000.0
          candidate_median: 1348078500.0
          change_pct: 77.4
          ci95_low_pct: 63.322
          ci95_high_pct: 81.337
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 92364708.0
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
          control_median: 33734656.0
          candidate_median: 37126144.0
          change_pct: 9.812
          ci95_low_pct: 8.358
          ci95_high_pct: 12.061
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
          control_median: 737116062.5
          candidate_median: 332752208.0
          change_pct: -52.413
          ci95_low_pct: -56.582
          ci95_high_pct: -51.024
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 34594312.5
          candidate_median: 31150312.5
          change_pct: -1.538
          ci95_low_pct: -18.674
          ci95_high_pct: 7.019
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 688631500.0
          candidate_median: 1096248000.0
          change_pct: 57.234
          ci95_low_pct: 43.709
          ci95_high_pct: 71.297
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 263791500.0
          candidate_median: 240733500.0
          change_pct: -8.274
          ci95_low_pct: -9.737
          ci95_high_pct: -5.613
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 424301000.0
          candidate_median: 858610500.0
          change_pct: 97.316
          ci95_low_pct: 78.451
          ci95_high_pct: 119.986
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 44430916.5
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
          control_median: 41738240.0
          candidate_median: 43401216.0
          change_pct: 2.038
          ci95_low_pct: 1.55
          ci95_high_pct: 5.077
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
          control_median: 901698250.0
          candidate_median: 441569208.0
          change_pct: -51.992
          ci95_low_pct: -54.113
          ci95_high_pct: -50.066
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 544073104.5
          candidate_median: 201939854.5
          change_pct: -63.942
          ci95_low_pct: -65.466
          ci95_high_pct: -62.164
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 863775500.0
          candidate_median: 888285500.0
          change_pct: 3.848
          ci95_low_pct: -1.142
          ci95_high_pct: 6.01
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 446653500.0
          candidate_median: 273573000.0
          change_pct: -40.142
          ci95_low_pct: -41.401
          ci95_high_pct: -37.993
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 417420000.0
          candidate_median: 611536000.0
          change_pct: 51.812
          ci95_low_pct: 39.425
          ci95_high_pct: 55.877
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 40040750.0
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
          control_median: 33955840.0
          candidate_median: 34037760.0
          change_pct: -0.024
          ci95_low_pct: -0.484
          ci95_high_pct: 0.796
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 320933291.5
          candidate_median: 206943125.0
          change_pct: -35.657
          ci95_low_pct: -36.177
          ci95_high_pct: -35.322
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 210547959.0
          candidate_median: 96605416.5
          change_pct: -54.139
          ci95_low_pct: -54.619
          ci95_high_pct: -53.7
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 317672500.0
          candidate_median: 204521000.0
          change_pct: -35.637
          ci95_low_pct: -36.099
          ci95_high_pct: -35.505
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 310245500.0
          candidate_median: 197894000.0
          change_pct: -36.332
          ci95_low_pct: -36.562
          ci95_high_pct: -36.114
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 7918500.0
          candidate_median: 6738500.0
          change_pct: -15.716
          ci95_low_pct: -21.363
          ci95_high_pct: -10.807
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 3393791.5
          candidate_median: 2560729.0
          change_pct: -25.332
          ci95_low_pct: -42.677
          ci95_high_pct: -6.475
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 32620544.0
          candidate_median: 30932992.0
          change_pct: -5.532
          ci95_low_pct: -5.988
          ci95_high_pct: -4.532
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
    change_pct: -54.532
    reason: "Against the original binary, final code improves cold index 54.53%, producer 60.05%, snapshot save 52.41%, warm revalidation 51.99%, and snapshot load 35.66%; all exact oracles passed"
    commit: null
---
# Cumulative effect through bounded parallel reconciliation

## Question

exp-032 replaces exp-027 as the exact-binary cumulative anchor after H12/exp-030 changed
the warm path. The comparison asks what the complete accepted campaign buys against
`b565882`, before the iterative traversal work, across every component job the real-tree
harness exposes. It is a reproduction, not a new code hypothesis.

## Method

The original binary (`713a7db44908`) and the final Rust-1.85-compatible candidate
(`3ac1e1b2ef50`) ran twelve interleaved pairs after three warmups for cold index,
producer-only scan, scan plus snapshot save, compatible-snapshot revalidation, and
snapshot load. The 60,067-entry APFS subject was freshly fingerprinted, remained
immutable throughout the run, and every sample passed the independent exact oracle.

The candidate includes the accepted parallel producer, direct child expectations,
extension interning, single-pass snapshot checksum/parse, region-aware breadth-first
scheduling, service-time adaptive cold workers, macOS bulk metadata for cold and warm
paths, and bounded four-worker immutable-baseline reconciliation waves.
The final candidate differs from the exp-030 measurement binary only by an equivalent
syntax rewrite required by the Rust 1.85 minimum supported version.
Complexity and failure modes belong to their individual experiment records; this anchor
changes no behavior.

## Results

Cold indexed scan wall improved 54.53% [-55.33%, -53.72%] and its measured component
66.74%. Producer-only wall improved 60.05% and component 62.60%. Cold scan plus snapshot
save improved 52.41%; the serialization component itself was statistically unclear, as
expected, because the gain is in its setup scan.

Compatible-snapshot warm-open wall improved 51.99% [-54.11%, -50.07%] and its
reconciliation component 63.94%. Snapshot-only load improved 35.66% in wall time and
54.14% in component time, with total CPU down 35.64% and RSS down 5.53%.

The cold wall gains deliberately spend parallel system CPU to remove elapsed wait:
cold-index total CPU is 82.43% higher and producer CPU 52.58% higher, while user CPU is
5.08% and 8.48% lower.
Warm total CPU is statistically unclear at +3.85%, with user CPU down 40.14% and system
CPU up 51.81%. The operating point favors interactive wall latency; explicit thread
controls and the serial reference remain available where CPU budget matters more.

## Verdict

**Accepted as the final cumulative anchor.** All five end-to-end jobs improve
decisively, with live non-cached scan paths roughly twice as fast and verified warm open
also roughly twice as fast against the original binary.
The next high-order work is not another cold constant tweak: it is snapshot bulk
load/persisted roll-ups, journal scoping, platform-specific Linux evidence, and memory
layout.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
