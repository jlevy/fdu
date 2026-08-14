---
title: Baseline on a real 60k-entry tree
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-000
  title: Baseline on a real 60k-entry tree
  date: "2026-08-10"
  hypotheses: []
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
    trials: 12
    warmups: 3
    interleaved: true
    control: "main @ b565882"
    candidate: none; this establishes the numbers
    control_binary:
      name: baseline
      sha256: 5584ac632e16b3feb2777089822da5923375e47fb36d8c16f75c29de39ec9241
      size_bytes: 468832
      args: []
    candidate_binary:
      name: baseline
      sha256: 5584ac632e16b3feb2777089822da5923375e47fb36d8c16f75c29de39ec9241
      size_bytes: 468832
      args: []
    toolchain: ""
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp000-baseline.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 627512875.0
          candidate_median: 627512875.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        component_ns:
          control_median: 514465041.5
          candidate_median: 514465041.5
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        cpu_ns:
          control_median: 605256000.0
          candidate_median: 605256000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        user_cpu_ns:
          control_median: 231353000.0
          candidate_median: 231353000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        system_cpu_ns:
          control_median: 373603000.0
          candidate_median: 373603000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        blocked_ns:
          control_median: 11276208.5
          candidate_median: 11276208.5
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 33570816.0
          candidate_median: 33570816.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 932458833.0
          candidate_median: 932458833.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        component_ns:
          control_median: 385843687.0
          candidate_median: 385843687.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        cpu_ns:
          control_median: 922814500.0
          candidate_median: 922814500.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        user_cpu_ns:
          control_median: 262414000.0
          candidate_median: 262414000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        system_cpu_ns:
          control_median: 660777000.0
          candidate_median: 660777000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        blocked_ns:
          control_median: 6441874.5
          candidate_median: 6441874.5
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 33562624.0
          candidate_median: 33562624.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 643233729.0
          candidate_median: 643233729.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        component_ns:
          control_median: 36877271.0
          candidate_median: 36877271.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        cpu_ns:
          control_median: 623503500.0
          candidate_median: 623503500.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        user_cpu_ns:
          control_median: 246284000.0
          candidate_median: 246284000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        system_cpu_ns:
          control_median: 377133000.0
          candidate_median: 377133000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        blocked_ns:
          control_median: 20826229.0
          candidate_median: 20826229.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 41959424.0
          candidate_median: 41959424.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 795170479.0
          candidate_median: 795170479.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        component_ns:
          control_median: 472306458.5
          candidate_median: 472306458.5
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        cpu_ns:
          control_median: 789422000.0
          candidate_median: 789422000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        user_cpu_ns:
          control_median: 420049500.0
          candidate_median: 420049500.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        system_cpu_ns:
          control_median: 369128000.0
          candidate_median: 369128000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        blocked_ns:
          control_median: 6074646.0
          candidate_median: 6074646.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 34062336.0
          candidate_median: 34062336.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 335995208.5
          candidate_median: 335995208.5
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        component_ns:
          control_median: 219478750.0
          candidate_median: 219478750.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        cpu_ns:
          control_median: 328682000.0
          candidate_median: 328682000.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        user_cpu_ns:
          control_median: 315787500.0
          candidate_median: 315787500.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        system_cpu_ns:
          control_median: 13309500.0
          candidate_median: 13309500.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        blocked_ns:
          control_median: 7689729.0
          candidate_median: 7689729.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 32907264.0
          candidate_median: 32907264.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
  reference_tools:
    - name: du
      wall_ns_median: 342459687.5
      argv:
        - "{binary}"
        - "-s"
        - "-k"
        - "{root}"
    - name: dust
      wall_ns_median: 211926500.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: No production code changed.
  verdict:
    decision: baseline
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: null
    reason: Establishes the reference numbers every later experiment is measured against
    commit: b565882
---
# Baseline on a real 60k-entry tree

## Hypothesis

—: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**BASELINE** — Establishes the reference numbers every later experiment is measured
against
