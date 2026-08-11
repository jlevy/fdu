---
title: Cumulative PR 3 review against the correctness-normalized base
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-012
  title: Cumulative PR 3 review against the correctness-normalized base
  date: "2026-08-11"
  hypotheses:
    - H1
    - H5
    - H10
    - H14
    - H18
    - H32
  subject:
    tree_label: reference-tree-60k
    tree_root_id: c8dea479163b8ed64a8e76046148d5a4deecdee809ff9c432219392f8018dc6f
    tree_engine_digest: eb51259c3d468eeb470d0d9d43ebb199fcdde2031754cba94b29a983bc19cf46
    tree_entries: 59654
    tree_directories: 7341
    tree_files: 52291
    tree_symlinks: 22
    tree_apparent_bytes: 1082046361
    tree_allocated_bytes: 1225879552
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
    trials: 16
    warmups: 3
    interleaved: true
    control: PR base plus only the required non-file roll-up correction (c0ddcb9)
    candidate: frozen reviewed implementation (bd479aa)
    control_binary:
      name: corrected
      sha256: 92cf380844976fc4e9ca52135f2c1dac64c9637f6d9880d894876b1b3bd08ebf
      size_bytes: 468816
      args: []
      engine_revision: c0ddcb9807dacd7190cfc6639104db8fc33be896
      harness_revision: bd479aaee90263dea8c7dc5ce9d131368e749568
      harness_sha256: 79332acd1cf1899c9265453e3fe48089fb14ca56c163c78ce1c7ff87ce043a6a
      target: aarch64-apple-darwin
      build_profile: release
      features: []
      build_command: compat-probe then cargo build --locked --release -p fdu --example perf_probe_review --no-default-features
    candidate_binary:
      name: candidate
      sha256: 4da08321bd34656d4addfe6dd87fc5e80d15125d3f822c02cba69491174efd05
      size_bytes: 535888
      args: []
      engine_revision: bd479aaee90263dea8c7dc5ce9d131368e749568
      harness_revision: bd479aaee90263dea8c7dc5ce9d131368e749568
      harness_sha256: 62a986a614823a5b3c979fe681ce9f94ab83f50938b3c388243489f63d5826ee
      target: aarch64-apple-darwin
      build_profile: release
      features: []
      build_command: cargo build --locked --release -p fdu --example perf_probe --no-default-features
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    evidence_grade: claim-grade
    run_schema: fdu-realtree-run-v2
    schedule: round-robin-by-ordinal-v1
    schedule_sha256: 9efcffe0c68b95f4a207671da91dce185b5e5c0964ded1e045bc12f2b32ff2e9
    schedule_seed: null
    run_artifact: docs/project/experiments/evidence/exp-012-run.json
    run_artifact_sha256: edc185a6dead31da62c1567be244a526193d66ec8a28259f6094eec5daa96e7b
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 677878125.0
          candidate_median: 335200125.0
          change_pct: -50.947
          ci95_low_pct: -52.31
          ci95_high_pct: -49.017
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        component_ns:
          control_median: 550631187.5
          candidate_median: 203755375.0
          change_pct: -63.574
          ci95_low_pct: -64.646
          ci95_high_pct: -60.456
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 659687500.0
          candidate_median: 1201847500.0
          change_pct: 83.11
          ci95_low_pct: 74.316
          ci95_high_pct: 91.234
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 251777000.0
          candidate_median: 255744500.0
          change_pct: 1.06
          ci95_low_pct: -1.642
          ci95_high_pct: 2.294
          direction: regression
          ci_excludes_zero: false
          significant_improvement: false
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 408318000.0
          candidate_median: 948113000.0
          change_pct: 136.671
          ci95_low_pct: 117.795
          ci95_high_pct: 148.678
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
        peak_rss_bytes:
          control_median: 33603584.0
          candidate_median: 34963456.0
          change_pct: 3.986
          ci95_low_pct: 2.813
          ci95_high_pct: 6.425
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 528007271.0
          candidate_median: 204364271.0
          change_pct: -61.236
          ci95_low_pct: -63.143
          ci95_high_pct: -60.443
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        component_ns:
          control_median: 521860749.5
          candidate_median: 197699416.5
          change_pct: -62.029
          ci95_low_pct: -63.948
          ci95_high_pct: -61.203
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 517923000.0
          candidate_median: 1155275500.0
          change_pct: 120.52
          ci95_low_pct: 114.141
          ci95_high_pct: 127.577
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 133807500.0
          candidate_median: 174310500.0
          change_pct: 29.708
          ci95_low_pct: 28.06
          ci95_high_pct: 31.801
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 384593500.0
          candidate_median: 978655500.0
          change_pct: 151.394
          ci95_low_pct: 143.177
          ci95_high_pct: 162.597
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
        peak_rss_bytes:
          control_median: 12607488.0
          candidate_median: 15687680.0
          change_pct: 23.797
          ci95_low_pct: 20.256
          ci95_high_pct: 26.008
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 712180229.5
          candidate_median: 382866000.0
          change_pct: -47.248
          ci95_low_pct: -48.553
          ci95_high_pct: -43.275
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        component_ns:
          control_median: 26644979.0
          candidate_median: 26885750.0
          change_pct: -0.069
          ci95_low_pct: -1.794
          ci95_high_pct: 7.247
          direction: improvement
          ci_excludes_zero: false
          significant_improvement: false
          significant: false
          pairs: 16
        cpu_ns:
          control_median: 683274000.0
          candidate_median: 1229110000.0
          change_pct: 80.283
          ci95_low_pct: 75.357
          ci95_high_pct: 90.097
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 267789500.0
          candidate_median: 270618000.0
          change_pct: 1.784
          ci95_low_pct: -0.943
          ci95_high_pct: 2.669
          direction: regression
          ci_excludes_zero: false
          significant_improvement: false
          significant: false
          pairs: 16
        system_cpu_ns:
          control_median: 414545500.0
          candidate_median: 957989000.0
          change_pct: 133.281
          ci95_low_pct: 122.688
          ci95_high_pct: 149.484
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
        peak_rss_bytes:
          control_median: 41951232.0
          candidate_median: 42909696.0
          change_pct: 2.043
          ci95_low_pct: 0.815
          ci95_high_pct: 3.463
          direction: regression
          ci_excludes_zero: true
          significant_improvement: false
          significant: true
          pairs: 16
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1013596896.0
          candidate_median: 626572708.5
          change_pct: -37.117
          ci95_low_pct: -38.728
          ci95_high_pct: -36.285
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        component_ns:
          control_median: 667827229.0
          candidate_median: 412169167.0
          change_pct: -37.533
          ci95_low_pct: -38.887
          ci95_high_pct: -35.924
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 990485500.0
          candidate_median: 616008000.0
          change_pct: -37.209
          ci95_low_pct: -38.319
          ci95_high_pct: -36.337
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 572217000.0
          candidate_median: 250415500.0
          change_pct: -56.303
          ci95_low_pct: -56.826
          ci95_high_pct: -55.937
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 416397000.0
          candidate_median: 367798500.0
          change_pct: -11.114
          ci95_low_pct: -12.678
          ci95_high_pct: -8.112
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        peak_rss_bytes:
          control_median: 34840576.0
          candidate_median: 32112640.0
          change_pct: -7.93
          ci95_low_pct: -8.228
          ci95_high_pct: -6.752
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 345086291.5
          candidate_median: 219777375.0
          change_pct: -36.707
          ci95_low_pct: -39.593
          ci95_high_pct: -34.779
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        component_ns:
          control_median: 219212729.0
          candidate_median: 94476250.5
          change_pct: -56.948
          ci95_low_pct: -58.168
          ci95_high_pct: -55.472
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        cpu_ns:
          control_median: 335982500.0
          candidate_median: 213648000.0
          change_pct: -36.844
          ci95_low_pct: -37.338
          ci95_high_pct: -35.979
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        user_cpu_ns:
          control_median: 322120000.0
          candidate_median: 202518500.0
          change_pct: -37.509
          ci95_low_pct: -37.909
          ci95_high_pct: -36.896
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        system_cpu_ns:
          control_median: 14243000.0
          candidate_median: 10722000.0
          change_pct: -21.816
          ci95_low_pct: -30.003
          ci95_high_pct: -17.305
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        blocked_ns:
          control_median: 9317333.0
          candidate_median: 5926250.0
          change_pct: -30.636
          ci95_low_pct: -62.102
          ci95_high_pct: -6.608
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
        peak_rss_bytes:
          control_median: 32833536.0
          candidate_median: 31113216.0
          change_pct: -5.538
          ci95_low_pct: -6.105
          ci95_high_pct: -5.131
          direction: improvement
          ci_excludes_zero: true
          significant_improvement: true
          significant: true
          pairs: 16
  reference_tools: []
  complexity:
    lines_changed: 981
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - higher cold-path CPU and scheduler contention
      - parallel cancellation and backpressure protocol
    notes: "No dependency or unsafe-code expansion; the reviewed diff also repairs public API, cancellation, oracle, and snapshot fanout correctness."
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -50.947
    reason: "Latency improved 50.95%, but cold-path CPU regressed 83.11% with the full interval above the 10% resource guardrail, so no universal-performance waiver is justified"
    commit: bd479aaee90263dea8c7dc5ce9d131368e749568
    latency_gate_passed: true
    resource_guardrails:
      - metric: cpu_ns
        maximum_regression_pct: 10.0
        observed_change_pct: 83.11
        ci95_low_pct: 74.316
        ci95_high_pct: 91.234
        status: failed
        reason: "95% interval is wholly above the +10% limit"
      - metric: peak_rss_bytes
        maximum_regression_pct: 10.0
        observed_change_pct: 3.986
        ci95_low_pct: 2.813
        ci95_high_pct: 6.425
        status: passed
        reason: "no statistically established regression above +10%"
    resource_waiver_reason: null
---
# Cumulative PR #3 Review Against the Correctness-Normalized Base

## Hypothesis

After repairing the producer, reducer, public API, and evidence contracts, the complete
performance stack should still deliver a material end-to-end latency win over the PR
base without exceeding the default 10% CPU or peak-RSS guardrails.

## What was tried

The run interleaved three release binaries under one exact v2 schedule:

- `base` is the literal PR base `fdd9e523`. The current semantic probe compiles against
  it, but the full-index jobs correctly reject its roll-up: it counts 548 apparent bytes
  from symlink/special entries that the independent file-only reducer excludes.
- `corrected` is `c0ddcb9`. Between the literal base and this revision the only
  production semantic change is the non-file roll-up fix (plus its matching golden), so
  it is the like-for-like timing control for index jobs.
- `candidate` is the frozen reviewed implementation `bd479aa`.

The exact-base producer remains directly comparable because it emits filesystem records
rather than the incorrect index reducer.
Corrected versus exact-base producer wall was -0.16% with an interval crossing zero,
showing that the normalization itself is performance-neutral for that job.
Candidate versus the exact producer base was -61.62% [-65.43%, -59.12%].

The previous legacy tree fingerprint had drifted and was not reused.
This run pins the current 59,654-entry content before and after; the tree did not change
during the run. The workstation was contended, so these are paired relative results
rather than an absolute-throughput claim.
Sixteen measured ordinals plus three warmups, alternating variant order, produced narrow
intervals despite that contention.

## What the numbers said

Against the correctness-normalized control:

- cold scan into a complete index: wall -50.95% [-52.31%, -49.02%], component -63.57%,
  but total CPU +83.11% [+74.32%, +91.23%] and RSS +3.99%;
- oracle-instrumented producer: wall -61.24%, but CPU +120.52% and RSS +23.80%;
- warm revalidation: wall -37.12%, CPU -37.21%, RSS -7.93%;
- snapshot load: wall -36.71%, CPU -36.84%, RSS -5.54%;
- cold scan plus snapshot save: wall -47.25%, CPU +80.28%, while the serialization
  component itself was unchanged.

The separate wide snapshot-load evidence has zero invalid samples and scales close to
linearly: 28.7 ms at 10k, 220.4 ms at 100k, 1.13 s at 500k, and 2.22 s at 1M. That
supports the direct parent/name lookup and does not resurrect the superseded exp-005
headline.

## Verdict

**REJECTED AS A UNIVERSAL PERFORMANCE WIN** — The latency result is real, but the cold
path buys it with a statistically established 83% CPU regression, far beyond the 10%
default. No waiver is justified for an automatic platform-independent policy.
The parallel path remains useful for explicitly latency-oriented operation, and the warm
and snapshot improvements are genuine resource wins; product policy and documentation
must expose that distinction rather than calling the stack unconditionally faster.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
