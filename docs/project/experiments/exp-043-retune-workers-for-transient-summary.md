---
title: Retune workers for transient summary
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-043
  title: Retune workers for transient summary
  date: "2026-08-13"
  hypotheses:
    - H65
  subject:
    tree_label: h65-cache-pressure-720k-confirm
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
    trials: 20
    warmups: 3
    interleaved: true
    control: H59 transient summary with the accepted automatic six-worker policy
    candidate: H59 transient summary with a fixed eight-worker pool
    control_binary:
      name: auto
      sha256: 0a02839ffe9c0221c96fbca2d20e2d6f97636b1891860389fee468986a677f73
      size_bytes: 1299840
      args:
        - --cache
        - "off"
        - --view
        - summary
        - --format
        - json
        - --color
        - never
    candidate_binary:
      name: t8
      sha256: 4ef4ba82689ce2f54fcfb7bee694616eae11a3d5bf3f0d17daec00276f82bce5
      size_bytes: 1299840
      args:
        - --cache
        - "off"
        - --view
        - summary
        - --format
        - json
        - --color
        - never
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp043-summary-worker-depth.json
  results:
    - job: rich-summary-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2210491062.5
          candidate_median: 2258449791.5
          change_pct: 0.669
          ci95_low_pct: -1.562
          ci95_high_pct: 3.994
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        cpu_ns:
          control_median: 8920694000.0
          candidate_median: 12531023500.0
          change_pct: 40.66
          ci95_low_pct: 38.261
          ci95_high_pct: 43.317
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        user_cpu_ns:
          control_median: 387452500.0
          candidate_median: 434005500.0
          change_pct: 11.674
          ci95_low_pct: 10.131
          ci95_high_pct: 14.335
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        system_cpu_ns:
          control_median: 8535105500.0
          candidate_median: 12102655500.0
          change_pct: 42.026
          ci95_low_pct: 39.627
          ci95_high_pct: 44.875
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        peak_rss_bytes:
          control_median: 15245312.0
          candidate_median: 15851520.0
          change_pct: 3.386
          ci95_low_pct: 1.22
          ci95_high_pct: 6.431
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        minor_faults:
          control_median: 1177.5
          candidate_median: 1215.0
          change_pct: 2.594
          ci95_low_pct: 0.691
          ci95_high_pct: 4.713
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        involuntary_context_switches:
          control_median: 18060.0
          candidate_median: 32786.5
          change_pct: 76.661
          ci95_low_pct: 73.334
          ci95_high_pct: 85.836
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        voluntary_context_switches:
          control_median: 45161.5
          candidate_median: 48040.5
          change_pct: 5.004
          ci95_low_pct: 1.379
          ci95_high_pct: 12.919
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
  reference_tools: []
  complexity:
    lines_changed: 12
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - A report-plan-specific worker policy could overfit one tree topology
    notes: A compile-time-only experimental override built fixed 8, 10, 12, and 16 worker binaries; the hook was removed after the curve and independent replication rejected a policy change
  verdict:
    decision: rejected
    primary_job: rich-summary-report
    primary_metric: wall_ns
    change_pct: 0.669
    reason: "Eight workers looked promising in the 901,963-entry screen but the independent 720,805-entry 20-pair confirmation changed wall by +0.67% [-1.56%, +3.99%] while CPU rose 40.66%; automatic six remains the policy"
    commit: null
---
# Retune Workers for Transient Summary

## Hypothesis

H65: H59 no longer constructs an index for an exact uncached summary, so the accepted
six-worker knee might no longer balance filesystem concurrency against consumer work.
A report-plan-specific fixed pool could compose with H59 without changing indexed scans.

## Method

A compile-time-only experiment hook built otherwise identical binaries with fixed pools
of 8, 10, 12, and 16 workers.
It never became a CLI or runtime flag and was removed after measurement.

The first five-pair screen on the self-contained 901,963-entry tree compared 8, 10, and
12 against the immutable H59 automatic/six control.
Eight looked promising at 5.2% faster, ten was neutral, and twelve was 3.9% slower.
A sixteen-pair confirmation compared eight and sixteen beside the same control.
Eight retained a favorable 4.55% median but its interval crossed zero [−13.57%, +1.92%];
sixteen was slower and used more memory.

The decision run used twenty adjacent pairs on the independent inactive 720,805-entry
cache-pressure tree.
Every run had one identical semantic digest and there were no invalid samples, baseline
drift, or tree mutation.

## Results

Eight workers changed paired wall time by **+0.67%** on the independent tree, with a 95%
interval from −1.56% to +3.99%. The resource evidence was decisively worse: aggregate
CPU rose 40.66%, system CPU 42.03%, user CPU 11.67%, peak RSS 3.39%, and involuntary
context switches 76.66%.

The topology difference explains why a short screen can mislead.
The broader `benchmarks` tree sometimes gives two additional workers enough independent
directories to hide latency, but the stable cache-pressure replication receives no wall
benefit while paying for all of their kernel work.
A static plan-specific depth would therefore overfit one tree.

## Verdict

**REJECTED** — The only promising arm did not reproduce and materially increased every
resource cost. The compile-time experiment hook is removed; transient summaries and
indexed scans both retain the accepted automatic six-worker operating point.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
