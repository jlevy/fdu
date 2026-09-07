---
title: Select detached consequences once per batch
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-077
  title: Select detached consequences once per batch
  date: "2026-09-01"
  hypotheses:
    - H91
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
    trials: 12
    warmups: 3
    interleaved: true
    control: correctness and instrumentation baseline at 1393d31
    candidate: zero-sized detached consequence sink at da5b8bc
    control_binary:
      name: control
      sha256: 860c849a3cdf7e51f8981825dfbb80b1a25101b598d00d920bae23c717da2fd6
      size_bytes: 2107184
      args: []
    candidate_binary:
      name: candidate
      sha256: 06f95875c2a5c0081ca62f1027415aa8406531cb5991fa90c041d94469dde57b
      size_bytes: 2123728
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-detached-consequence-sink.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 75431145.5
          candidate_median: 76980166.5
          control_p95_over_median: 1.103
          candidate_p95_over_median: 1.187
          change_pct: -3.014
          ci95_low_pct: -4.455
          ci95_high_pct: 7.161
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 54144083.5
          candidate_median: 55725791.5
          control_p95_over_median: 1.143
          candidate_p95_over_median: 1.263
          change_pct: -4.226
          ci95_low_pct: -6.354
          ci95_high_pct: 9.208
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 367422500.0
          candidate_median: 346332000.0
          control_p95_over_median: 1.17
          candidate_p95_over_median: 1.337
          change_pct: -6.601
          ci95_low_pct: -10.829
          ci95_high_pct: 5.031
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 77873500.0
          candidate_median: 56607000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.138
          change_pct: -27.758
          ci95_low_pct: -28.679
          ci95_high_pct: -26.433
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 289189500.0
          candidate_median: 290449500.0
          control_p95_over_median: 1.217
          candidate_p95_over_median: 1.391
          change_pct: -0.717
          ci95_low_pct: -6.628
          ci95_high_pct: 13.082
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 13049856.0
          candidate_median: 12541952.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.018
          change_pct: -3.578
          ci95_low_pct: -4.437
          ci95_high_pct: -2.797
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - voluntary_context_switches is missing a paired percent interval
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: rejected
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
          control_median: 59698666.5
          candidate_median: 55632645.5
          control_p95_over_median: 1.216
          candidate_p95_over_median: 1.048
          change_pct: -6.568
          ci95_low_pct: -11.197
          ci95_high_pct: -4.487
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 55530208.5
          candidate_median: 51491625.5
          control_p95_over_median: 1.229
          candidate_p95_over_median: 1.051
          change_pct: -7.243
          ci95_low_pct: -11.959
          ci95_high_pct: -4.857
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 346862500.0
          candidate_median: 315969000.0
          control_p95_over_median: 1.219
          candidate_p95_over_median: 1.048
          change_pct: -6.55
          ci95_low_pct: -12.768
          ci95_high_pct: -5.446
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 63445000.0
          candidate_median: 39864500.0
          control_p95_over_median: 1.098
          candidate_p95_over_median: 1.019
          change_pct: -36.768
          ci95_low_pct: -37.915
          ci95_high_pct: -36.267
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 283471000.0
          candidate_median: 276190500.0
          control_p95_over_median: 1.246
          candidate_p95_over_median: 1.055
          change_pct: 0.18
          ci95_low_pct: -7.243
          ci95_high_pct: 1.712
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 14852096.0
          candidate_median: 14393344.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.012
          change_pct: -2.915
          ci95_low_pct: -4.208
          ci95_high_pct: -2.343
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - voluntary_context_switches is missing a paired percent interval
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: rejected
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
          control_median: 562641041.5
          candidate_median: 565001625.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.028
          change_pct: 0.314
          ci95_low_pct: -0.173
          ci95_high_pct: 1.867
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 323317687.5
          candidate_median: 324654542.0
          control_p95_over_median: 1.028
          candidate_p95_over_median: 1.026
          change_pct: 0.48
          ci95_low_pct: -0.064
          ci95_high_pct: 1.797
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 560061000.0
          candidate_median: 561648000.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.027
          change_pct: 0.331
          ci95_low_pct: -0.188
          ci95_high_pct: 1.572
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 546453500.0
          candidate_median: 547634500.0
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.029
          change_pct: 0.254
          ci95_low_pct: -0.259
          ci95_high_pct: 1.488
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 13243500.0
          candidate_median: 13800500.0
          control_p95_over_median: 1.152
          candidate_p95_over_median: 1.082
          change_pct: 2.234
          ci95_low_pct: -4.857
          ci95_high_pct: 9.213
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 3067229.0
          candidate_median: 3256958.5
          control_p95_over_median: 1.132
          candidate_p95_over_median: 1.166
          change_pct: 2.61
          ci95_low_pct: -4.932
          ci95_high_pct: 21.702
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 120184832.0
          candidate_median: 120217600.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.011
          change_pct: 0.027
          ci95_low_pct: -0.454
          ci95_high_pct: 0.505
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: delta-apply-large
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 657851771.0
          candidate_median: 656133666.5
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.063
          change_pct: 0.038
          ci95_low_pct: -0.849
          ci95_high_pct: 1.579
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 415272125.0
          candidate_median: 415662291.5
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.06
          change_pct: 0.33
          ci95_low_pct: -0.859
          ci95_high_pct: 1.765
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 654276000.0
          candidate_median: 653129000.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.062
          change_pct: 0.103
          ci95_low_pct: -0.755
          ci95_high_pct: 1.601
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 629607500.0
          candidate_median: 629351000.0
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.062
          change_pct: 0.093
          ci95_low_pct: -0.435
          ci95_high_pct: 1.286
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 22982000.0
          candidate_median: 22784500.0
          control_p95_over_median: 1.157
          candidate_p95_over_median: 1.095
          change_pct: -4.16
          ci95_low_pct: -6.682
          ci95_high_pct: 9.689
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 3237354.0
          candidate_median: 3213896.0
          control_p95_over_median: 1.298
          candidate_p95_over_median: 1.199
          change_pct: -9.42
          ci95_low_pct: -18.311
          ci95_high_pct: -0.939
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 177922048.0
          candidate_median: 177979392.0
          control_p95_over_median: 1.0
          candidate_p95_over_median: 1.0
          change_pct: 0.032
          ci95_low_pct: 0.009
          ci95_high_pct: 0.06
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
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
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 349646750.0
          candidate_median: 345570500.0
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.1
          change_pct: 0.031
          ci95_low_pct: -1.657
          ci95_high_pct: 2.584
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 172293958.5
          candidate_median: 173329687.0
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.116
          change_pct: 2.052
          ci95_low_pct: -0.596
          ci95_high_pct: 3.85
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 366409000.0
          candidate_median: 363390000.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.07
          change_pct: -0.051
          ci95_low_pct: -1.862
          ci95_high_pct: 2.764
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 208362000.0
          candidate_median: 205471000.0
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.022
          change_pct: -1.682
          ci95_low_pct: -2.784
          ci95_high_pct: -0.12
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 158621500.0
          candidate_median: 158780000.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.116
          change_pct: 1.278
          ci95_low_pct: -1.389
          ci95_high_pct: 7.738
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 41664512.0
          candidate_median: 41426944.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.004
          change_pct: -0.669
          ci95_low_pct: -1.307
          ci95_high_pct: -0.118
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
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
  reference_tools:
    - name: dust
      wall_ns_median: 94408521.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 281
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "One private zero-sized sink and one generic reducer; no per-entry lifecycle branch, dependency, unsafe block, public mode, or duplicated mutation engine."
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -6.568
    reason: "default-tree improved 6.57% with a paired 95% interval of -11.20% to -4.49%; component time improved 7.24%, detached component allocations fell 33.7%, and all exact streaming controls preserved their semantic oracles without a material timing shift."
    commit: da5b8bc
---
# Select detached consequences once per batch

## Hypothesis

H91: Detached baseline construction pays for exact commit consequences that cannot
escape a one-shot index.
Selecting a zero-sized consequence sink once per batch should reduce `default-tree`
component time and allocations without changing facts, stats, or exact streaming
history.

## What was tried

The reducer became generic over one private consequence sink.
`ExactConsequences` retains effective changes and derives the same impact, commit, and
journal state as before.
`NoConsequences` accepts the same reducer events but retains nothing.
The lifecycle chooses one sink at the batch boundary, so there is no per-entry mode
branch and no second mutation engine.

The release candidate at `da5b8bc` was compared with the instrumented correctness
baseline at `1393d31` over 12 interleaved pairs for all five campaign jobs.
Exact jobs had to preserve both the independent engine digest and exact commit digest.

## What the numbers said

Detached scoped allocations fell from 244,369 to 162,025, a 33.7% reduction, and scoped
allocated bytes fell 24.5%. Effective paths, impact derivation, compatibility
projection, and journal work all reached zero on the detached path.

`default-tree` wall time improved 6.57%, with a paired 95% interval of -11.20% to
-4.49%; component time improved 7.24%, with an interval of -11.96% to -4.86%. The
isolated cold-scan medians improved, but their intervals crossed zero because a few
filesystem-heavy samples dominated that shorter job.
Opened discovery and both public delta controls preserved their exact digests and showed
no material timing movement.

## Verdict

**ACCEPTED.** The change removes work that cannot be observed, clears the project’s 3%
timing bar on the primary one-shot job, materially reduces allocation, and preserves a
single reducer and every exact semantic oracle.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
