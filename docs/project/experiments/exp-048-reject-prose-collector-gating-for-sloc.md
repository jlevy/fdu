---
title: Reject prose collector gating for SLOC
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-048
  title: Reject prose collector gating for SLOC
  date: "2026-08-13"
  hypotheses:
    - H80
  subject:
    tree_label: fdu-content-selfhost-20260813
    tree_root_id: 0d8bca813ccc20705265ad42baad61b86c412e7927cf3fd4b8703be5e93c1f57
    tree_engine_digest: 98360347c76f3db629e4f96dd15f450e66f529b1c34de19138d6b222a392518e
    tree_entries: 307
    tree_directories: 74
    tree_files: 233
    tree_symlinks: 0
    tree_apparent_bytes: 3175738
    tree_allocated_bytes: 3760128
    tree_max_depth: 8
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
    control: frozen code-sloc-v1 semantic baseline
    candidate: skip prose-only collectors for code families
    control_binary:
      name: control
      sha256: d13aeb15098f4c3e74cba352d00e7fb1acd32c3f93585af6c2dc5007c16e11fc
      size_bytes: 867168
      args: []
    candidate_binary:
      name: candidate
      sha256: 0ab4bccf609fbc534067249d25a5b3dc5dd6082eda41838073a2a2f9bbc77150
      size_bytes: 867168
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: null
  results:
    - job: code-sloc
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 20247458.0
          candidate_median: 19856875.0
          change_pct: 1.501
          ci95_low_pct: -4.595
          ci95_high_pct: 4.234
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 9179625.5
          candidate_median: 8919166.5
          change_pct: 1.338
          ci95_low_pct: -6.427
          ci95_high_pct: 4.952
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 71680000.0
          candidate_median: 73281500.0
          change_pct: 2.131
          ci95_low_pct: -3.838
          ci95_high_pct: 8.284
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 29189000.0
          candidate_median: 28117000.0
          change_pct: -4.666
          ci95_low_pct: -6.059
          ci95_high_pct: -2.865
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 42168500.0
          candidate_median: 45036500.0
          change_pct: 8.017
          ci95_low_pct: -3.399
          ci95_high_pct: 16.986
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
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
          control_median: 9699328.0
          candidate_median: 9830400.0
          change_pct: 0.506
          ci95_low_pct: -1.887
          ci95_high_pct: 2.289
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: code-sloc-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 10118167.0
          candidate_median: 10146125.0
          change_pct: -0.571
          ci95_low_pct: -2.181
          ci95_high_pct: 3.775
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 2067145.5
          candidate_median: 2039583.5
          change_pct: -1.289
          ci95_low_pct: -5.108
          ci95_high_pct: 1.329
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 7506000.0
          candidate_median: 7497500.0
          change_pct: 0.044
          ci95_low_pct: -3.914
          ci95_high_pct: 3.339
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 4854500.0
          candidate_median: 4849500.0
          change_pct: -0.171
          ci95_low_pct: -1.906
          ci95_high_pct: 1.709
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 2650500.0
          candidate_median: 2620000.0
          change_pct: 2.694
          ci95_low_pct: -9.698
          ci95_high_pct: 6.388
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 2651437.5
          candidate_median: 2659937.5
          change_pct: 1.248
          ci95_low_pct: -3.386
          ci95_high_pct: 6.981
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 8019968.0
          candidate_median: 7938048.0
          change_pct: -1.12
          ci95_low_pct: -1.331
          ci95_high_pct: -0.412
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 18635104.5
          candidate_median: 19007833.5
          change_pct: -2.164
          ci95_low_pct: -3.645
          ci95_high_pct: 6.089
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 8457791.5
          candidate_median: 8100562.0
          change_pct: -7.875
          ci95_low_pct: -10.193
          ci95_high_pct: 3.053
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 79089500.0
          candidate_median: 72040500.0
          change_pct: -9.994
          ci95_low_pct: -16.6
          ci95_high_pct: 3.133
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 19019500.0
          candidate_median: 17330000.0
          change_pct: -7.255
          ci95_low_pct: -10.994
          ci95_high_pct: -5.435
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 60278000.0
          candidate_median: 54911000.0
          change_pct: -10.272
          ci95_low_pct: -19.981
          ci95_high_pct: 6.835
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
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
          control_median: 9953280.0
          candidate_median: 9764864.0
          change_pct: -1.557
          ci95_low_pct: -2.808
          ci95_high_pct: -1.146
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 9806812.5
          candidate_median: 9836896.0
          change_pct: 1.356
          ci95_low_pct: -4.736
          ci95_high_pct: 5.238
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 2041854.0
          candidate_median: 2057854.5
          change_pct: 1.497
          ci95_low_pct: -0.719
          ci95_high_pct: 3.219
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 7211000.0
          candidate_median: 7315500.0
          change_pct: 1.58
          ci95_low_pct: -1.858
          ci95_high_pct: 3.672
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 4719000.0
          candidate_median: 4753000.0
          change_pct: 1.954
          ci95_low_pct: -1.259
          ci95_high_pct: 2.988
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 2458000.0
          candidate_median: 2519000.0
          change_pct: 3.078
          ci95_low_pct: -5.043
          ci95_high_pct: 7.163
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 2572812.5
          candidate_median: 2575562.5
          change_pct: 0.331
          ci95_low_pct: -11.044
          ci95_high_pct: 11.556
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 7979008.0
          candidate_median: 7970816.0
          change_pct: -0.308
          ci95_low_pct: -0.716
          ci95_high_pct: 0.103
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 31
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: One mode bit and conditional prose counters; no dependency or unsafe code.
  verdict:
    decision: rejected
    primary_job: code-sloc
    primary_metric: wall_ns
    change_pct: 1.501
    reason: "The 12-pair wall interval crossed zero and the median did not clear the 3% acceptance threshold; cache-hit and basic jobs were also neutral."
    commit: d7363a298ac58905597b2ede8c9f3240938a3129
---
# Reject prose collector gating for SLOC

## Hypothesis

The frozen self-host profile attributed 8.74% of samples to
`BasicAccumulator::push_text`, compared with 5.42% in `CodeAccumulator::finish_line`.
The basic pass computed prose-only word, paragraph, and logical-word statistics for code
files, then discarded those values.
H80 predicted that gating those collectors by the already-known content family would
reduce `code-sloc` wall and component time by at least 3%.

## What was tried

The candidate added one mode bit to `BasicAccumulator` and skipped only prose counters
for known non-document families.
It retained UTF-8 and NUL admission, LF/CRLF/lone-CR handling, physical-line counts,
blank-line counts, and the independent SLOC parser.
The exact golden suite and multilingual self-host digest were unchanged before timing.

## What the numbers said

Across 12 interleaved pairs on the immutable 233-file self-host corpus, `code-sloc` wall
changed by +1.50% with a 95% interval of [-4.60%, +4.23%]. The component interval also
crossed zero. The change did reduce measured user CPU by 4.67%, but total CPU and wall
did not improve decisively because concurrent open and scheduling costs dominated the
saved character work.

The negative controls behaved as expected: basic content and both basic and SLOC
cache-hit paths were statistically neutral.

## Verdict

**REJECTED** — The primary wall interval crossed zero and the median did not clear the
3% acceptance threshold.
The implementation was reverted; the SLOC jobs and partial-coverage-aware harness
validation remain because they are reusable evidence infrastructure.
