---
title: Skip journalling on the bootstrap apply path
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-003
  title: Skip journalling on the bootstrap apply path
  date: "2026-08-11"
  hypotheses:
    - H8
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
    control: "HEAD a0cc981, bootstrap routed through the live journalling apply"
    candidate: "bootstrap applies directly, with no effective-op list and no journal entry"
    control_binary:
      name: control
      sha256: 80e6049b1259dc6c7a6438a6c684a668c096fdd298d9aaf6bd1417ae6c0b820c
      size_bytes: 519328
      args: []
    candidate_binary:
      name: candidate
      sha256: 85013f4a5510af26df54b2257e3c192f61004841864f1f7a08ee27a291153fdb
      size_bytes: 519328
      args: []
    toolchain: ""
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp003-baseline-apply.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 309721687.5
          candidate_median: 308745708.5
          change_pct: 1.019
          ci95_low_pct: -3.007
          ci95_high_pct: 4.886
          significant: false
          pairs: 14
        component_ns:
          control_median: 194477562.5
          candidate_median: 193946062.5
          change_pct: 2.48
          ci95_low_pct: -5.005
          ci95_high_pct: 8.527
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 1283518000.0
          candidate_median: 1258571500.0
          change_pct: -1.009
          ci95_low_pct: -5.475
          ci95_high_pct: 4.703
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 282472500.0
          candidate_median: 270254500.0
          change_pct: -3.274
          ci95_low_pct: -4.134
          ci95_high_pct: -2.46
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 1002217000.0
          candidate_median: 978143500.0
          change_pct: -0.257
          ci95_low_pct: -5.665
          ci95_high_pct: 7.384
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 36806656.0
          candidate_median: 35422208.0
          change_pct: -3.973
          ci95_low_pct: -4.806
          ci95_high_pct: -2.644
          significant: true
          pairs: 14
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 350829292.0
          candidate_median: 346245270.5
          change_pct: -4.923
          ci95_low_pct: -5.845
          ci95_high_pct: 0.724
          significant: false
          pairs: 14
        component_ns:
          control_median: 35359917.0
          candidate_median: 35082833.5
          change_pct: -4.32
          ci95_low_pct: -9.87
          ci95_high_pct: 5.891
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 1312993000.0
          candidate_median: 1297794500.0
          change_pct: -1.053
          ci95_low_pct: -5.247
          ci95_high_pct: 3.983
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 302325000.0
          candidate_median: 290128000.0
          change_pct: -4.287
          ci95_low_pct: -6.927
          ci95_high_pct: -2.979
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 992990500.0
          candidate_median: 1003201500.0
          change_pct: 0.855
          ci95_low_pct: -5.532
          ci95_high_pct: 7.07
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 44998656.0
          candidate_median: 43507712.0
          change_pct: -2.571
          ci95_low_pct: -4.534
          ci95_high_pct: -1.726
          significant: true
          pairs: 14
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 802365270.5
          candidate_median: 780684583.0
          change_pct: -2.298
          ci95_low_pct: -2.907
          ci95_high_pct: -1.783
          significant: true
          pairs: 14
        component_ns:
          control_median: 476291645.5
          candidate_median: 472175396.0
          change_pct: -0.873
          ci95_low_pct: -1.558
          ci95_high_pct: 0.25
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 797225000.0
          candidate_median: 774276500.0
          change_pct: -2.194
          ci95_low_pct: -3.13
          ci95_high_pct: -1.787
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 426676000.0
          candidate_median: 407473500.0
          change_pct: -4.114
          ci95_low_pct: -4.689
          ci95_high_pct: -3.562
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 370571500.0
          candidate_median: 368179500.0
          change_pct: -0.16
          ci95_low_pct: -1.341
          ci95_high_pct: 0.822
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 6427729.0
          candidate_median: 6386417.0
          change_pct: -3.995
          ci95_low_pct: -12.077
          ci95_high_pct: 6.535
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 33898496.0
          candidate_median: 33783808.0
          change_pct: -0.557
          ci95_low_pct: -1.724
          ci95_high_pct: 0.0
          significant: false
          pairs: 14
  reference_tools:
    - name: dust
      wall_ns_median: 226252833.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 32
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - a second copy of the arbitration loop to keep in step with the live one
    notes: "Reverted. Useful negative result: the allocator cost the profile showed is in the producer, not in apply. That is what redirected exp-004 at normalize() rather than at the journal."
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 1.019
    reason: "Removing roughly 120,000 path clones per scan produced no measurable change, so a duplicated arbitration loop was not worth carrying"
    commit: null
---
# Skip journalling on the bootstrap apply path

## Hypothesis

H8: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**REJECTED** — Removing roughly 120,000 path clones per scan produced no measurable
change, so a duplicated arbitration loop was not worth carrying
