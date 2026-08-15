---
title: Calibrate adaptive workers from initial filesystem service time
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-021
  title: Calibrate adaptive workers from initial filesystem service time
  date: "2026-08-12"
  hypotheses:
    - H31
  checkpoint:
    profile: index-core-v1
    kept_variant: candidate
    source_revision: 2b1f99bba4826f3af2c0d9968f94dd57bcc3a167
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
    control: fixed six-worker automatic pool
    candidate: measure the first 16k entries and expand only above 30 microseconds of worker service per entry
    control_binary:
      name: control
      sha256: be3349ee5238da00b5bce9ff7f72e68fd3fc0a9f96eae16c969c520f0e90977f
      size_bytes: 535968
      args: []
    candidate_binary:
      name: candidate
      sha256: 78ce1157d4ed6d86599b378ef6e831ba437973217280188b99c15975bd3b794f
      size_bytes: 552512
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp021-service-calibration-large.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6300437708.5
          candidate_median: 6056068750.5
          change_pct: -5.313
          ci95_low_pct: -8.371
          ci95_high_pct: -2.704
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 4915391542.0
          candidate_median: 4672508062.5
          change_pct: -7.111
          ci95_low_pct: -11.058
          ci95_high_pct: -2.979
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 22674599500.0
          candidate_median: 34478970500.0
          change_pct: 51.165
          ci95_low_pct: 43.806
          ci95_high_pct: 55.435
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 2953826000.0
          candidate_median: 3328417000.0
          change_pct: 12.925
          ci95_low_pct: 11.133
          ci95_high_pct: 15.105
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 19734928000.0
          candidate_median: 31108980000.0
          change_pct: 56.75
          ci95_low_pct: 48.334
          ci95_high_pct: 60.639
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 325689344.0
          candidate_median: 329728000.0
          change_pct: 1.429
          ci95_low_pct: 0.847
          ci95_high_pct: 1.642
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
          control_median: 12473618083.0
          candidate_median: 11180008396.0
          change_pct: -10.094
          ci95_low_pct: -17.689
          ci95_high_pct: -4.508
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 5465735354.5
          candidate_median: 4912828000.0
          change_pct: -8.264
          ci95_low_pct: -16.26
          ci95_high_pct: -4.221
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 43581295000.0
          candidate_median: 66177877000.0
          change_pct: 44.412
          ci95_low_pct: 35.54
          ci95_high_pct: 52.563
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 3881746000.0
          candidate_median: 4530755500.0
          change_pct: 17.759
          ci95_low_pct: 9.895
          ci95_high_pct: 19.249
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 39797765000.0
          candidate_median: 61602674500.0
          change_pct: 47.378
          ci95_low_pct: 37.698
          ci95_high_pct: 55.581
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 325582848.0
          candidate_median: 331104256.0
          change_pct: 1.627
          ci95_low_pct: 1.504
          ci95_high_pct: 1.931
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 181
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - a noisy first 16k entries can choose the wrong fixed pool for the remainder of one scan
    notes: "No dependency, unsafe code, polling, or per-entry clocks; explicit thread counts remain fixed. Activated scans trade wall latency for 51% more aggregate CPU and 1.43% RSS."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -5.313
    reason: "Latency calibration improved 720k cold-index wall 5.31% and producer wall 10.09%, while the separate 120k boundary left wall, CPU, faults, and RSS unchanged"
    commit: null
---
# Calibrate adaptive workers from initial filesystem service time

## Hypothesis

H31 originally proposed choosing in-flight depth from measured first-operation latency.
Exp-015 proved the worker-count opportunity, while exp-018 through exp-020 showed why
entry count is the wrong selector: it identifies a large tree only after much of the
walk has finished and needlessly activates near a medium tree’s end.
Initial aggregate worker service time should distinguish the measured cached and
cache-pressure states after a small fraction of either tree, independent of eventual
size.

## What was tried

Automatic scans start at the existing conservative count.
The queue aggregates the chunk-level work time already collected for attribution—no
per-entry clocks—and makes one decision after 16,384 successful entries.
An average at or above 30 microseconds per entry sends the existing in-band control
message and expands the pool, bounded by twice available parallelism and sixteen
workers. A faster calibration disables the reserve for the rest of that scan.
Explicit thread counts never calibrate or adapt.

The threshold came from measured whole-run attribution before the code change: about 18
microseconds per entry on the 60k tree, 22 on the 120k boundary, and 42 or more on the
720k cache-pressure control.
Unit tests pin both sides of the one-shot decision, pool bounds, traversal equivalence,
and explicit-count scope semantics.

## What the numbers said

On the immutable 720,805-entry subject, producer wall improved 10.09% [−17.69%, −4.51%]
and component time 8.26%. End-to-end cold-index wall improved 5.31% [−8.37%, −2.70%] and
component time 7.11%. Every oracle check passed and the tree fingerprint remained
stable.

The improvement buys latency with concurrency rather than eliminating work.
Activated cold-index scans used 51.17% more aggregate CPU [+43.81%, +55.44%], 1.43% more
peak RSS, and 1.47% more minor faults.
Those costs are appropriate only in the slow state, which is why the no-activation
evidence is part of the decision.

On the separate immutable 120,135-entry boundary, calibration retained six workers.
Cold-index wall was +0.18% [−3.63%, +1.86%], total CPU −1.46%, peak RSS +0.27%, and
minor faults −0.35%; every interval crossed zero.
Producer wall and resource counters were likewise unclear.
This removes the RSS and fault regressions the 100k scale trigger caused on the same
subject.

## Verdict

**Accepted.** The direct state signal clears the end-to-end bar when it activates and
has no measured cost when it does not.
Its remaining limitation is a one-shot decision: a noisy or unrepresentative first 16k
entries can select the wrong fixed pool for the rest of one scan.
The caller’s explicit thread setting remains the escape hatch and a future controller
may revisit the decision continuously only if evidence justifies the extra coordination.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
