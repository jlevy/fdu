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
    trials: 20
    warmups: 4
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
    run_artifact: benchmarks/results/realtree/run-exp007-009-requiem.json
  results:
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 712913291.5
          candidate_median: 661987562.5
          change_pct: -7.092
          ci95_low_pct: -8.917
          ci95_high_pct: -5.758
          significant: true
          pairs: 20
        component_ns:
          control_median: 481543854.5
          candidate_median: 431683625.0
          change_pct: -9.355
          ci95_low_pct: -12.817
          ci95_high_pct: -8.306
          significant: true
          pairs: 20
        cpu_ns:
          control_median: 701300000.0
          candidate_median: 655901500.0
          change_pct: -5.843
          ci95_low_pct: -8.532
          ci95_high_pct: -5.697
          significant: true
          pairs: 20
        user_cpu_ns:
          control_median: 312068000.0
          candidate_median: 265396500.0
          change_pct: -15.097
          ci95_low_pct: -15.43
          ci95_high_pct: -14.384
          significant: true
          pairs: 20
        system_cpu_ns:
          control_median: 387964000.0
          candidate_median: 389797500.0
          change_pct: 1.06
          ci95_low_pct: -2.595
          ci95_high_pct: 1.484
          significant: false
          pairs: 20
        blocked_ns:
          control_median: 10677583.0
          candidate_median: 7405229.0
          change_pct: -31.944
          ci95_low_pct: -39.474
          ci95_high_pct: -19.853
          significant: true
          pairs: 20
        peak_rss_bytes:
          control_median: 34021376.0
          candidate_median: 33939456.0
          change_pct: -0.338
          ci95_low_pct: -0.767
          ci95_high_pct: 0.46
          significant: false
          pairs: 20
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 331246374.5
          candidate_median: 369856750.0
          change_pct: 5.742
          ci95_low_pct: -2.82
          ci95_high_pct: 15.069
          significant: false
          pairs: 20
        component_ns:
          control_median: 170653875.0
          candidate_median: 183425646.0
          change_pct: 1.58
          ci95_low_pct: -7.447
          ci95_high_pct: 16.934
          significant: false
          pairs: 20
        cpu_ns:
          control_median: 256737000.0
          candidate_median: 260348000.0
          change_pct: 0.975
          ci95_low_pct: -0.165
          ci95_high_pct: 2.93
          significant: false
          pairs: 20
        user_cpu_ns:
          control_median: 241632500.0
          candidate_median: 242487000.0
          change_pct: 0.53
          ci95_low_pct: -0.505
          ci95_high_pct: 2.545
          significant: false
          pairs: 20
        system_cpu_ns:
          control_median: 15052500.0
          candidate_median: 15889500.0
          change_pct: 6.746
          ci95_low_pct: -0.294
          ci95_high_pct: 16.456
          significant: false
          pairs: 20
        blocked_ns:
          control_median: 73839729.0
          candidate_median: 105217375.0
          change_pct: 16.256
          ci95_low_pct: -6.185
          ci95_high_pct: 81.16
          significant: false
          pairs: 20
        peak_rss_bytes:
          control_median: 32751616.0
          candidate_median: 32628736.0
          change_pct: -0.398
          ci95_low_pct: -0.926
          ci95_high_pct: 0.172
          significant: false
          pairs: 20
  reference_tools:
    - name: dust
      wall_ns_median: 222827604.0
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
    notes: "A net -10-line dispatch change; the equivalence test locking the two reconcile paths together is what lets the slow twin be deleted rather than maintained. First measured under load average 17 (interval spanned zero), committed then as a simplification, and confirmed as a real win on the quiet re-run."
  verdict:
    decision: accepted
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -7.092
    reason: "Quiet-machine re-run: warm-revalidate wall -7.09% with 95% interval [-8.92%, -5.76%] over 20 paired trials; the first, load-average-17 run was underpowered, not wrong"
    commit: 92d6212
---
# Direct reconcile reads expectations off entry ids

## Hypothesis

H14: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**ACCEPTED** — Quiet-machine re-run: warm-revalidate wall -7.09% with 95% interval
[-8.92%, -5.76%] over 20 paired trials; the first, load-average-17 run was underpowered,
not wrong
