---
title: Scoped counters stay below the exploratory acceptance threshold
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-075
  title: Scoped counters stay below the exploratory acceptance threshold
  date: "2026-09-01"
  hypotheses: []
  subject:
    tree_label: cargo-registry-src
    tree_root_id: 0fc8126bf6ad20b7e7fd34c8419abf6aac0521d5877fcbe6b2472316e07dc158
    tree_engine_digest: 51114fc9f7b62d7bdbc789326ead71249fe6e2ab7deabbc5a3bcd09ee3b6d523
    tree_provenance: Live local Cargo registry source cache observed in place; package-manager state and exact tree shape are not reconstructible.
    tree_reconstructible: false
    tree_entries: 11142
    tree_directories: 2241
    tree_files: 8901
    tree_symlinks: 0
    tree_apparent_bytes: 179605080
    tree_allocated_bytes: 203902976
    tree_max_depth: 10
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
    trials: 3
    warmups: 1
    interleaved: true
    control: correctness head b5d9ba4 before scoped instrumentation
    candidate: scoped instrumentation at 1393d31
    control_binary:
      name: before
      sha256: 99e2affb057e56c018353f084be17126bb76974a7163a5f6ec4184fcb84a7034
      size_bytes: 1826000
      args: []
    candidate_binary:
      name: instrumented
      sha256: 860c849a3cdf7e51f8981825dfbb80b1a25101b598d00d920bae23c717da2fd6
      size_bytes: 2107184
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-instrumentation-overhead.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 76175333.0
          candidate_median: 71918458.0
          control_p95_over_median: 1.133
          candidate_p95_over_median: 1.002
          change_pct: -5.588
          ci95_low_pct: -16.533
          ci95_high_pct: -1.817
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 3
        component_ns:
          control_median: 53558917.0
          candidate_median: 51484625.0
          control_p95_over_median: 1.203
          candidate_p95_over_median: 1.009
          change_pct: -2.984
          ci95_low_pct: -20.606
          ci95_high_pct: 1.102
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 3
        cpu_ns:
          control_median: 369414000.0
          candidate_median: 346716000.0
          control_p95_over_median: 1.192
          candidate_p95_over_median: 1.007
          change_pct: -6.144
          ci95_low_pct: -20.751
          ci95_high_pct: -0.211
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 3
        user_cpu_ns:
          control_median: 76551000.0
          candidate_median: 76452000.0
          control_p95_over_median: 1.001
          candidate_p95_over_median: 1.002
          change_pct: -0.129
          ci95_low_pct: -1.614
          ci95_high_pct: 0.291
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 3
        system_cpu_ns:
          control_median: 292863000.0
          candidate_median: 270264000.0
          control_p95_over_median: 1.243
          candidate_p95_over_median: 1.008
          change_pct: -7.717
          ci95_low_pct: -25.167
          ci95_high_pct: 0.191
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 3
        peak_rss_bytes:
          control_median: 12697600.0
          candidate_median: 13238272.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.004
          change_pct: 3.871
          ci95_low_pct: 3.193
          ci95_high_pct: 5.052
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 3
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes straddles its +5% regression limit"
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
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 59279791.0
          candidate_median: 59653417.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.022
          change_pct: 1.929
          ci95_low_pct: 0.63
          ci95_high_pct: 2.45
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 3
        component_ns:
          control_median: 55160125.0
          candidate_median: 55490000.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.023
          change_pct: 2.069
          ci95_low_pct: 0.598
          ci95_high_pct: 2.348
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 3
        cpu_ns:
          control_median: 342879000.0
          candidate_median: 342714000.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.004
          change_pct: 0.388
          ci95_low_pct: -0.422
          ci95_high_pct: 0.539
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 3
        user_cpu_ns:
          control_median: 60936000.0
          candidate_median: 62573000.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.008
          change_pct: 2.686
          ci95_low_pct: 0.92
          ci95_high_pct: 4.25
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 3
        system_cpu_ns:
          control_median: 282356000.0
          candidate_median: 280854000.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.001
          change_pct: -0.439
          ci95_low_pct: -0.713
          ci95_high_pct: 0.069
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 3
        peak_rss_bytes:
          control_median: 14663680.0
          candidate_median: 15040512.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.0
          change_pct: 2.57
          ci95_low_pct: 1.436
          ci95_high_pct: 3.269
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 3
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
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
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 1151
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: Adds five exact jobs and flat runtime-gated lifecycle counters; no dependency or unsafe code.
  verdict:
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 1.929
    reason: "The three-pair uncontrolled screen measured +1.93% on default-tree and -5.59% on cold-scan-index; this is insufficient for a timing claim but bounds the enabled-off instrumentation below the 3% experiment bar in the slower direction."
    commit: 1393d31
---
# Scoped counters stay below the exploratory acceptance threshold

## Question

Does compiling in the new runtime-gated counters materially slow ordinary runs when
`FDU_COUNTERS` is unset?

## Method

The run interleaved the correctness-fixed probe before and after the instrumentation
commit on one unchanged real tree.
Counters were disabled in both variants.
The comparison used three exploratory pairs on an uncontrolled host because its purpose
was to catch a large observer effect, not to accept an optimization.

## Result

The two jobs moved in opposite directions: `default-tree` was 1.93% slower and
`cold-scan-index` was 5.59% faster by median wall time.
The slower result stayed below the 3% experiment threshold, but three uncontrolled pairs
cannot establish equivalence.

## Decision

Retain the instrumentation for mechanism attribution.
Its exact counters are needed to distinguish work skipped from work merely shifted, and
this screen found no observer cost large enough to invalidate the Phase 1 profiles.
Recheck the disabled path if the final parity margin is narrower than 3%.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
