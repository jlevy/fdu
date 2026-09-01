---
title: Remove the eager compatibility projection
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-078
  title: Remove the eager compatibility projection
  date: "2026-09-01"
  hypotheses:
    - H92
  subject:
    tree_label: cargo-registry-src-v2
    tree_root_id: 0d6ac3b56b7696752b6af951b3802fd843b8d1235fa49cad9f2a2214cd8e403b
    tree_engine_digest: 1c2f63e8a0cb7ff48e2ba2380715832093ef125973190619c9973a79aebeea63
    tree_provenance: Live local Cargo registry source cache observed in place after the original corpus changed during validation; package-manager state and exact tree shape are not reconstructible.
    tree_reconstructible: false
    tree_entries: 11141
    tree_directories: 2240
    tree_files: 8901
    tree_symlinks: 0
    tree_apparent_bytes: 179605080
    tree_allocated_bytes: 203902976
    tree_max_depth: 9
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
    control: detached consequence sink at da5b8bc
    candidate: single exact Commit representation at db18e5e
    control_binary:
      name: control
      sha256: 06f95875c2a5c0081ca62f1027415aa8406531cb5991fa90c041d94469dde57b
      size_bytes: 2123728
      args: []
    candidate_binary:
      name: candidate
      sha256: a41678887ad74a7ea2b3fba3ec04e2cbebd48edf980871edc20d5eb761feb629
      size_bytes: 2123712
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-compatibility-projection-removal.json
  results:
    - job: delta-apply-batched
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 592571833.0
          candidate_median: 586073958.5
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.005
          change_pct: -1.053
          ci95_low_pct: -3.641
          ci95_high_pct: -0.702
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 341610438.0
          candidate_median: 333885104.5
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.004
          change_pct: -2.457
          ci95_low_pct: -4.998
          ci95_high_pct: -2.104
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 589436000.0
          candidate_median: 582411000.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.007
          change_pct: -1.087
          ci95_low_pct: -3.514
          ci95_high_pct: -0.589
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 573124500.0
          candidate_median: 566865000.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.006
          change_pct: -1.062
          ci95_low_pct: -3.685
          ci95_high_pct: -0.7
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 15690000.0
          candidate_median: 15648500.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.037
          change_pct: 0.18
          ci95_low_pct: -6.047
          ci95_high_pct: 5.332
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 3319937.5
          candidate_median: 3124770.5
          control_p95_over_median: 1.181
          candidate_p95_over_median: 1.159
          change_pct: -7.613
          ci95_low_pct: -38.253
          ci95_high_pct: -0.759
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 121790464.0
          candidate_median: 120102912.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.013
          change_pct: -1.049
          ci95_low_pct: -2.181
          ci95_high_pct: -0.368
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
    - job: delta-apply-large
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 651625021.0
          candidate_median: 641883312.5
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.017
          change_pct: -1.551
          ci95_low_pct: -2.46
          ci95_high_pct: -1.296
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 411657021.0
          candidate_median: 402533000.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.025
          change_pct: -2.375
          ci95_low_pct: -3.647
          ci95_high_pct: -2.031
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 648589000.0
          candidate_median: 638809500.0
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.012
          change_pct: -1.501
          ci95_low_pct: -2.437
          ci95_high_pct: -1.31
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 630132000.0
          candidate_median: 620965500.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.013
          change_pct: -1.402
          ci95_low_pct: -2.355
          ci95_high_pct: -1.215
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 17882500.0
          candidate_median: 17231000.0
          control_p95_over_median: 1.247
          candidate_p95_over_median: 1.191
          change_pct: -5.719
          ci95_low_pct: -10.774
          ci95_high_pct: 0.847
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        blocked_ns:
          control_median: 3065541.5
          candidate_median: 2884937.5
          control_p95_over_median: 1.227
          candidate_p95_over_median: 1.113
          change_pct: -3.817
          ci95_low_pct: -10.601
          ci95_high_pct: 2.352
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 177954816.0
          candidate_median: 165355520.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.02
          change_pct: -7.094
          ci95_low_pct: -7.499
          ci95_high_pct: -6.703
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
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
          control_median: 301750188.0
          candidate_median: 301058145.5
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.013
          change_pct: -0.781
          ci95_low_pct: -2.14
          ci95_high_pct: 0.326
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 154968916.5
          candidate_median: 153389958.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.023
          change_pct: -1.496
          ci95_low_pct: -2.203
          ci95_high_pct: -0.64
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 318007000.0
          candidate_median: 316366500.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.015
          change_pct: -1.071
          ci95_low_pct: -2.381
          ci95_high_pct: -0.091
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 170733000.0
          candidate_median: 168322500.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.018
          change_pct: -1.571
          ci95_low_pct: -1.951
          ci95_high_pct: -1.084
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 147865500.0
          candidate_median: 147349000.0
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.02
          change_pct: -0.452
          ci95_low_pct: -2.668
          ci95_high_pct: 1.386
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 37478400.0
          candidate_median: 37666816.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.006
          change_pct: 0.59
          ci95_low_pct: -0.348
          ci95_high_pct: 0.964
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
      wall_ns_median: 93598812.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 288
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Deletes 128 net lines and one unreleased public compatibility type; adds no dependency, unsafe block, failure mode, or alternate reducer. The generic 3% threshold for added complexity does not apply to a strict simplification."
  verdict:
    decision: accepted
    primary_job: delta-apply-large
    primary_metric: wall_ns
    change_pct: -1.551
    reason: "Large-batch wall time improved 1.55% with a paired 95% interval of -2.46% to -1.30%, component time improved 2.38%, and peak RSS improved 7.09%; batched wall time also improved 1.05%, while opened wall time was noninferior and its component improved 1.50%."
    commit: db18e5e
---
# Remove the eager compatibility projection

## Hypothesis

H92: Exact application eagerly clones every entry path into an `AppliedDelta` even
though `Commit` already owns the complete effective change.
Removing that unreleased projection should reduce exact-path allocations, memory, and
component time while preserving the Python change-feed output and exact commit digests.

## What was tried

`AppliedDelta`, `Commit::applied_delta`, and the duplicate fields on `ApplyOutcome` and
`Since` were deleted.
Rust consumers now use exact `Commit` values directly.
The Python `since()` binding traverses `EffectiveChange` and emits the same
entry-operation dictionaries as before, still omitting control-only and reclassification
changes.

The saved sink-only release binary at `da5b8bc` was compared with the single-
representation candidate at `db18e5e` over 12 interleaved pairs.
The first attempt was discarded in full when the Cargo registry tree differed from its
preregistered fingerprint.
The run recorded here used a new fingerprint for the stable 11,141-entry tree, produced
no invalid sample or baseline drift, and passed every engine and exact commit oracle.

## What the numbers said

The 100,001-operation exact batch eliminated 100,002 scoped allocations, an 8.1%
reduction, and 13.4 MB of scoped allocation, a 6.9% reduction.
Its median wall time improved 1.55% with a paired 95% interval of -2.46% to -1.30%;
component time improved 2.38%, and peak RSS improved 7.09%. Repeated batches improved
1.05% in wall time and 2.46% in component time.
Opened discovery wall time was noninferior, while its component time improved 1.50% with
an interval entirely below zero.

The harness reported each wall result below its generic 3% threshold for accepting added
complexity. This experiment adds no complexity: it deletes 128 net lines, one public
compatibility type, and the second owned path vector.
The raw harness verdict is therefore retained as evidence, while the operator decision
applies the project’s simplification rule.

## Verdict

**ACCEPTED.** One exact representation preserves the complete streaming contract and
Python output while reducing source size, allocation, component time, and memory.
The 3% bar for taking on complexity does not justify retaining redundant machinery.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
