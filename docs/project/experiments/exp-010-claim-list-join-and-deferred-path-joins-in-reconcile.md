---
title: Claim-list join and deferred path joins in reconcile
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-010
  title: Claim-list join and deferred path joins in reconcile
  date: 2026-08-11
  hypotheses:
    - H17
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
    warmups: 3
    interleaved: true
    control: "HEAD eb1c884: per-directory BTreeMap of child expectations, PathBuf join per entry"
    candidate: sorted claim-list with binary search; the path join deferred until an op or descent needs it
    control_binary:
      name: control
      sha256: eb25a8f293b17f8de9f481c2ca17790f7fd086196375e97d74eb9f4848ef8d2d
      size_bytes: 519328
      args: []
    candidate_binary:
      name: h17
      sha256: cd39db9416cc9356bb75664f4bfdc65f395f4eef8e93426189e7f7116fe2dba2
      size_bytes: 519328
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp010-011-warm-join-and-grouped-merges.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 463034854.0
          candidate_median: 483088125.0
          change_pct: 1.497
          ci95_low_pct: -0.999
          ci95_high_pct: 4.055
          significant: false
          pairs: 16
        component_ns:
          control_median: 292152187.0
          candidate_median: 304109312.5
          change_pct: 1.406
          ci95_low_pct: -1.778
          ci95_high_pct: 5.383
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 1211369500.0
          candidate_median: 1256933500.0
          change_pct: 1.755
          ci95_low_pct: -2.056
          ci95_high_pct: 6.345
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 281023000.0
          candidate_median: 282123500.0
          change_pct: 0.47
          ci95_low_pct: -1.865
          ci95_high_pct: 2.346
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 928885000.0
          candidate_median: 968667000.0
          change_pct: 3.01
          ci95_low_pct: -2.417
          ci95_high_pct: 7.675
          significant: false
          pairs: 16
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 34742272.0
          candidate_median: 34553856.0
          change_pct: -0.845
          ci95_low_pct: -2.197
          ci95_high_pct: 0.048
          significant: false
          pairs: 16
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 576702833.5
          candidate_median: 572710583.5
          change_pct: -0.928
          ci95_low_pct: -3.213
          ci95_high_pct: 3.498
          significant: false
          pairs: 16
        component_ns:
          control_median: 234121666.5
          candidate_median: 239539187.5
          change_pct: 3.409
          ci95_low_pct: -1.285
          ci95_high_pct: 9.778
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 2376804000.0
          candidate_median: 2387416000.0
          change_pct: 0.678
          ci95_low_pct: -2.916
          ci95_high_pct: 2.48
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 358365500.0
          candidate_median: 360182500.0
          change_pct: 0.652
          ci95_low_pct: -1.208
          ci95_high_pct: 2.203
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 2015262000.0
          candidate_median: 2027492500.0
          change_pct: 0.451
          ci95_low_pct: -3.802
          ci95_high_pct: 3.31
          significant: false
          pairs: 16
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 34922496.0
          candidate_median: 34357248.0
          change_pct: -1.713
          ci95_low_pct: -2.415
          ci95_high_pct: 0.0
          significant: false
          pairs: 16
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 698540250.0
          candidate_median: 695578417.0
          change_pct: -0.031
          ci95_low_pct: -1.37
          ci95_high_pct: 1.638
          significant: false
          pairs: 16
        component_ns:
          control_median: 458570896.0
          candidate_median: 455838125.0
          change_pct: -0.104
          ci95_low_pct: -1.953
          ci95_high_pct: 1.505
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 694326500.0
          candidate_median: 691180000.0
          change_pct: -0.053
          ci95_low_pct: -1.325
          ci95_high_pct: 1.139
          significant: false
          pairs: 16
        user_cpu_ns:
          control_median: 280121000.0
          candidate_median: 276402000.0
          change_pct: -1.405
          ci95_low_pct: -1.807
          ci95_high_pct: -0.275
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 414367500.0
          candidate_median: 415225500.0
          change_pct: 1.033
          ci95_low_pct: -0.999
          ci95_high_pct: 2.193
          significant: false
          pairs: 16
        blocked_ns:
          control_median: 4507708.5
          candidate_median: 4525146.0
          change_pct: 4.066
          ci95_low_pct: -14.304
          ci95_high_pct: 18.674
          significant: false
          pairs: 16
        peak_rss_bytes:
          control_median: 31965184.0
          candidate_median: 32079872.0
          change_pct: 0.538
          ci95_low_pct: -0.997
          ci95_high_pct: 1.87
          significant: false
          pairs: 16
  reference_tools:
    - name: dust
      wall_ns_median: 286187687.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 90
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Reverted. Useful negative: the portable warm path's userland slack is spent - what remains is the syscall floor (blocked on the rustix decision) and the structural H12 parallel form."
  verdict:
    decision: rejected
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -0.031
    reason: "Nothing there: -0.03% with a tight interval [-1.37%, +1.64%] over 16 quiet paired trials; after H14 the expectation map already read straight off entry ids, and the remaining allocations are noise next to one fstatat per entry"
    commit: null
---
# Claim-list join and deferred path joins in reconcile

## Hypothesis

H17: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**REJECTED** — Nothing there: -0.03% with a tight interval [-1.37%, +1.64%] over 16
quiet paired trials; after H14 the expectation map already read straight off entry ids,
and the remaining allocations are noise next to one fstatat per entry
