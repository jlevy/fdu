---
title: Bounded parallel directory producer
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-001
  title: Bounded parallel directory producer
  date: "2026-08-10"
  hypotheses:
    - H1
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
    control: serial read_dir walker (--threads 1)
    candidate: four producer threads feeding one index consumer (--threads 4)
    control_binary:
      name: serial
      sha256: 426e3ae76721f0642734c4a092323c3d3df673122391909ea345819bc2edb227
      size_bytes: 519328
      args:
        - "--threads"
        - "1"
    candidate_binary:
      name: par4
      sha256: 426e3ae76721f0642734c4a092323c3d3df673122391909ea345819bc2edb227
      size_bytes: 519328
      args:
        - "--threads"
        - "4"
    toolchain: ""
    build_profile: release
    evidence_grade: legacy
    run_schema: fdu-realtree-run-v1
    schedule: round-robin-by-ordinal-v1
    schedule_sha256: null
    schedule_seed: null
    run_artifact: docs/project/experiments/evidence/exp-001-run.json
    run_artifact_sha256: 2296447b9e3f7eb045c055630089803a83a6d08a90f9914cf4eef144acb4e5ea
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 621757333.0
          candidate_median: 310774958.5
          change_pct: -50.033
          ci95_low_pct: -51.024
          ci95_high_pct: -43.743
          significant: true
          pairs: 12
        component_ns:
          control_median: 507024520.5
          candidate_median: 196556187.5
          change_pct: -61.64
          ci95_low_pct: -62.778
          ci95_high_pct: -54.774
          significant: true
          pairs: 12
        cpu_ns:
          control_median: 612012500.0
          candidate_median: 973572500.0
          change_pct: 58.046
          ci95_low_pct: 54.555
          ci95_high_pct: 66.561
          significant: false
          pairs: 12
        user_cpu_ns:
          control_median: 231015500.0
          candidate_median: 271879500.0
          change_pct: 17.36
          ci95_low_pct: 16.389
          ci95_high_pct: 22.58
          significant: false
          pairs: 12
        system_cpu_ns:
          control_median: 379393000.0
          candidate_median: 702645000.0
          change_pct: 83.5
          ci95_low_pct: 77.055
          ci95_high_pct: 92.721
          significant: false
          pairs: 12
        blocked_ns:
          control_median: 8143396.0
          candidate_median: 0.0
          change_pct: -100.0
          ci95_low_pct: -100.0
          ci95_high_pct: -100.0
          significant: true
          pairs: 12
        peak_rss_bytes:
          control_median: 33275904.0
          candidate_median: 35758080.0
          change_pct: 7.486
          ci95_low_pct: 6.214
          ci95_high_pct: 9.831
          significant: false
          pairs: 12
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 968524812.5
          candidate_median: 482966562.5
          change_pct: -50.662
          ci95_low_pct: -51.704
          ci95_high_pct: -45.946
          significant: true
          pairs: 12
        component_ns:
          control_median: 396807187.5
          candidate_median: 192336333.5
          change_pct: -52.555
          ci95_low_pct: -55.2
          ci95_high_pct: -46.324
          significant: true
          pairs: 12
        cpu_ns:
          control_median: 960882000.0
          candidate_median: 1640718000.0
          change_pct: 70.117
          ci95_low_pct: 64.48
          ci95_high_pct: 79.819
          significant: false
          pairs: 12
        user_cpu_ns:
          control_median: 262676000.0
          candidate_median: 335877500.0
          change_pct: 27.501
          ci95_low_pct: 23.851
          ci95_high_pct: 31.38
          significant: false
          pairs: 12
        system_cpu_ns:
          control_median: 699657500.0
          candidate_median: 1319959500.0
          change_pct: 87.39
          ci95_low_pct: 79.888
          ci95_high_pct: 97.51
          significant: false
          pairs: 12
        blocked_ns:
          control_median: 10342729.0
          candidate_median: 0.0
          change_pct: -100.0
          ci95_low_pct: -100.0
          ci95_high_pct: -100.0
          significant: true
          pairs: 12
        peak_rss_bytes:
          control_median: 33259520.0
          candidate_median: 36167680.0
          change_pct: 7.914
          ci95_low_pct: 7.095
          ci95_high_pct: 10.399
          significant: false
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 828400729.0
          candidate_median: 818697583.5
          change_pct: 0.279
          ci95_low_pct: -0.534
          ci95_high_pct: 4.738
          significant: false
          pairs: 12
        component_ns:
          control_median: 501386125.5
          candidate_median: 497779395.5
          change_pct: 0.254
          ci95_low_pct: -1.478
          ci95_high_pct: 3.183
          significant: false
          pairs: 12
        cpu_ns:
          control_median: 814799000.0
          candidate_median: 813232000.0
          change_pct: -0.19
          ci95_low_pct: -0.715
          ci95_high_pct: 2.155
          significant: false
          pairs: 12
        user_cpu_ns:
          control_median: 424418500.0
          candidate_median: 423112500.0
          change_pct: 0.212
          ci95_low_pct: -0.229
          ci95_high_pct: 1.824
          significant: false
          pairs: 12
        system_cpu_ns:
          control_median: 388376500.0
          candidate_median: 390826500.0
          change_pct: -0.146
          ci95_low_pct: -2.274
          ci95_high_pct: 3.239
          significant: false
          pairs: 12
        blocked_ns:
          control_median: 6666729.0
          candidate_median: 6598062.0
          change_pct: 15.81
          ci95_low_pct: -4.452
          ci95_high_pct: 126.203
          significant: false
          pairs: 12
        peak_rss_bytes:
          control_median: 34521088.0
          candidate_median: 33923072.0
          change_pct: -1.053
          ci95_low_pct: -2.913
          ci95_high_pct: 0.024
          significant: false
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 210
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - a worker panic is now reported as a partial scan rather than propagating
    notes: "std threads, one mutex-guarded work list, one channel. Producers still never touch the index. The sweep also measured 2, 6 and 8 threads: 6 matched 4 within noise and 8 was 4% worse, which is where the automatic cap of 6 comes from."
  verdict:
    decision: superseded
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -50.033
    reason: "Legacy v1 evidence measured the latency win but did not preserve the full roll-up oracle, exact toolchain, source manifests, or schedule digest required for an accepted claim; exp-012 supersedes the cumulative performance decision"
    commit: a0cc981
---
# Bounded parallel directory producer

## Hypothesis

H1 predicted that filesystem observation had enough independent directory work for a
bounded producer pool to reduce cold-scan wall time, even though one consumer still
applied observations to the index.

## What was tried

The same release binary was measured with one and four producer threads.
The candidate used a bounded directory queue and retained the single Delta consumer,
isolating producer concurrency from index mutation.

## What the numbers said

Cold-scan wall fell about 50% in the v1 run.
The result correctly motivated the worker pool, but the original implementation and
evidence later proved incomplete: cancellation could hang, the observation channel was
unbounded, and the v1 oracle did not cover every roll-up reducer.
This branch repairs those contracts before remeasurement.

## Verdict

**SUPERSEDED** — the latency observation remains historical evidence; exp-012 applies
the full oracle, provenance, and resource gates to the true-base cumulative candidate.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
