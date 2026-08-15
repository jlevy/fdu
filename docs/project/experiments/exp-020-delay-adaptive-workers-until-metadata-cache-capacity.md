---
title: Delay adaptive workers until metadata-cache capacity
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-020
  title: Delay adaptive workers until metadata-cache capacity
  date: "2026-08-12"
  hypotheses:
    - H31
  checkpoint:
    profile: index-core-v1
    kept_variant: control
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
    candidate: spawn reserve workers after 262144 observed entries
    control_binary:
      name: control
      sha256: be3349ee5238da00b5bce9ff7f72e68fd3fc0a9f96eae16c969c520f0e90977f
      size_bytes: 535968
      args: []
    candidate_binary:
      name: candidate
      sha256: 5745c3254acae638fc28ef687d0dc974a7101262bf893a3df9c8bee08e5bb369
      size_bytes: 552512
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp020-capacity-threshold-large.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6185819229.0
          candidate_median: 6036831937.5
          change_pct: -1.71
          ci95_low_pct: -2.898
          ci95_high_pct: 1.054
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 4847468500.0
          candidate_median: 4654766583.0
          change_pct: -2.817
          ci95_low_pct: -4.518
          ci95_high_pct: -0.628
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 22821764500.0
          candidate_median: 31111446000.0
          change_pct: 35.584
          ci95_low_pct: 9.009
          ci95_high_pct: 43.215
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 2879123000.0
          candidate_median: 3184001000.0
          change_pct: 11.412
          ci95_low_pct: 5.604
          ci95_high_pct: 14.499
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 19960637000.0
          candidate_median: 27889961000.0
          change_pct: 38.752
          ci95_low_pct: 9.569
          ci95_high_pct: 47.641
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 325206016.0
          candidate_median: 329023488.0
          change_pct: 1.188
          ci95_low_pct: 0.911
          ci95_high_pct: 1.288
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
          control_median: 11180542208.5
          candidate_median: 10979718583.5
          change_pct: -4.16
          ci95_low_pct: -6.347
          ci95_high_pct: -0.255
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 4932185334.0
          candidate_median: 4883320667.0
          change_pct: -3.453
          ci95_low_pct: -6.518
          ci95_high_pct: 3.059
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 43551028500.0
          candidate_median: 55435741000.0
          change_pct: 29.774
          ci95_low_pct: 15.951
          ci95_high_pct: 33.677
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 3745038500.0
          candidate_median: 4251967000.0
          change_pct: 13.476
          ci95_low_pct: 8.539
          ci95_high_pct: 14.985
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 39831889000.0
          candidate_median: 51240458500.0
          change_pct: 31.933
          ci95_low_pct: 16.845
          ci95_high_pct: 35.417
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 325820416.0
          candidate_median: 330178560.0
          change_pct: 1.348
          ci95_low_pct: 0.994
          ci95_high_pct: 1.53
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
    notes: Scale alone cannot distinguish a cached medium tree from a latency-bound large one early enough; supersede with first-chunk service-time calibration.
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -1.71
    reason: "The safer trigger removed the 120k activation but improved 720k end-to-end wall only 1.71% with an interval crossing zero, below the bar"
    commit: null
---
# Delay adaptive workers until metadata-cache capacity

## Hypothesis

The 100k trigger activated too near the end of a 120k scan.
Moving it to 262,144—the development host’s measured vnode ceiling rounded to a portable
power of two—should avoid that boundary cost while leaving roughly two thirds of the
720k subject for the sixteen-worker pool.

## What was tried

Only the entry threshold changed.
The same in-band message created the same bounded reserve, explicit thread counts
remained fixed, and the immutable 720,805-entry subject and twelve-pair protocol matched
the earlier large-tree runs.

## What the numbers said

Producer wall improved 4.16% [−6.35%, −0.26%], but the user-facing end-to-end job did
not: cold-index wall improved only 1.71% [−2.90%, +1.05%]. The later activation
therefore discarded most of the benefit needed to repay the concurrency path.
Scale could be made safe at the boundary or early enough to matter, but these
experiments did not find one fixed scale that did both.

## Verdict

**Rejected.** A first scan can measure its own initial service time without already
knowing its eventual size.
Exp-021 tests that direct signal instead of another scale constant.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
