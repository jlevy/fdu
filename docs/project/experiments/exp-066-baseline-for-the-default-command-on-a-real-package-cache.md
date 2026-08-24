---
title: Baseline for the default command on a real package cache
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-066
  title: Baseline for the default command on a real package cache
  date: "2026-08-23"
  hypotheses: []
  subject:
    tree_label: rustup-toolchains
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: fa9b471063e3ffc70ade7d188d8416f9dfdd99f5bb525a74611203aa7681bc8e
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains. Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 175191
    tree_directories: 4956
    tree_files: 170235
    tree_symlinks: 0
    tree_apparent_bytes: 4900108867
    tree_allocated_bytes: 5420306432
    tree_max_depth: 16
    tree_mutated_during_run: false
    host_cpu: Apple M1 Pro
    host_arch: arm64
    host_cores: 10
    host_performance_cores: 8
    host_efficiency_cores: 2
    host_memory_bytes: 34359738368
    host_system: Darwin 25.5.0
    filesystem: apfs
    host_virtualization: bare-metal
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: "main at 778aa74, perf_probe default-tree"
    candidate: "the same binary: a self-comparison that establishes the default-path numbers"
    control_binary:
      name: control
      sha256: 64fab3d3060e99ef7b2456ffad47a43b282a16a4db913e8d3811b71983cbf9b3
      size_bytes: 1561392
      args: []
    candidate_binary:
      name: candidate
      sha256: 64fab3d3060e99ef7b2456ffad47a43b282a16a4db913e8d3811b71983cbf9b3
      size_bytes: 1561392
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-066-default-command-baseline.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 299472812.5
          candidate_median: 316383083.5
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.183
          change_pct: 5.923
          ci95_low_pct: -1.094
          ci95_high_pct: 20.035
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 295515124.5
          candidate_median: 311526229.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.172
          change_pct: 5.928
          ci95_low_pct: -0.888
          ci95_high_pct: 19.307
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1506611500.0
          candidate_median: 1543780000.0
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.077
          change_pct: 2.804
          ci95_low_pct: -6.943
          ci95_high_pct: 14.018
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 62694000.0
          candidate_median: 63084000.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.053
          change_pct: 0.771
          ci95_low_pct: -2.421
          ci95_high_pct: 4.969
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1443673000.0
          candidate_median: 1482219500.0
          control_p95_over_median: 1.069
          candidate_p95_over_median: 1.078
          change_pct: 2.98
          ci95_low_pct: -7.211
          ci95_high_pct: 14.999
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 11755520.0
          candidate_median: 12984320.0
          control_p95_over_median: 1.151
          candidate_p95_over_median: 1.046
          change_pct: -1.521
          ci95_low_pct: -8.039
          ci95_high_pct: 18.488
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes straddles its +5% regression limit"
          - "minor_faults straddles its +10% regression limit"
          - voluntary_context_switches is missing a paired percent interval
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: inconclusive
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 659372604.0
          candidate_median: 648868854.5
          control_p95_over_median: 1.071
          candidate_p95_over_median: 1.114
          change_pct: -0.922
          ci95_low_pct: -4.728
          ci95_high_pct: 7.172
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 307201125.0
          candidate_median: 308637812.5
          control_p95_over_median: 1.11
          candidate_p95_over_median: 1.223
          change_pct: 0.921
          ci95_low_pct: -7.418
          ci95_high_pct: 15.804
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1999162500.0
          candidate_median: 2048638500.0
          control_p95_over_median: 1.063
          candidate_p95_over_median: 1.092
          change_pct: -1.036
          ci95_low_pct: -7.0
          ci95_high_pct: 11.151
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 511002000.0
          candidate_median: 519939000.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.026
          change_pct: 1.141
          ci95_low_pct: -1.966
          ci95_high_pct: 4.995
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1475150500.0
          candidate_median: 1537227000.0
          control_p95_over_median: 1.095
          candidate_p95_over_median: 1.104
          change_pct: -2.754
          ci95_low_pct: -9.423
          ci95_high_pct: 13.749
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 87375872.0
          candidate_median: 88145920.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.016
          change_pct: 0.276
          ci95_low_pct: -1.522
          ci95_high_pct: 1.74
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "voluntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 386864625.0
          candidate_median: 362204416.5
          control_p95_over_median: 1.119
          candidate_p95_over_median: 1.123
          change_pct: -1.925
          ci95_low_pct: -11.846
          ci95_high_pct: 3.531
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 381916270.5
          candidate_median: 357341312.5
          control_p95_over_median: 1.119
          candidate_p95_over_median: 1.125
          change_pct: -1.868
          ci95_low_pct: -11.894
          ci95_high_pct: 3.433
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1787411000.0
          candidate_median: 1718030000.0
          control_p95_over_median: 1.098
          candidate_p95_over_median: 1.023
          change_pct: -5.132
          ci95_low_pct: -13.158
          ci95_high_pct: -0.915
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 209084000.0
          candidate_median: 209362000.0
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.058
          change_pct: -0.82
          ci95_low_pct: -6.567
          ci95_high_pct: 7.722
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1573423500.0
          candidate_median: 1524664500.0
          control_p95_over_median: 1.106
          candidate_p95_over_median: 1.028
          change_pct: -5.655
          ci95_low_pct: -12.825
          ci95_high_pct: -1.714
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 106512384.0
          candidate_median: 105922560.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.025
          change_pct: -0.241
          ci95_low_pct: -0.64
          ci95_high_pct: 0.206
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree-first
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 385828437.5
          candidate_median: 375887375.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.098
          change_pct: -3.913
          ci95_low_pct: -9.717
          ci95_high_pct: 6.211
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 380516729.0
          candidate_median: 368831812.5
          control_p95_over_median: 1.113
          candidate_p95_over_median: 1.104
          change_pct: -3.938
          ci95_low_pct: -10.139
          ci95_high_pct: 5.813
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1746251000.0
          candidate_median: 1708975500.0
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.088
          change_pct: -2.831
          ci95_low_pct: -5.033
          ci95_high_pct: 2.305
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 203399000.0
          candidate_median: 218827000.0
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.02
          change_pct: 5.409
          ci95_low_pct: 0.744
          ci95_high_pct: 13.831
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1544438500.0
          candidate_median: 1502394500.0
          control_p95_over_median: 1.1
          candidate_p95_over_median: 1.091
          change_pct: -3.114
          ci95_low_pct: -5.873
          ci95_high_pct: 0.381
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 106438656.0
          candidate_median: 106938368.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.014
          change_pct: 0.693
          ci95_low_pct: -1.095
          ci95_high_pct: 1.726
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 272062250.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -1.925
    reason: "First measurement of fdu <dir> itself: the repeated run rewrites a 13.9 MB snapshot it never reads on all 24 trials, and the write plus render is about 70 ms of a 375 ms default run."
    commit: 62f82f6
---
## What was measured

The default command, `fdu <dir>`, for the first time in this record: 66 artifacts
preceded this one and none of them timed what a user gets by typing nothing else.
`cold-scan-index`, the proxy every cumulative checkpoint used, is the probe’s walk plus
the index build; it excludes the render and the snapshot write.

Two new jobs drive the real one-shot path through `prepare_report` with cache policy
`auto`, the tree view at its default depth, the text renderer run to completion, and the
save joined before exit — what the command line does before it returns.
`default-tree-first` empties the snapshot path before every trial (the first run on a
tree); `default-tree` prepares the snapshot once and leaves it in place (the second run,
over an unchanged tree).
Both arms are the same binary, so the paired comparisons above are a self-test and their
verdicts mean nothing; the absolute numbers and the probe’s `snapshot_written` flag are
the result.

Entry points: `perf_probe default-tree` (`crates/fdu-core/examples/perf_probe.rs`), jobs
`default-tree-first` and `default-tree` in
`explorations/benchmarks/realtree/measure.py`.

## Result

On the 175k-entry rustup store, warm page cache, uncontrolled host:

| job | wall | component | peak RSS |
| --- | ---: | ---: | ---: |
| `aggregate-summary` | 300–316 ms | 296–312 ms | 12 MiB |
| `cold-scan-index` | 649–659 ms (probe wall, oracle included) | 307–309 ms | 84 MiB |
| `default-tree-first` | 376–386 ms | 369–381 ms | 102 MiB |
| `default-tree` | 362–387 ms | 357–382 ms | 102 MiB |

Three things the proxy could not show:

- **The repeated run costs what the first run costs.** `default-tree` rewrote the
  13,925,460-byte snapshot on all 24 measured trials (`snapshot_written: true` on every
  one), over a tree that did not change.
  The snapshot is never read on this path — `plan_report` sets `read_snapshot` only when
  analysis is requested, because revalidation would stat every entry regardless — so on
  the default path it is write-only cost.
  This is `fdu-2um8`, now visible in the record.
- **The default run is about 70 ms over the index build.** Component 357–382 ms against
  `cold-scan-index`’s 307–309 ms: the render plus the serialize, CRC, write,
  `F_FULLFSYNC` and rename of 13.9 MB, about 18–20% of a default run on this subject.
  The cache-layers plan priced the write at about a third of a default run on `/usr`
  (Linux); this is the macOS number for a package cache.
- **The probe’s own oracle is half of `cold-scan-index`’s wall.** 649 ms wall against
  307 ms component: the digest walk over the retained index is the difference, which is
  what `fdu-4xtm` (engine-scoped counters and a `--no-oracle` mode) exists to remove
  from attribution runs.
  The default-tree jobs carry no such overhead — their oracle is the five tallies read
  off the root node — so their wall is the user’s wall.

## Regime

Exploratory. The host was not quiet: an `ANECompilerService` process had held a core at
~99% for over a day and another session’s Python was running, so the quiet gate refused
the cell (CPU busy 44.3% > 25%) and the run was taken uncontrolled.
Paired intervals on a self-comparison show the noise floor of that regime on this
subject — roughly ±6–10% on the default jobs at twelve trials — which is the width a
candidate will have to clear.
The next experiment on this path (`fdu-2um8`) should be run when the host is quiet, or
with 16–20 trials if it cannot be.
