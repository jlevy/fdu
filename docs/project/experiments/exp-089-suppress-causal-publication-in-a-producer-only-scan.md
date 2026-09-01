---
title: Suppress causal publication in a producer-only scan
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-089
  title: Suppress causal publication in a producer-only scan
  date: "2026-09-01"
  hypotheses:
    - H106
  subject:
    tree_label: metabrowser-current
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 58bc2ea1deb0e212c7368177328184d412a1b2da24be8a77a7985a4bf6d4bc64
    tree_provenance: Live local metabrowser source checkout observed in place; working-tree state and exact filesystem metadata are not reconstructible.
    tree_reconstructible: false
    tree_entries: 113794
    tree_directories: 15221
    tree_files: 98525
    tree_symlinks: 48
    tree_apparent_bytes: 2311017461
    tree_allocated_bytes: 2591035392
    tree_max_depth: 21
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
    control: phase-instrumented causal streaming control at 80e5897
    candidate: unordered producer-only diagnostic
    control_binary:
      name: control
      sha256: 8ed1536071efa35821d449c97279092e9f830c722d6e9480283a856580e4e8e1
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: 7259a0e8e1b2a3370d2ea81d054b7a83e1ec444d9548e730aad05e697320bd62
      size_bytes: 2156752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h106-unordered-producer.json
  results:
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 770273166.5
          candidate_median: 771909104.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.122
          change_pct: 0.376
          ci95_low_pct: -0.121
          ci95_high_pct: 0.963
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 314397874.5
          candidate_median: 315546750.0
          control_p95_over_median: 1.059
          candidate_p95_over_median: 1.082
          change_pct: 0.68
          ci95_low_pct: -1.041
          ci95_high_pct: 2.13
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 3542338000.0
          candidate_median: 3531647000.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.031
          change_pct: 0.296
          ci95_low_pct: -0.352
          ci95_high_pct: 5.254
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 570276000.0
          candidate_median: 556374000.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.03
          change_pct: -0.886
          ci95_low_pct: -2.768
          ci95_high_pct: 1.889
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 2968456000.0
          candidate_median: 2967689500.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.038
          change_pct: 0.614
          ci95_low_pct: 0.254
          ci95_high_pct: 5.146
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 79953920.0
          candidate_median: 80445440.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.007
          change_pct: 0.051
          ci95_low_pct: -0.852
          ci95_high_pct: 1.171
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
  reference_tools: []
  complexity:
    lines_changed: 43
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - The diagnostic stream can publish children before parents and is therefore safe only for its order-insensitive summary sink.
    notes: "The diagnostic isolated causal publication with a small threaded flag, but the flat result rules out the much larger pending-child builder it was intended to justify."
  verdict:
    decision: rejected
    primary_job: cold-scan-producer
    primary_metric: component_ns
    change_pct: 0.68
    reason: "Producer component time changed +0.68% with a 95% interval of -1.04% to +2.13%, missing the 3% gate and establishing no improvement."
    commit: null
---
# Suppress causal publication in a producer-only scan

## Hypothesis

H106: matched counter-disabled profiles sampled the current `scan_into_index` consumer
1,182 times versus 822 for the pre-rewrite control, but earlier experiments showed that
removing preparation or reducing application calls did not shorten wall time.
If the remaining cost is interaction between consumer work and the parent-before-child
publication barrier, suppressing the early causal flush in a producer-only diagnostic
should improve `cold-scan-producer` component time by at least 3%.

## What was tried

The diagnostic let workers make discovered directories claimable without first
publishing the batch that contained their parent facts.
Workers retained the configured batch limit and all normal admission, metadata, and
traversal logic.
The unordered stream was consumed only into an order-insensitive compact
summary; a separate exact scan kept the causal path and verified the complete summary
outside the component timer.
No index was built from the unordered stream.

## What the numbers said

The 12-pair comparison changed producer component time by +0.68%, with a paired 95%
interval from -1.04% to +2.13%. Whole-process wall changed +0.38%, with an interval from
-0.12% to +0.96%. Every compact summary matched the separate exact validation scan.

The result agrees with the earlier pre-rewrite producer comparison: publication
frequency is not a material part of producer component time on this subject.
The extra one-shot cost is therefore not explained by the causal queue barrier, even
when that barrier is removed rather than merely coalesced downstream.

## Verdict

**REJECTED.** The diagnostic missed its 3% component gate and established no
improvement. Remove the unordered publication spike and do not build a pending-child
one-shot reducer around this mechanism.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
