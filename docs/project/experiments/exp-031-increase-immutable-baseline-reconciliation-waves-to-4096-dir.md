---
title: Increase immutable-baseline reconciliation waves to 4096 directories
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-031
  title: Increase immutable-baseline reconciliation waves to 4096 directories
  date: "2026-08-12"
  hypotheses:
    - H56
  checkpoint:
    profile: index-core-v1
    kept_variant: control
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
    control: exp-030 1024-directory immutable-baseline waves
    candidate: 4096-directory immutable-baseline waves
    control_binary:
      name: control
      sha256: 54db14278796b5ab1233ed71eefe07e2061c3913a957ac8e4f5fa79a8a4c2765
      size_bytes: 585680
      args: []
    candidate_binary:
      name: candidate
      sha256: bd3efd16e2c9dc6ca8f5b6271454a4121464c3740b0a1a66a3193daebd5cd82b
      size_bytes: 585680
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp031-larger-reconciliation-wave-small-final.json
  results:
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 477585979.0
          candidate_median: 482463500.5
          change_pct: 1.642
          ci95_low_pct: -3.885
          ci95_high_pct: 10.067
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 194151375.0
          candidate_median: 220191687.5
          change_pct: 13.239
          ci95_low_pct: -0.471
          ci95_high_pct: 19.562
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 866928500.0
          candidate_median: 884929500.0
          change_pct: 4.873
          ci95_low_pct: -2.168
          ci95_high_pct: 10.101
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 277451500.0
          candidate_median: 275492000.0
          change_pct: 1.332
          ci95_low_pct: -3.382
          ci95_high_pct: 5.166
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 585478500.0
          candidate_median: 613649500.0
          change_pct: 5.338
          ci95_low_pct: -1.084
          ci95_high_pct: 13.88
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 33923072.0
          candidate_median: 33783808.0
          change_pct: -0.462
          ci95_low_pct: -2.056
          ci95_high_pct: 1.402
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 1
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - effective changed-tree deltas may wait behind four times as many directory reads
    notes: one constant change; no dependency or unsafe change; the preregistered 720k gate was not triggered
  verdict:
    decision: rejected
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: 1.642
    reason: "Warm wall was +1.64% with an interval spanning -3.88% to +10.07%, component was +13.24%, and CPU plus context-switch signals did not show the predicted startup amortization"
    commit: null
---
# Increase immutable-baseline reconciliation waves to 4,096 directories

## Hypothesis

H56 followed exp-030’s post-change profile, where scoped thread startup, waiting, and
thread entry frames together accounted for about 13% of 60k warm samples.
Increasing the immutable-baseline wave from 1,024 to 4,096 directories should amortize
worker creation while retaining the four-worker depth, deferred-operation bound, exact
comparison, and delta-only apply path.

The pre-registered 60k gate required warm wall or reconciliation component time to
improve at least 3% with its confidence interval below zero, no more than 5% additional
RSS, and exact oracle parity.
A 720k confirmation would run only if that gate passed.

## What was tried

One constant increased the maximum directories compared before a wave joins and applies
effective changes. No worker, syscall, parser, index operation, dependency, or unsafe
boundary changed. The exact accepted exp-030 binary and candidate ran twelve interleaved
pairs after three warmups on the immutable 60,067-entry APFS subject.

The candidate remained bounded by the same 65,536 deferred changes, but a changed-tree
delta could wait behind four times as many directory reads.
That latency cost required a clear throughput result rather than a within-noise change.

## What the numbers said

Warm-open wall was +1.64% [-3.88%, +10.07%] and reconciliation component time was
+13.24% [-0.47%, +19.56%]. Total CPU was +4.87%, system CPU +5.34%, and involuntary
context switches -4.59%; every interval included zero.
RSS was neutral at -0.46%. Every sample passed the exact oracle and the tree remained
unchanged.

The larger unit did not reduce a measured coordination counter and lengthened the
component median. The thread-related profile frames were therefore not evidence that
worker creation lay on the critical wall-time path.
Coarser waves also give the region-aware frontier fewer apply/scheduling boundaries, so
carrying them would trade progress latency for no demonstrated throughput benefit.

## Verdict

**Rejected and reverted.** Neither registered performance signal improved, and the
component moved in the wrong direction.
The 720k run was not triggered.
The accepted 1,024-directory wave remains the measured balance between amortization,
region scheduling, and progressive delta delivery.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
