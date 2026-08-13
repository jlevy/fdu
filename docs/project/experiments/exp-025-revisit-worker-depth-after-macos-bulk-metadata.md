---
title: Revisit worker depth after macOS bulk metadata
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-025
  title: Revisit worker depth after macOS bulk metadata
  date: 2026-08-12
  hypotheses:
    - H52
  subject:
    tree_label: cache-pressure-12x
    tree_root_id: ffd40fd8482e8ed64bd19bcd1a724389532ca4889be43adf830122279ac63180
    tree_engine_digest: f2909250591b9b64d98956b0b2d8a9c3bd588b4c23f046a4660f3f174173dc23
    tree_entries: 720805
    tree_directories: 88201
    tree_files: 632340
    tree_symlinks: 264
    tree_apparent_bytes: 13021004064
    tree_allocated_bytes: 14760886272
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
    control: six fixed workers on the exp-022 bulk backend
    candidate: the pre-H26 sixteen-worker large-tree target
    control_binary:
      name: t6
      sha256: 52e0b303402ac0eafa11b06013b731126d81bef482acc962cca3ad9fa2ebc879
      size_bytes: 552576
      args:
        - "--threads"
        - "6"
    candidate_binary:
      name: t16
      sha256: 52e0b303402ac0eafa11b06013b731126d81bef482acc962cca3ad9fa2ebc879
      size_bytes: 552576
      args:
        - "--threads"
        - "16"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp025-post-bulk-thread-depth-large-final.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3700514833.0
          candidate_median: 4374272937.5
          change_pct: 19.193
          ci95_low_pct: 11.809
          ci95_high_pct: 25.0
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        component_ns:
          control_median: 2338441291.5
          candidate_median: 2987021041.0
          change_pct: 28.391
          ci95_low_pct: 18.741
          ci95_high_pct: 38.002
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        cpu_ns:
          control_median: 11127563500.0
          candidate_median: 23309298000.0
          change_pct: 107.021
          ci95_low_pct: 98.703
          ci95_high_pct: 122.91
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 2443067500.0
          candidate_median: 2730024500.0
          change_pct: 11.823
          ci95_low_pct: 9.832
          ci95_high_pct: 14.01
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 8574582500.0
          candidate_median: 20601604500.0
          change_pct: 135.382
          ci95_low_pct: 125.96
          ci95_high_pct: 153.352
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 326819840.0
          candidate_median: 436051968.0
          change_pct: 33.463
          ci95_low_pct: 31.529
          ci95_high_pct: 35.571
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
          control_median: 6260263292.0
          candidate_median: 6920151458.5
          change_pct: 12.65
          ci95_low_pct: 5.991
          ci95_high_pct: 14.583
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        component_ns:
          control_median: 2375552104.0
          candidate_median: 2641207958.5
          change_pct: 11.518
          ci95_low_pct: 3.82
          ci95_high_pct: 14.344
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        cpu_ns:
          control_median: 20477423000.0
          candidate_median: 44000079500.0
          change_pct: 117.043
          ci95_low_pct: 104.29
          ci95_high_pct: 128.991
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 2894791000.0
          candidate_median: 3294883500.0
          change_pct: 14.63
          ci95_low_pct: 10.688
          ci95_high_pct: 16.083
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 17617898000.0
          candidate_median: 40705196000.0
          change_pct: 133.891
          ci95_low_pct: 119.18
          ci95_high_pct: 148.51
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 327163904.0
          candidate_median: 435372032.0
          change_pct: 32.903
          ci95_low_pct: 28.271
          ci95_high_pct: 41.358
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
    notes: "configuration-only reproduction using one exact binary; no code, dependency, or unsafe change"
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 19.193
    reason: "Sixteen workers regressed 720k indexed wall 19.19% and producer wall 12.65%, while roughly doubling CPU and adding about one-third RSS; the exploratory 6/8/12/16 curve found no smaller retune above the acceptance bar"
    commit: null
---
# Revisit worker depth after macOS bulk metadata

## Hypothesis

H52 revisited H31 after H26 changed the unit of filesystem work.
Sixteen workers were the measured knee when each of 720,805 entries incurred a separate
metadata syscall, but `getattrlistbulk` now amortizes that wait across a directory.
Six workers should therefore match or beat deeper pools on the same cache-pressure tree
while using less CPU and memory, and the existing service-time trigger should remain
below its scale-up threshold.

## What was tried

An exploratory interleaved curve ran the exact exp-022 binary at fixed depths of 6, 8,
12, and 16 workers. Explicit counts disable adaptation, so the run isolated pool depth
without changing code.
Six and eight were neutral for indexed and producer scans; twelve offered no
improvement; sixteen was visibly slower.
The consequential old-versus- new comparison was then repeated with twelve paired trials
after three warmups on the immutable 720,805-entry APFS subject.

The automatic configuration was also invoked directly on the same tree.
Its aggregate worker work was 14.84 seconds over a 2.48-second component--approximately
six active workers--and 20.6 microseconds per entry, below the pre-registered
30-microsecond H31 scale-up threshold.
H26 therefore makes the current trigger retain the conservative pool without another
policy branch.

## What the numbers said

Against six workers, the old sixteen-worker target regressed end-to-end cold-index wall
19.19% [+11.81%, +25.00%] and its component 28.39%. Total CPU rose 107.02%, system CPU
135.38%, peak RSS 33.46%, minor faults 34.26%, and involuntary context switches 206.91%.

Producer-only wall regressed 12.65% [+5.99%, +14.58%] and its scan component 11.52%. Its
total CPU rose 117.04%, system CPU 133.89%, peak RSS 32.90%, and context switches
165.22%. Every sample passed the independent oracle and the tree remained unchanged.

The exploratory curve also rejected a tempting smaller retune: eight versus six was
-0.65% [-9.42%, +6.43%] for indexed wall and -2.47% [-11.07%, +1.57%] for producer wall,
below the acceptance threshold with both intervals crossing zero.

## Verdict

**Rejected the deeper worker target; confirmed H52.** The breadth-first region scheduler
still scales correctly, but the optimum depends on the syscall backend.
`getattrlistbulk` removed the latency that justified sixteen workers, and the existing
service-time calibration automatically stays at six.
No code change is needed.
The pre-bulk H31 result remains valid for the portable high-latency path; it must not be
generalized to the macOS bulk backend.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
