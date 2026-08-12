---
title: Spawn reserve workers only after observed scan scale
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-018
  title: Spawn reserve workers only after observed scan scale
  date: 2026-08-12
  hypotheses:
    - H31
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
    candidate: start at six and spawn up to sixteen after 100k observed entries
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
    run_artifact: benchmarks/results/realtree/run-exp018-threshold-spawn-large.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6517919104.5
          candidate_median: 6238313333.5
          change_pct: -4.043
          ci95_low_pct: -5.557
          ci95_high_pct: -1.853
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 5148859708.5
          candidate_median: 4854206729.5
          change_pct: -5.425
          ci95_low_pct: -7.172
          ci95_high_pct: -2.892
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 22952032000.0
          candidate_median: 32294318500.0
          change_pct: 41.241
          ci95_low_pct: 37.729
          ci95_high_pct: 44.258
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 2951298000.0
          candidate_median: 3368831000.0
          change_pct: 13.843
          ci95_low_pct: 11.404
          ci95_high_pct: 14.507
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 20003220500.0
          candidate_median: 28918405500.0
          change_pct: 45.115
          ci95_low_pct: 41.536
          ci95_high_pct: 48.815
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
          control_median: 324935680.0
          candidate_median: 330244096.0
          change_pct: 1.561
          ci95_low_pct: 1.31
          ci95_high_pct: 1.786
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
          control_median: 11651661083.0
          candidate_median: 10840229708.0
          change_pct: -4.634
          ci95_low_pct: -7.922
          ci95_high_pct: 0.62
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 5149887958.0
          candidate_median: 4644860812.5
          change_pct: -5.025
          ci95_low_pct: -10.391
          ci95_high_pct: -1.617
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 43076638500.0
          candidate_median: 64502789500.0
          change_pct: 47.423
          ci95_low_pct: 39.548
          ci95_high_pct: 62.414
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 3794215500.0
          candidate_median: 4472883500.0
          change_pct: 17.767
          ci95_low_pct: 16.302
          ci95_high_pct: 22.799
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 39301622500.0
          candidate_median: 59992761000.0
          change_pct: 50.444
          ci95_low_pct: 41.918
          ci95_high_pct: 65.632
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
          control_median: 325640192.0
          candidate_median: 330260480.0
          change_pct: 1.378
          ci95_low_pct: 1.276
          ci95_high_pct: 1.472
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 128
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - a threshold control message must create the reserve exactly once without extending channel lifetime
    notes: "No dependency or unsafe code; explicit thread counts remain fixed. Large-tree latency trades for roughly 41% more CPU and 1.56% RSS."
  verdict:
    decision: superseded
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -4.043
    reason: "The 720k endpoint passed, but exp-019 found needless RSS and fault regressions just above the 100k trigger; service-time calibration supersedes scale alone"
    commit: null
---
# Spawn reserve workers only after observed scan scale

## Hypothesis

H31's explicit configuration run established two different knees: six workers for the
warm-small tree and sixteen under metadata-cache pressure. Exp-017 preserved the first
count but paid to pre-create the second, adding measurable small-tree resource cost.
Creating reserve workers only when an active walk crosses 100,000 observed entries
should leave the 60k execution path identical while recovering a meaningful portion of
the 720k latency gain.

## What was tried

An automatic scan now resolves an initial and maximum pool. On this ten-core host those
counts are six and sixteen; smaller hosts cap the reserve at twice their available
parallelism, and explicit `threads` values remain fixed. The queue counts successful
entries as claimed directory chunks finish. Its threshold-crossing worker sends one
in-band control message carrying a live observation-channel sender. The consumer uses
that sender to create the additional scoped workers, then drops it.

This shape matters for the small case: no reserve thread, polling loop, retained sender,
dependency, or unsafe block exists before the threshold. Unit tests pin the pool bounds,
single threshold transition, explicit-count semantics, traversal equivalence, and queue
scheduling. The benchmark probe's independent digest continued to validate every trial.

## What the numbers said

On the 720,805-entry cache-pressure corpus, cold-index wall improved 4.04%
[−5.56%, −1.85%] and its scan component improved 5.43%
[−7.17%, −2.89%]. Producer component improved 5.03%
[−10.39%, −1.62%]; whole producer wall had the same −4.63% median but a wider
[−7.92%, +0.62%] interval because that job includes an untimed validation scan in
its process wall.

The adaptive cold-index median, 6.238 seconds, essentially matched explicit sixteen
workers in exp-015 at 6.232 seconds. Its relative gain was smaller because the paired
six-worker control was faster in this later run. This is why the verdict uses paired
evidence from one run rather than comparing medians across runs.

Latency still trades for work after activation: large cold-index CPU regressed 41.24%
[+37.73%, +44.26%], peak RSS 1.56% [+1.31%, +1.79%], and minor faults 1.75%.
The separate 60,067-entry validation never crossed the threshold. There, cold-index
wall was −0.48% [−3.07%, +1.93%] and producer wall +1.17%
[−1.14%, +4.64%]; total CPU, peak RSS, and minor-fault intervals also showed no
regression.

## Verdict

**Superseded.** The implementation cleared the two preregistered endpoints, but the
post-review 120k boundary in exp-019 showed that observed scale alone activates too
late to help and still pays reserve-thread memory. Exp-021 retains the in-band spawn
mechanism and replaces the scale trigger with initial service-time calibration.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
