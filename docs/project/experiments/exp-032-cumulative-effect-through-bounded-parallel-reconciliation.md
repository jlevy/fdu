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
    candidate: "current code through exp-030, including accepted cold, snapshot, BFS, bulk metadata, and bounded parallel reconciliation changes"
    control_binary:
      name: control
      sha256: 713a7db449084172489d1e4fd3bc1c8b9f40cf3c352eb65f4af505e127b917d4
      size_bytes: 468832
      args: []
    candidate_binary:
      name: candidate
      sha256: 54db14278796b5ab1233ed71eefe07e2061c3913a957ac8e4f5fa79a8a4c2765
      size_bytes: 585680
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp032-cumulative-through-parallel-reconciliation.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 584398104.0
          candidate_median: 270294562.5
          change_pct: -53.588
          ci95_low_pct: -54.379
          ci95_high_pct: -53.274
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 474619396.0
          candidate_median: 159673083.0
          change_pct: -66.254
          ci95_low_pct: -67.182
          ci95_high_pct: -65.965
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 578740500.0
          candidate_median: 1090399500.0
          change_pct: 88.214
          ci95_low_pct: 84.616
          ci95_high_pct: 90.408
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 227686500.0
          candidate_median: 192163000.0
          change_pct: -15.76
          ci95_low_pct: -15.908
          ci95_high_pct: -15.143
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 350945000.0
          candidate_median: 897188000.0
          change_pct: 155.321
          ci95_low_pct: 148.528
          ci95_high_pct: 158.46
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 4087666.0
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
          control_median: 33464320.0
          candidate_median: 34299904.0
          change_pct: 2.396
          ci95_low_pct: 2.108
          ci95_high_pct: 2.694
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
          control_median: 945004479.0
          candidate_median: 397101979.5
          change_pct: -57.869
          ci95_low_pct: -58.374
          ci95_high_pct: -56.854
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 397724250.5
          candidate_median: 169244020.5
          change_pct: -57.332
          ci95_low_pct: -58.465
          ci95_high_pct: -56.485
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 936223000.0
          candidate_median: 1778245500.0
          change_pct: 91.624
          ci95_low_pct: 88.866
          ci95_high_pct: 94.135
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 263774500.0
          candidate_median: 236082000.0
          change_pct: -10.341
          ci95_low_pct: -11.15
          ci95_high_pct: -8.165
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 671364500.0
          candidate_median: 1542476000.0
          change_pct: 130.341
          ci95_low_pct: 126.673
          ci95_high_pct: 134.194
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 7338916.5
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
          control_median: 33529856.0
          candidate_median: 35020800.0
          change_pct: 4.621
          ci95_low_pct: 3.507
          ci95_high_pct: 5.021
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
          control_median: 619310187.5
          candidate_median: 302702250.5
          change_pct: -51.326
          ci95_low_pct: -51.88
          ci95_high_pct: -50.541
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 23639958.5
          candidate_median: 23400333.5
          change_pct: -0.566
          ci95_low_pct: -4.541
          ci95_high_pct: 3.872
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 608553500.0
          candidate_median: 1147993000.0
          change_pct: 86.238
          ci95_low_pct: 84.943
          ci95_high_pct: 89.881
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 243712000.0
          candidate_median: 209915000.0
          change_pct: -13.626
          ci95_low_pct: -14.302
          ci95_high_pct: -12.192
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 366481500.0
          candidate_median: 939285500.0
          change_pct: 153.782
          ci95_low_pct: 150.802
          ci95_high_pct: 158.179
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 10336208.5
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
          control_median: 41934848.0
          candidate_median: 43180032.0
          change_pct: 2.868
          ci95_low_pct: 0.969
          ci95_high_pct: 3.68
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
          control_median: 798986437.5
          candidate_median: 374705854.5
          change_pct: -54.26
          ci95_low_pct: -55.979
          ci95_high_pct: -52.588
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 483675208.5
          candidate_median: 166508979.5
          change_pct: -67.019
          ci95_low_pct: -68.802
          ci95_high_pct: -64.294
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 786321500.0
          candidate_median: 837277000.0
          change_pct: 2.291
          ci95_low_pct: -1.451
          ci95_high_pct: 6.963
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 420928500.0
          candidate_median: 252323000.0
          change_pct: -40.035
          ci95_low_pct: -41.732
          ci95_high_pct: -39.349
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 367617500.0
          candidate_median: 579704000.0
          change_pct: 51.046
          ci95_low_pct: 43.551
          ci95_high_pct: 60.416
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 8645646.0
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
          control_median: 33890304.0
          candidate_median: 34054144.0
          change_pct: 0.377
          ci95_low_pct: -0.485
          ci95_high_pct: 1.264
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
          control_median: 330053770.5
          candidate_median: 213628020.5
          change_pct: -35.248
          ci95_low_pct: -35.892
          ci95_high_pct: -33.815
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 215967520.5
          candidate_median: 99924916.5
          change_pct: -54.409
          ci95_low_pct: -55.183
          ci95_high_pct: -51.895
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 324349000.0
          candidate_median: 208998000.0
          change_pct: -35.593
          ci95_low_pct: -36.076
          ci95_high_pct: -34.807
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 316243500.0
          candidate_median: 201639000.0
          change_pct: -36.114
          ci95_low_pct: -36.48
          ci95_high_pct: -35.437
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 8427500.0
          candidate_median: 7227000.0
          change_pct: -17.251
          ci95_low_pct: -24.053
          ci95_high_pct: -10.015
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 4726270.5
          candidate_median: 4782520.5
          change_pct: 5.711
          ci95_low_pct: -29.292
          ci95_high_pct: 53.683
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 32571392.0
          candidate_median: 30629888.0
          change_pct: -5.94
          ci95_low_pct: -6.739
          ci95_high_pct: -4.524
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
    change_pct: -53.588
    reason: "Against the original binary, current code improves cold index 53.59%, producer 57.87%, snapshot save 51.33%, warm revalidation 54.26%, and snapshot load 35.25%; all exact oracles passed"
    commit: null
---
# Cumulative effect through bounded parallel reconciliation

## Question

exp-032 replaces exp-027 as the exact-binary cumulative anchor after H12/exp-030
changed the warm path. The comparison asks what the complete accepted campaign buys
against `b565882`, before the iterative traversal work, across every component job the
real-tree harness exposes. It is a reproduction, not a new code hypothesis.

## Method

The original binary (`713a7db44908`) and current binary through exp-030
(`54db14278796`) ran twelve interleaved pairs after three warmups for cold index,
producer-only scan, scan plus snapshot save, compatible-snapshot revalidation, and
snapshot load. The 60,067-entry APFS subject was freshly fingerprinted, remained
immutable throughout the run, and every sample passed the independent exact oracle.

The candidate includes the accepted parallel producer, direct child expectations,
extension interning, single-pass snapshot checksum/parse, region-aware breadth-first
scheduling, service-time adaptive cold workers, macOS bulk metadata for cold and warm
paths, and bounded four-worker immutable-baseline reconciliation waves. Complexity and
failure modes belong to their individual experiment records; this anchor changes no
code.

## Results

Cold indexed scan wall improved 53.59% [-54.38%, -53.27%] and its measured component
66.25%. Producer-only wall improved 57.87% and component 57.33%. Cold scan plus snapshot
save improved 51.33%; the serialization component itself was neutral, as expected,
because the gain is in its setup scan.

Compatible-snapshot warm-open wall improved 54.26% [-55.98%, -52.59%] and its
reconciliation component 67.02%. Snapshot-only load improved 35.25% in wall time and
54.41% in component time, with total CPU down 35.59% and RSS down 5.94%.

The cold wall gains deliberately spend parallel system CPU to remove elapsed wait:
cold-index total CPU is 88.21% higher and producer CPU 91.62% higher, while user CPU is
15.76% and 10.34% lower. Warm total CPU is statistically unclear at +2.29%, with user
CPU down 40.03% and system CPU up 51.05%. The operating point favors interactive wall
latency; explicit thread controls and the serial reference remain available where CPU
budget matters more.

## Verdict

**Accepted as the new cumulative anchor.** All five end-to-end jobs improve decisively,
with live non-cached scan paths roughly twice as fast and verified warm open now gaining
slightly more than cold index against the original binary. The next high-order work is
not another cold constant tweak: it is snapshot bulk load/persisted roll-ups, journal
scoping, platform-specific Linux evidence, and memory layout.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
