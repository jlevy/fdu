---
title: Parallel revalidation sweep
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-002
  title: Parallel revalidation sweep
  date: 2026-08-10
  hypotheses:
    - H9
  subject:
    tree_label: reference-tree-60k
    tree_root_id: 40406544ab63512154d1962a5c6bbe3bee60c1d3c6315f3b267b99871d03d825
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
    trials: 12
    warmups: 3
    interleaved: true
    control: "exp-001 build: parallel scan, serial revalidation"
    candidate: revalidation sweep parallelized with the same worker pool
    control_binary:
      name: control
      sha256: 80e6049b1259dc6c7a6438a6c684a668c096fdd298d9aaf6bd1417ae6c0b820c
      size_bytes: 519328
      args: []
    candidate_binary:
      name: candidate
      sha256: 767e8500bf323ff4d86547467542fd59367d0ad06a7caeb3529d9b993017618f
      size_bytes: 519328
      args: []
    toolchain: ""
    build_profile: release
    evidence_grade: legacy
    run_schema: fdu-realtree-run-v1
    schedule: round-robin-by-ordinal-v1
    schedule_sha256: null
    schedule_seed: null
    run_artifact: docs/project/experiments/evidence/exp-002-run.json
    run_artifact_sha256: 48c303d73f2c9f1e0b6c20173ad0a80b7c606a3cd6b75dd9a7914692995d9aea
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 304994166.0
          candidate_median: 304347833.5
          change_pct: -1.267
          ci95_low_pct: -4.196
          ci95_high_pct: 1.439
          significant: false
          pairs: 12
        component_ns:
          control_median: 190720729.5
          candidate_median: 189996312.5
          change_pct: -1.656
          ci95_low_pct: -5.97
          ci95_high_pct: 3.465
          significant: false
          pairs: 12
        cpu_ns:
          control_median: 1254782000.0
          candidate_median: 1263797000.0
          change_pct: -0.452
          ci95_low_pct: -5.509
          ci95_high_pct: 4.6
          significant: false
          pairs: 12
        user_cpu_ns:
          control_median: 279845500.0
          candidate_median: 271842000.0
          change_pct: -2.668
          ci95_low_pct: -2.934
          ci95_high_pct: -1.616
          significant: true
          pairs: 12
        system_cpu_ns:
          control_median: 972808000.0
          candidate_median: 992102500.0
          change_pct: 0.422
          ci95_low_pct: -5.968
          ci95_high_pct: 6.548
          significant: false
          pairs: 12
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 36560896.0
          candidate_median: 36536320.0
          change_pct: -0.158
          ci95_low_pct: -0.959
          ci95_high_pct: 2.455
          significant: false
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 810036833.5
          candidate_median: 786415854.0
          change_pct: -2.593
          ci95_low_pct: -4.548
          ci95_high_pct: -0.223
          significant: true
          pairs: 12
        component_ns:
          control_median: 478669541.5
          candidate_median: 470765312.5
          change_pct: -2.105
          ci95_low_pct: -5.293
          ci95_high_pct: 1.525
          significant: false
          pairs: 12
        cpu_ns:
          control_median: 801145500.0
          candidate_median: 780502500.0
          change_pct: -2.36
          ci95_low_pct: -4.02
          ci95_high_pct: 0.046
          significant: false
          pairs: 12
        user_cpu_ns:
          control_median: 427602000.0
          candidate_median: 416603000.0
          change_pct: -2.308
          ci95_low_pct: -3.313
          ci95_high_pct: -1.792
          significant: true
          pairs: 12
        system_cpu_ns:
          control_median: 371773500.0
          candidate_median: 365314500.0
          change_pct: -1.642
          ci95_low_pct: -4.626
          ci95_high_pct: 2.335
          significant: false
          pairs: 12
        blocked_ns:
          control_median: 7955562.5
          candidate_median: 5058499.5
          change_pct: -34.325
          ci95_low_pct: -69.521
          ci95_high_pct: 11.327
          significant: false
          pairs: 12
        peak_rss_bytes:
          control_median: 33873920.0
          candidate_median: 34070528.0
          change_pct: -0.048
          ci95_low_pct: -0.193
          ci95_high_pct: 2.807
          significant: false
          pairs: 12
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 324355125.0
          candidate_median: 316575228.5
          change_pct: -2.361
          ci95_low_pct: -2.792
          ci95_high_pct: -1.778
          significant: true
          pairs: 12
        component_ns:
          control_median: 214044166.5
          candidate_median: 208105604.0
          change_pct: -2.6
          ci95_low_pct: -3.446
          ci95_high_pct: -1.794
          significant: true
          pairs: 12
        cpu_ns:
          control_median: 321667000.0
          candidate_median: 313333500.0
          change_pct: -2.302
          ci95_low_pct: -2.709
          ci95_high_pct: -2.088
          significant: true
          pairs: 12
        user_cpu_ns:
          control_median: 310307500.0
          candidate_median: 302723000.0
          change_pct: -2.403
          ci95_low_pct: -2.742
          ci95_high_pct: -1.993
          significant: true
          pairs: 12
        system_cpu_ns:
          control_median: 10805000.0
          candidate_median: 10056000.0
          change_pct: -2.39
          ci95_low_pct: -10.133
          ci95_high_pct: 3.35
          significant: false
          pairs: 12
        blocked_ns:
          control_median: 2910416.5
          candidate_median: 3120687.5
          change_pct: -3.88
          ci95_low_pct: -11.727
          ci95_high_pct: 20.172
          significant: false
          pairs: 12
        peak_rss_bytes:
          control_median: 32571392.0
          candidate_median: 32677888.0
          change_pct: 0.101
          ci95_low_pct: -0.476
          ci95_high_pct: 0.674
          significant: false
          pairs: 12
  reference_tools:
    - name: dust
      wall_ns_median: 220190499.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 180
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - a second concurrent producer path to keep correct as the index evolves
    notes: "Reverted. The negative result is the value: it located the warm-path bottleneck in apply rather than in traversal, which is what pointed at exp-004. Note also that warm-snapshot-load, which does not touch this code at all, moved a similar -2.36% in the same run — a reminder of how much of a small number can be drift."
  verdict:
    decision: rejected
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -2.593
    reason: "A real 2.59% but under the 3% bar, for roughly 180 lines of concurrency"
    commit: null
---
# Parallel revalidation sweep

## Hypothesis

H9 predicted that applying the producer pool to the unchanged-tree revalidation sweep
would reduce warm-start wall time enough to justify another concurrent path.

## What was tried

The control kept exp-001's parallel cold scan and serial revalidation. The candidate
routed revalidation through the same pool, adding roughly 180 lines of concurrency.

## What the numbers said

Warm-revalidate wall improved 2.59% in the paired run. The direction was favorable, but
the effect did not clear the predeclared 3% carrying-cost threshold; the extra machinery
also did not change the N-stat verification floor.

## Verdict

**REJECTED** — A real 2.59% but under the 3% bar, for roughly 180 lines of concurrency

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
