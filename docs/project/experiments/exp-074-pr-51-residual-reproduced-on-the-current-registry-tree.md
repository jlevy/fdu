---
title: "PR #51 residual reproduced on the current registry tree"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-074
  title: "PR #51 residual reproduced on the current registry tree"
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
    trials: 4
    warmups: 1
    interleaved: true
    control: pre-rewrite main at b75bf85
    candidate: "PR #51 head at e8f1bed"
    control_binary:
      name: pre-rewrite
      sha256: a8192e6426bf5d358a803ce66c2ba845bd18f0d3ab57025d37cfd59149205b8d
      size_bytes: 1561440
      args: []
    candidate_binary:
      name: pr51
      sha256: 1fc84f5f388b5a12d449f26095076b95a54ce495adcb0ef22e988a74ce7abb4b
      size_bytes: 1826000
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-initial-three-way.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 68414646.0
          candidate_median: 74092166.5
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.015
          change_pct: 8.213
          ci95_low_pct: 1.187
          ci95_high_pct: 11.865
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 4
        component_ns:
          control_median: 48027041.5
          candidate_median: 53118750.0
          control_p95_over_median: 1.09
          candidate_p95_over_median: 1.004
          change_pct: 10.903
          ci95_low_pct: -1.158
          ci95_high_pct: 13.73
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        cpu_ns:
          control_median: 296934500.0
          candidate_median: 357323500.0
          control_p95_over_median: 1.073
          candidate_p95_over_median: 1.01
          change_pct: 20.23
          ci95_low_pct: 11.801
          ci95_high_pct: 27.21
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 4
        user_cpu_ns:
          control_median: 31608000.0
          candidate_median: 77334000.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.006
          change_pct: 144.982
          ci95_low_pct: 139.702
          ci95_high_pct: 145.781
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 4
        system_cpu_ns:
          control_median: 265385500.0
          candidate_median: 279574000.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.016
          change_pct: 5.753
          ci95_low_pct: -2.979
          ci95_high_pct: 12.233
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        peak_rss_bytes:
          control_median: 10829824.0
          candidate_median: 13008896.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.026
          change_pct: 21.22
          ci95_low_pct: 17.825
          ci95_high_pct: 21.461
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 4
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes exceeds its +5% regression limit"
          - "minor_faults exceeds its +10% regression limit"
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
          minor_faults: rejected
          peak_rss_bytes: rejected
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 59034750.0
          candidate_median: 63563541.5
          control_p95_over_median: 1.098
          candidate_p95_over_median: 1.047
          change_pct: 7.678
          ci95_low_pct: 2.688
          ci95_high_pct: 11.186
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 4
        component_ns:
          control_median: 54824167.0
          candidate_median: 59288687.5
          control_p95_over_median: 1.093
          candidate_p95_over_median: 1.031
          change_pct: 8.15
          ci95_low_pct: 1.985
          ci95_high_pct: 13.001
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 4
        cpu_ns:
          control_median: 310367500.0
          candidate_median: 365564000.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.025
          change_pct: 19.198
          ci95_low_pct: 4.038
          ci95_high_pct: 24.61
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 4
        user_cpu_ns:
          control_median: 17726000.0
          candidate_median: 65429500.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.009
          change_pct: 264.508
          ci95_low_pct: 233.411
          ci95_high_pct: 288.529
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 4
        system_cpu_ns:
          control_median: 292836500.0
          candidate_median: 300065500.0
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.035
          change_pct: 4.088
          ci95_low_pct: -10.325
          ci95_high_pct: 8.943
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        peak_rss_bytes:
          control_median: 11952128.0
          candidate_median: 14606336.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.032
          change_pct: 22.646
          ci95_low_pct: 21.154
          ci95_high_pct: 23.472
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 4
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes exceeds its +5% regression limit"
          - "minor_faults exceeds its +10% regression limit"
          - voluntary_context_switches is missing a paired percent interval
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: inconclusive
          major_faults: within-limit
          minor_faults: rejected
          peak_rss_bytes: rejected
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 28439
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Measures the complete PR #51 stack against its pre-rewrite parent; this record introduces no implementation."
  verdict:
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 7.678
    reason: "On this 11,142-entry exploratory subject, PR #51 remained 7.68% slower on default-tree and 8.21% slower on cold-scan-index; the run establishes the local gap but is not claim-grade."
    commit: e8f1bed
---
# PR #51 residual reproduced on the current registry tree

## Question

Does the current local registry tree reproduce the detached one-shot regression measured
in the PR #51 review?

## Method

The run interleaved immutable release probes for the pre-rewrite main commit, PR #51,
and the correctness-fixed branch on one unchanged 11,142-entry tree.
This record selects the pre-rewrite and PR #51 variants.
Every sample produced the same index digest and entry totals.

The host was uncontrolled and the run used four pairs, so the result locates the gap but
does not make a final parity claim.

## Result

PR #51 was 7.68% slower on `default-tree` and 8.21% slower on `cold-scan-index` by
median wall time. Its engine component was 8.15% and 10.90% slower, respectively.
The wall-time intervals exclude zero, while the cold component interval does not.

## Decision

Use this run as the local pre-optimization baseline and keep the parity bead open.
The five-job correctness baseline and scoped counters own mechanism attribution; a
quiet-host run with at least 12 pairs owns the final verdict.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
