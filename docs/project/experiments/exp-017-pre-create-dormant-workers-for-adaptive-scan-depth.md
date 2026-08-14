---
title: Pre-create dormant workers for adaptive scan depth
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-017
  title: Pre-create dormant workers for adaptive scan depth
  date: "2026-08-12"
  hypotheses:
    - H31
  subject:
    tree_label: metabrowser
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
    control: fixed six-worker automatic pool
    candidate: six active workers plus ten dormant reserve threads
    control_binary:
      name: control
      sha256: be3349ee5238da00b5bce9ff7f72e68fd3fc0a9f96eae16c969c520f0e90977f
      size_bytes: 535968
      args: []
    candidate_binary:
      name: candidate
      sha256: c7912bad3d33911fdd9536c58fe8627b0c411ef04cbb6e661caf595a4bab9b62
      size_bytes: 552480
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp017-adaptive-workers-small.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 338600521.0
          candidate_median: 330019395.5
          change_pct: -2.191
          ci95_low_pct: -12.303
          ci95_high_pct: 1.928
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 222732604.0
          candidate_median: 214103854.0
          change_pct: -2.314
          ci95_low_pct: -11.282
          ci95_high_pct: 4.762
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1251395500.0
          candidate_median: 1235759500.0
          change_pct: 3.006
          ci95_low_pct: -8.702
          ci95_high_pct: 9.418
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 248475500.0
          candidate_median: 251317000.0
          change_pct: 1.202
          ci95_low_pct: -0.085
          ci95_high_pct: 4.131
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 1006034500.0
          candidate_median: 983255000.0
          change_pct: 4.07
          ci95_low_pct: -11.343
          ci95_high_pct: 10.741
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 34013184.0
          candidate_median: 34308096.0
          change_pct: 0.532
          ci95_low_pct: -0.265
          ci95_high_pct: 2.181
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 494171521.0
          candidate_median: 500703542.0
          change_pct: 2.009
          ci95_low_pct: -1.856
          ci95_high_pct: 5.594
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 204620542.0
          candidate_median: 207694187.5
          change_pct: 0.879
          ci95_low_pct: -5.318
          ci95_high_pct: 5.071
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 2041141000.0
          candidate_median: 2116881000.0
          change_pct: 5.668
          ci95_low_pct: 1.33
          ci95_high_pct: 8.441
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 307869000.0
          candidate_median: 320294500.0
          change_pct: 3.333
          ci95_low_pct: 2.468
          ci95_high_pct: 5.026
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 1733506500.0
          candidate_median: 1791589000.0
          change_pct: 5.571
          ci95_low_pct: 0.947
          ci95_high_pct: 9.406
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 34078720.0
          candidate_median: 34627584.0
          change_pct: 1.4
          ci95_low_pct: 0.527
          ci95_high_pct: 2.769
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 126
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - reserve-thread wakeup must stay synchronized with queue completion
    notes: Superseded in the working tree by threshold-triggered spawning; no dormant reserve is needed.
  verdict:
    decision: rejected
    primary_job: cold-scan-producer
    primary_metric: wall_ns
    change_pct: 2.009
    reason: "Small-tree wall was unclear, but dormant reserves measurably added 5.67% CPU, 2.38% minor faults, and 1.40% peak RSS before ever activating"
    commit: null
---
# Pre-create dormant workers for adaptive scan depth

## Hypothesis

Exp-015 showed that sixteen active workers help under cache pressure and hurt the fully
warm 60k case. H31’s first implementation therefore started six workers and pre-created
ten reserve threads blocked on the queue until the walk crossed 100,000 observed
entries. Because the reference tree has only 60,067 entries, the reserves could never
activate there; small-tree wall, CPU, faults, and RSS should all have matched the
six-worker control.

## What was tried

The automatic pool resolved to an initial and maximum worker count.
All maximum workers were spawned with the scan, but reserve workers waited outside the
attribution timer until queue accounting crossed the threshold.
Explicit thread counts remained fixed.
Two queue tests pinned the automatic bounds and threshold transition.

## What the numbers said

Small-tree wall did remain unclear: +2.01% [−1.86%, +5.59%] for the producer and −2.19%
[−12.30%, +1.93%] for cold-index.
The reserve was not free, however.
Producer CPU regressed 5.67% [+1.33%, +8.44%], minor faults 2.38% [+1.12%, +3.50%], and
peak RSS 1.40% [+0.53%, +2.77%]. Cold-index resource intervals were less conclusive, but
there is no reason to pay a clear setup cost in the isolating producer job when no
reserve worker performs scan work.

## Verdict

**Rejected.** Reserve workers should be created only when the threshold is crossed.
An in-band control message can carry the last live channel sender to the consumer, which
can then spawn the additional scoped workers without polling or keeping a small scan’s
channel artificially open.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
