---
title: Correctness fixes preserve the streaming performance baseline
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-076
  title: Correctness fixes preserve the streaming performance baseline
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
    control: "PR #51 head plus scoped instrumentation"
    candidate: snapshot and path correctness fixes plus scoped instrumentation
    control_binary:
      name: pr51
      sha256: b78c890e20d32ddbfa11ae058bf2f27a84ab9c8650abe0098c304cb5e6cb179e
      size_bytes: 2107184
      args: []
    candidate_binary:
      name: correctness
      sha256: 860c849a3cdf7e51f8981825dfbb80b1a25101b598d00d920bae23c717da2fd6
      size_bytes: 2107184
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-five-job-correctness-baseline.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 86722187.0
          candidate_median: 76499895.5
          control_p95_over_median: 1.142
          candidate_p95_over_median: 1.046
          change_pct: -10.056
          ci95_low_pct: -25.064
          ci95_high_pct: 9.068
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        component_ns:
          control_median: 64962812.0
          candidate_median: 56029270.5
          control_p95_over_median: 1.197
          candidate_p95_over_median: 1.056
          change_pct: -11.45
          ci95_low_pct: -31.424
          ci95_high_pct: 14.217
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        cpu_ns:
          control_median: 425276500.0
          candidate_median: 383959500.0
          control_p95_over_median: 1.157
          candidate_p95_over_median: 1.065
          change_pct: -9.701
          ci95_low_pct: -24.722
          ci95_high_pct: 18.133
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        user_cpu_ns:
          control_median: 85210000.0
          candidate_median: 76444000.0
          control_p95_over_median: 1.128
          candidate_p95_over_median: 1.013
          change_pct: -9.645
          ci95_low_pct: -19.468
          ci95_high_pct: -0.701
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 4
        system_cpu_ns:
          control_median: 338456500.0
          candidate_median: 307605000.0
          control_p95_over_median: 1.18
          candidate_p95_over_median: 1.081
          change_pct: -9.508
          ci95_low_pct: -26.312
          ci95_high_pct: 23.521
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        peak_rss_bytes:
          control_median: 13393920.0
          candidate_median: 13049856.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.004
          change_pct: -2.568
          ci95_low_pct: -5.656
          ci95_high_pct: -0.621
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 4
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
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
          control_median: 60435916.5
          candidate_median: 60263417.0
          control_p95_over_median: 1.099
          candidate_p95_over_median: 1.017
          change_pct: 0.294
          ci95_low_pct: -10.255
          ci95_high_pct: 3.082
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        component_ns:
          control_median: 56182333.0
          candidate_median: 56031708.5
          control_p95_over_median: 1.106
          candidate_p95_over_median: 1.016
          change_pct: 0.256
          ci95_low_pct: -10.905
          ci95_high_pct: 3.205
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        cpu_ns:
          control_median: 352624000.0
          candidate_median: 345533500.0
          control_p95_over_median: 1.116
          candidate_p95_over_median: 1.019
          change_pct: -2.245
          ci95_low_pct: -13.336
          ci95_high_pct: 3.774
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        user_cpu_ns:
          control_median: 62273000.0
          candidate_median: 63348000.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.034
          change_pct: 2.46
          ci95_low_pct: 0.649
          ci95_high_pct: 4.267
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 4
        system_cpu_ns:
          control_median: 290363000.0
          candidate_median: 282286000.0
          control_p95_over_median: 1.139
          candidate_p95_over_median: 1.015
          change_pct: -3.231
          ci95_low_pct: -15.982
          ci95_high_pct: 3.662
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        peak_rss_bytes:
          control_median: 14974976.0
          candidate_median: 14893056.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.01
          change_pct: -0.875
          ci95_low_pct: -1.713
          ci95_high_pct: 0.11
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
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
    - job: delta-apply-batched
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 579192541.5
          candidate_median: 579127271.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.018
          change_pct: 0.517
          ci95_low_pct: -2.509
          ci95_high_pct: 2.368
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
        component_ns:
          control_median: 329386313.0
          candidate_median: 330948520.5
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.025
          change_pct: 0.621
          ci95_low_pct: -3.178
          ci95_high_pct: 4.288
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        cpu_ns:
          control_median: 576205500.0
          candidate_median: 576495000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.017
          change_pct: 0.593
          ci95_low_pct: -1.977
          ci95_high_pct: 2.124
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
        user_cpu_ns:
          control_median: 556664000.0
          candidate_median: 559047000.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.017
          change_pct: 0.479
          ci95_low_pct: -1.638
          ci95_high_pct: 2.548
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
        system_cpu_ns:
          control_median: 17328500.0
          candidate_median: 16671500.0
          control_p95_over_median: 1.24
          candidate_p95_over_median: 1.149
          change_pct: -3.553
          ci95_low_pct: -11.051
          ci95_high_pct: 5.376
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        blocked_ns:
          control_median: 3652208.0
          candidate_median: 3113979.0
          control_p95_over_median: 1.494
          candidate_p95_over_median: 1.05
          change_pct: -8.292
          ci95_low_pct: -59.534
          ci95_high_pct: 87.551
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        peak_rss_bytes:
          control_median: 122445824.0
          candidate_median: 121028608.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.007
          change_pct: -1.328
          ci95_low_pct: -1.752
          ci95_high_pct: 0.477
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
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
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: delta-apply-large
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 659440916.5
          candidate_median: 667989312.5
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.004
          change_pct: 1.084
          ci95_low_pct: -2.893
          ci95_high_pct: 4.581
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        component_ns:
          control_median: 414588687.5
          candidate_median: 421790229.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.004
          change_pct: 1.631
          ci95_low_pct: -2.609
          ci95_high_pct: 5.444
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        cpu_ns:
          control_median: 656869500.0
          candidate_median: 665412500.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.003
          change_pct: 1.171
          ci95_low_pct: -3.027
          ci95_high_pct: 4.517
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        user_cpu_ns:
          control_median: 636028500.0
          candidate_median: 645589500.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.003
          change_pct: 1.337
          ci95_low_pct: -2.667
          ci95_high_pct: 4.284
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        system_cpu_ns:
          control_median: 20635000.0
          candidate_median: 19434500.0
          control_p95_over_median: 1.048
          candidate_p95_over_median: 1.08
          change_pct: -4.017
          ci95_low_pct: -13.74
          ci95_high_pct: 12.581
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        blocked_ns:
          control_median: 2896646.0
          candidate_median: 2859146.5
          control_p95_over_median: 1.09
          candidate_p95_over_median: 1.132
          change_pct: 5.722
          ci95_low_pct: -26.998
          ci95_high_pct: 41.885
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        peak_rss_bytes:
          control_median: 178012160.0
          candidate_median: 177922048.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.0
          change_pct: -0.051
          ci95_low_pct: -1.228
          ci95_high_pct: 1.78
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
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
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 361985833.5
          candidate_median: 349339021.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.024
          change_pct: -2.837
          ci95_low_pct: -6.557
          ci95_high_pct: 2.448
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
        component_ns:
          control_median: 172190562.5
          candidate_median: 172513708.5
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.025
          change_pct: 0.542
          ci95_low_pct: -1.623
          ci95_high_pct: 2.389
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
        cpu_ns:
          control_median: 380766500.0
          candidate_median: 369285000.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.021
          change_pct: -2.842
          ci95_low_pct: -5.635
          ci95_high_pct: 2.477
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
        user_cpu_ns:
          control_median: 215104500.0
          candidate_median: 215314500.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.007
          change_pct: 0.098
          ci95_low_pct: -1.606
          ci95_high_pct: 1.355
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
        system_cpu_ns:
          control_median: 165228000.0
          candidate_median: 154606000.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.035
          change_pct: -5.644
          ci95_low_pct: -12.261
          ci95_high_pct: 4.092
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 4
        peak_rss_bytes:
          control_median: 42024960.0
          candidate_median: 42319872.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.004
          change_pct: 0.2
          ci95_low_pct: -0.779
          ci95_high_pct: 2.655
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 4
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
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
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 2396
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: Comparison isolates the two correctness fixes; the same instrumentation commit is cherry-picked onto both binaries.
  verdict:
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 0.294
    reason: "All five exact jobs passed their semantic oracles; default-tree moved +0.29%, opened discovery -2.84% wall and +0.54% component, and both public delta jobs stayed within 1.1% wall, with four-pair intervals crossing zero."
    commit: b5d9ba4
---
# Correctness fixes preserve the streaming performance baseline

## Question

Did the snapshot-scope and encoded-path correctness fixes create a separate performance
change that Phase 2 must account for?

## Method

The same scoped-instrumentation commit was applied to PR #51 and to the
correctness-fixed branch.
The run interleaved all five jobs on one unchanged real tree.
Every sample had to satisfy the exact index oracle; the opened and synthetic delta jobs
also had to satisfy their independent commit-shape oracles.

The host was uncontrolled and the comparison used four pairs, so this record establishes
the working baseline rather than an equivalence claim.

## Result

All semantic oracles passed.
`default-tree` moved 0.29% slower, opened discovery moved 2.84% faster by wall time and
0.54% slower by component time, and both public delta jobs stayed within 1.1% wall time.
Every four-pair interval crossed zero.

## Decision

Treat instrumented PR #51 and the correctness-fixed branch as the same Phase 2
performance baseline.
The correctness changes stay in place, and optimization experiments compare against the
correctness-fixed commit rather than weakening either public invariant.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
