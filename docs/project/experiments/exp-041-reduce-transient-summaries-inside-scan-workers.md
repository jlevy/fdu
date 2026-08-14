---
title: Reduce transient summaries inside scan workers
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-041
  title: Reduce transient summaries inside scan workers
  date: "2026-08-13"
  hypotheses:
    - H62
  subject:
    tree_label: h62-self-contained-901k
    tree_root_id: c95b1edda5762c399d4eaaf8494b1e1866f5554814d9db5c3fe353a5a13bc7a0
    tree_engine_digest: e7ed1ac6334eb80379d3a8b259188115462014f247c147b5560682cbb27d1fca
    tree_entries: 901963
    tree_directories: 110369
    tree_files: 791261
    tree_symlinks: 333
    tree_apparent_bytes: 16537459815
    tree_allocated_bytes: 18714202112
    tree_max_depth: 23
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
    control: H59 transient summary reduced from generic observation batches
    candidate: worker-local summary reduction with paths retained only for directories
    control_binary:
      name: h59
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
      name: h62
      sha256: 7e3cc13865837f98b530599a532dfc2e9b0b8634992ac26f8a7c755694fa79c1
      size_bytes: 1332912
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
    run_artifact: benchmarks/results/realtree/run-exp041-worker-local-summary.json
  results:
    - job: rich-summary-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3031837333.5
          candidate_median: 2966024417.0
          change_pct: -1.377
          ci95_low_pct: -3.705
          ci95_high_pct: -0.306
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        cpu_ns:
          control_median: 10673321500.0
          candidate_median: 10521707500.0
          change_pct: -1.123
          ci95_low_pct: -3.262
          ci95_high_pct: -0.221
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        user_cpu_ns:
          control_median: 483564000.0
          candidate_median: 309015000.0
          change_pct: -36.227
          ci95_low_pct: -36.949
          ci95_high_pct: -34.832
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        system_cpu_ns:
          control_median: 10183656000.0
          candidate_median: 10212767500.0
          change_pct: 0.534
          ci95_low_pct: -1.684
          ci95_high_pct: 1.377
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 16
        peak_rss_bytes:
          control_median: 14467072.0
          candidate_median: 9404416.0
          change_pct: -34.765
          ci95_low_pct: -35.952
          ci95_high_pct: -34.17
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        minor_faults:
          control_median: 1130.5
          candidate_median: 805.0
          change_pct: -28.436
          ci95_low_pct: -29.365
          ci95_high_pct: -28.117
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        involuntary_context_switches:
          control_median: 26868.0
          candidate_median: 24094.0
          change_pct: -10.922
          ci95_low_pct: -14.731
          ci95_high_pct: -6.52
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
  reference_tools: []
  complexity:
    lines_changed: 299
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - A second walker implementation could drift from the generic scan contract
    notes: The prototype was retained only while screening the preregistered H63 composition; exp-042 also missed the wall gate, so both engine layers were reverted
  verdict:
    decision: rejected
    primary_job: rich-summary-report
    primary_metric: wall_ns
    change_pct: -1.377
    reason: "Worker-local reduction cuts user CPU 36.23% and RSS 34.77%, but its 1.38% paired wall improvement misses the preregistered 3% production bar; the 720,805-entry replication is likewise only 1.26% with its interval crossing zero"
    commit: null
---
# Reduce Transient Summaries Inside Scan Workers

## Hypothesis

H62: now that H59 no longer constructs an index, the existing scan workers can reduce
the exact rich summary themselves.
Files should require no relative-path join, `Op`, observation batch, or channel handoff;
only directories that may be descended into need a path.
The same directory queue, scope, metadata reader, adaptive-worker policy, errors, and
report bytes must remain exact.

## Method

The immutable H59 control at `0916a40` and the worker-local prototype ran sixteen
adjacent pairs after three warmups on the mutation-free self-contained 901,963-entry
tree. Both used the same `fdu-transient-summary` contract and produced one identical
stable semantic hash in every sample.
The harness now permits distinct variant labels to share one work contract, so neither
binary is mislabeled as an indexed summary.

A second 20-pair run used the inactive 720,805-entry cache-pressure tree.
Both runs had zero invalid sample, semantic mismatch, baseline drift, or tree mutation.

## Results

On 901,963 entries, worker-local reduction improved paired wall only 1.38%
[0.31%, 3.71%], below the preregistered 3% bar.
The mechanism itself was clear: user CPU fell 36.23%, peak RSS 34.77%, minor faults
28.44%, and involuntary context switches 10.92%; system CPU was unchanged.

The independent 720,805-entry replication agreed: wall improved 1.26% [−0.59%, 3.10%],
user CPU 36.61%, RSS 27.66%, faults 23.25%, and involuntary context switches 10.31%.
System time remains the dominant floor, so less Rust allocation and channel work does
not yet translate into a substantial user-visible speedup.

## Verdict

**REJECTED AS A STANDALONE CHANGE** — The candidate misses the wall-time bar on two
mutation-free large trees despite decisive secondary resource improvements.

The prototype remained temporarily only to screen H63’s narrower macOS bulk records as
the preregistered composition.
Exp-042 changed wall by +1.86% [−1.96%, +4.56%], so that composition also failed and
both engine layers were reverted.
H62 is not production code.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
