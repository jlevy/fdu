---
title: Adaptive worker threshold at the first crossing scale
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-019
  title: Adaptive worker threshold at the first crossing scale
  date: "2026-08-12"
  hypotheses:
    - H31
  subject:
    tree_label: threshold-boundary-2x
    tree_root_id: 377b56b314cb17ac9c2a01a84be98bc75bda3944893db80c36a28d0a1cd39acb
    tree_engine_digest: c3bede368a3f64ee946a26282ac4158b207225a33fd2e7208e73ccfc00967366
    tree_entries: 120135
    tree_directories: 14701
    tree_files: 105390
    tree_symlinks: 44
    tree_apparent_bytes: 2170167344
    tree_allocated_bytes: 2460147712
    tree_max_depth: 20
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
    control: fixed six-worker automatic pool
    candidate: spawn ten reserve workers after 100k observed entries
    control_binary:
      name: control
      sha256: be3349ee5238da00b5bce9ff7f72e68fd3fc0a9f96eae16c969c520f0e90977f
      size_bytes: 535968
      args: []
    candidate_binary:
      name: candidate
      sha256: ca4c8918a82cd40c239f2bfcf9ca36c7bb9390f147a9b41d66b2e63fb250dd2c
      size_bytes: 552512
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp019-threshold-boundary.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 634205667.0
          candidate_median: 637465479.5
          change_pct: 1.23
          ci95_low_pct: -1.846
          ci95_high_pct: 3.803
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 408111916.5
          candidate_median: 407247146.0
          change_pct: 0.809
          ci95_low_pct: -3.193
          ci95_high_pct: 4.293
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 2622272000.0
          candidate_median: 2701135000.0
          change_pct: 1.704
          ci95_low_pct: -2.051
          ci95_high_pct: 8.042
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 491823500.0
          candidate_median: 498280000.0
          change_pct: 3.386
          ci95_low_pct: 0.095
          ci95_high_pct: 5.981
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 2161292000.0
          candidate_median: 2213647500.0
          change_pct: 0.949
          ci95_low_pct: -2.825
          ci95_high_pct: 9.123
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 63094784.0
          candidate_median: 63971328.0
          change_pct: 1.638
          ci95_low_pct: 0.523
          ci95_high_pct: 2.322
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
          control_median: 1096734479.5
          candidate_median: 1090024999.5
          change_pct: -2.682
          ci95_low_pct: -6.417
          ci95_high_pct: 3.529
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 439300437.0
          candidate_median: 432510979.0
          change_pct: -1.259
          ci95_low_pct: -8.61
          ci95_high_pct: 8.227
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 4649573500.0
          candidate_median: 4647513500.0
          change_pct: 4.163
          ci95_low_pct: -1.801
          ci95_high_pct: 6.383
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 658103000.0
          candidate_median: 670890500.0
          change_pct: 1.258
          ci95_low_pct: -1.095
          ci95_high_pct: 3.776
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 3993390000.0
          candidate_median: 3971710000.0
          change_pct: 5.064
          ci95_low_pct: -2.153
          ci95_high_pct: 7.035
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 63176704.0
          candidate_median: 65273856.0
          change_pct: 3.23
          ci95_low_pct: 2.848
          ci95_high_pct: 3.727
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: Boundary validation of exp-018; raise the trigger toward metadata-cache capacity and remeasure both endpoints.
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 1.23
    reason: "At 120k entries the threshold added no wall benefit but measurably regressed RSS and minor faults, so 100k activates before enough work remains"
    commit: null
---
# Adaptive worker threshold at the first crossing scale

## Hypothesis

H31’s 100,000-entry trigger passed its 60k and 720k endpoints, but a threshold policy
also needs the first scale that crosses it.
A 120k tree leaves only about one sixth of the walk after activation.
If thread creation is truly conditional on useful remaining work, wall and resource
counters should remain neutral there.

## What was tried

The exp-018 binary and its fixed-six control ran unchanged on an immutable 120,135-entry
subject made from two APFS clones of the pinned reference tree.
Twelve measured pairs followed three warmups for producer-only and end-to-end index
jobs. This was a boundary validation, not another implementation.

## What the numbers said

Cold-index wall was +1.23% [−1.85%, +3.80%] and producer wall −2.68% [−6.42%, +3.53%]:
neither showed a benefit.
The cost did show. Cold-index peak RSS regressed 1.64% [+0.52%, +2.32%], minor faults
1.55%, and user CPU 3.39%. Producer peak RSS regressed 3.23% [+2.85%, +3.73%] and minor
faults 3.84%. The reserves were created near completion, performed too little useful
work to move wall, and still enlarged the process.

## Verdict

**Rejected.** This supersedes exp-018’s production decision.
Raising the scale trigger was tested next; the durable lesson is that total scale is
only a proxy for the state that actually matters: filesystem service latency.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
