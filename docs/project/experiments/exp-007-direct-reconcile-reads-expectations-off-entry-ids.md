---
title: Direct reconcile reads expectations off entry ids
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-007
  title: Direct reconcile reads expectations off entry ids
  date: 2026-08-11
  hypotheses:
    - H14
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
    control: "HEAD c428fbd: exclusive reconcile re-derives child expectations through path joins and root descents"
    candidate: "ReconcileTarget::Direct routed through collect_child_expectations; the slow twin deleted"
    control_binary:
      name: control
      sha256: eafe60a411c408c5024f0c0fea832f576a845518bb641a79d3e6445626026a0e
      size_bytes: 519328
      args: []
    candidate_binary:
      name: h14
      sha256: 654ca229a3b89508fbe92d5c8ecf70190fa9a41940f991a7495ab2f95c39040b
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
          control_median: 508235063.0
          candidate_median: 511332125.0
          change_pct: 20.567
          ci95_low_pct: -20.797
          ci95_high_pct: 25.869
          significant: false
          pairs: 14
        component_ns:
          control_median: 289833979.5
          candidate_median: 327129062.5
          change_pct: 16.516
          ci95_low_pct: -11.244
          ci95_high_pct: 40.73
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 1080453500.0
          candidate_median: 1065645500.0
          change_pct: -2.648
          ci95_low_pct: -5.352
          ci95_high_pct: 5.84
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 265129000.0
          candidate_median: 265170000.0
          change_pct: 0.119
          ci95_low_pct: -1.533
          ci95_high_pct: 2.55
          significant: false
          pairs: 14
        system_cpu_ns:
          control_median: 821456000.0
          candidate_median: 802061000.0
          change_pct: -3.646
          ci95_low_pct: -7.277
          ci95_high_pct: 7.915
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 38076416.0
          candidate_median: 38330368.0
          change_pct: 3.251
          ci95_low_pct: -0.473
          ci95_high_pct: 12.022
          significant: false
          pairs: 14
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 526571688.0
          candidate_median: 519887334.0
          change_pct: 6.039
          ci95_low_pct: -0.967
          ci95_high_pct: 8.75
          significant: false
          pairs: 14
        component_ns:
          control_median: 208809500.0
          candidate_median: 215709770.5
          change_pct: 3.03
          ci95_low_pct: -0.923
          ci95_high_pct: 10.233
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 2066768500.0
          candidate_median: 2113609000.0
          change_pct: 4.111
          ci95_low_pct: -0.027
          ci95_high_pct: 7.476
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 331013000.0
          candidate_median: 334253000.0
          change_pct: 0.222
          ci95_low_pct: -0.719
          ci95_high_pct: 1.638
          significant: false
          pairs: 14
        system_cpu_ns:
          control_median: 1734306000.0
          candidate_median: 1779819500.0
          change_pct: 4.512
          ci95_low_pct: -0.142
          ci95_high_pct: 9.174
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 36913152.0
          candidate_median: 37412864.0
          change_pct: 1.081
          ci95_low_pct: -0.758
          ci95_high_pct: 2.94
          significant: false
          pairs: 14
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1095792458.5
          candidate_median: 1037379771.0
          change_pct: -1.59
          ci95_low_pct: -10.726
          ci95_high_pct: 1.483
          significant: false
          pairs: 14
        component_ns:
          control_median: 737713854.5
          candidate_median: 636129708.5
          change_pct: -5.917
          ci95_low_pct: -11.664
          ci95_high_pct: -2.995
          significant: true
          pairs: 14
        cpu_ns:
          control_median: 848281500.0
          candidate_median: 782870000.0
          change_pct: -5.375
          ci95_low_pct: -8.726
          ci95_high_pct: -4.198
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 343678500.0
          candidate_median: 288477500.0
          change_pct: -14.009
          ci95_low_pct: -15.659
          ci95_high_pct: -13.148
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 504330500.0
          candidate_median: 494191000.0
          change_pct: 0.341
          ci95_low_pct: -2.63
          ci95_high_pct: 1.862
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 232111458.5
          candidate_median: 240636271.0
          change_pct: 18.18
          ci95_low_pct: -35.857
          ci95_high_pct: 39.174
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 34635776.0
          candidate_median: 34308096.0
          change_pct: -0.621
          ci95_low_pct: -1.559
          ci95_high_pct: 0.72
          significant: false
          pairs: 14
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 286425937.0
          candidate_median: 283683583.0
          change_pct: 2.241
          ci95_low_pct: -0.107
          ci95_high_pct: 18.351
          significant: false
          pairs: 14
        component_ns:
          control_median: 138619979.0
          candidate_median: 133516541.5
          change_pct: -2.374
          ci95_low_pct: -5.633
          ci95_high_pct: 5.448
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 243129500.0
          candidate_median: 245301000.0
          change_pct: 0.808
          ci95_low_pct: -0.14
          ci95_high_pct: 1.498
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 228887500.0
          candidate_median: 230362500.0
          change_pct: 0.654
          ci95_low_pct: -0.585
          ci95_high_pct: 1.285
          significant: false
          pairs: 14
        system_cpu_ns:
          control_median: 14399500.0
          candidate_median: 15133500.0
          change_pct: 2.655
          ci95_low_pct: -3.823
          ci95_high_pct: 6.872
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 47391937.0
          candidate_median: 38183083.0
          change_pct: 27.62
          ci95_low_pct: -11.928
          ci95_high_pct: 71.329
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 33095680.0
          candidate_median: 33095680.0
          change_pct: 0.0
          ci95_low_pct: -1.299
          ci95_high_pct: 0.852
          significant: false
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
    lines_changed: 14
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: A dispatch change. The shared-handle path has read expectations directly since abeb377; the equivalence test locking the two paths together is what lets the twin be deleted rather than maintained.
  verdict:
    decision: in-progress
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -1.59
    reason: "Underpowered, not refuted: component median fell 737.7 to 636.1 ms but the wall interval spans zero at -1.59% [-10.73%, +1.48%] under load average 17 from concurrent builds; committed as a net -10-line simplification on the equivalence test's authority, with the focused re-measurement queued for a quiet machine"
    commit: 92d6212
---
# Direct reconcile reads expectations off entry ids

## Hypothesis

H14: _state what you expected to be slow, why,
and which metric would move._

## What was tried

_The smallest change that tests the hypothesis._

## What the numbers said

_Read the tables in the frontmatter. Say what surprised you._

## Verdict

**IN-PROGRESS** — Underpowered, not refuted: component median fell 737.7 to 636.1 ms but the wall interval spans zero at -1.59% [-10.73%, +1.48%] under load average 17 from concurrent builds; committed as a net -10-line simplification on the equivalence test's authority, with the focused re-measurement queued for a quiet machine
