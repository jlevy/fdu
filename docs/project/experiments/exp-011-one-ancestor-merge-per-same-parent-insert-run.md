---
title: One ancestor merge per same-parent insert run
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-011
  title: One ancestor merge per same-parent insert run
  date: 2026-08-11
  hypotheses:
    - H13
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
    control: exp-010 build
    candidate: consecutive same-parent inserts accumulate contributions locally; one upward merge per run instead of per file
    control_binary:
      name: h17
      sha256: cd39db9416cc9356bb75664f4bfdc65f395f4eef8e93426189e7f7116fe2dba2
      size_bytes: 519328
      args: []
    candidate_binary:
      name: h17h13
      sha256: 2fd4e35e59f9ab4532822b24860a2ff55ab94d1628344ad392cc4cb206c2457f
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
          control_median: 483088125.0
          candidate_median: 447694292.0
          change_pct: -2.531
          ci95_low_pct: -8.387
          ci95_high_pct: 0.226
          significant: false
          pairs: 16
        component_ns:
          control_median: 304109312.5
          candidate_median: 282263958.5
          change_pct: -7.008
          ci95_low_pct: -11.451
          ci95_high_pct: -1.626
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 1256933500.0
          candidate_median: 1203976000.0
          change_pct: -2.137
          ci95_low_pct: -5.421
          ci95_high_pct: -0.294
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 282123500.0
          candidate_median: 271561500.0
          change_pct: -3.66
          ci95_low_pct: -4.953
          ci95_high_pct: -1.615
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 968667000.0
          candidate_median: 933441000.0
          change_pct: -1.831
          ci95_low_pct: -6.624
          ci95_high_pct: 0.464
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
          control_median: 34553856.0
          candidate_median: 34185216.0
          change_pct: -1.474
          ci95_low_pct: -2.223
          ci95_high_pct: 0.143
          significant: false
          pairs: 16
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 572710583.5
          candidate_median: 572943271.0
          change_pct: -0.258
          ci95_low_pct: -3.818
          ci95_high_pct: 6.187
          significant: false
          pairs: 16
        component_ns:
          control_median: 239539187.5
          candidate_median: 240409916.5
          change_pct: -2.656
          ci95_low_pct: -6.693
          ci95_high_pct: 0.182
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 2387416000.0
          candidate_median: 2335741000.0
          change_pct: -2.896
          ci95_low_pct: -3.94
          ci95_high_pct: -0.733
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 360182500.0
          candidate_median: 351324000.0
          change_pct: -2.522
          ci95_low_pct: -3.543
          ci95_high_pct: -1.671
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 2027492500.0
          candidate_median: 1986882500.0
          change_pct: -2.74
          ci95_low_pct: -4.191
          ci95_high_pct: -0.436
          significant: true
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
          control_median: 34357248.0
          candidate_median: 34570240.0
          change_pct: 0.001
          ci95_low_pct: -0.798
          ci95_high_pct: 2.477
          significant: false
          pairs: 16
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 695578417.0
          candidate_median: 695529125.5
          change_pct: -0.706
          ci95_low_pct: -1.036
          ci95_high_pct: -0.117
          significant: true
          pairs: 16
        component_ns:
          control_median: 455838125.0
          candidate_median: 455347541.5
          change_pct: -0.988
          ci95_low_pct: -1.587
          ci95_high_pct: -0.478
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 691180000.0
          candidate_median: 688390000.0
          change_pct: -0.717
          ci95_low_pct: -1.014
          ci95_high_pct: -0.462
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 276402000.0
          candidate_median: 275189000.0
          change_pct: -0.365
          ci95_low_pct: -0.735
          ci95_high_pct: -0.038
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 415225500.0
          candidate_median: 413273000.0
          change_pct: -0.939
          ci95_low_pct: -1.48
          ci95_high_pct: -0.469
          significant: true
          pairs: 16
        blocked_ns:
          control_median: 4525146.0
          candidate_median: 4752416.5
          change_pct: 1.893
          ci95_low_pct: -18.91
          ci95_high_pct: 25.151
          significant: false
          pairs: 16
        peak_rss_bytes:
          control_median: 32079872.0
          candidate_median: 32063488.0
          change_pct: 0.0
          ci95_low_pct: -0.457
          ci95_high_pct: 1.466
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
    lines_changed: 80
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Reverted. The interaction is the finding: merge-batching and key-interning compete for the same cost, and interning alone captured it. Re-test only if by_ext grows heavy again (content-tier reducers)."
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -2.531
    reason: "Direction right but under the bar: -2.53% [-8.39%, +0.23%] on cold scan; H18 already removed the expensive part of each merge, so cutting ~520k merges to ~73k amortized work that had become a few integer adds"
    commit: null
---
# One ancestor merge per same-parent insert run

## Hypothesis

H13: _state what you expected to be slow, why,
and which metric would move._

## What was tried

_The smallest change that tests the hypothesis._

## What the numbers said

_Read the tables in the frontmatter. Say what surprised you._

## Verdict

**REJECTED** — Direction right but under the bar: -2.53% [-8.39%, +0.23%] on cold scan; H18 already removed the expensive part of each merge, so cutting ~520k merges to ~73k amortized work that had become a few integer adds
