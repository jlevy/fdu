---
title: Coalesce causal scanner fragments in the one-shot builder
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-088
  title: Coalesce causal scanner fragments in the one-shot builder
  date: "2026-09-01"
  hypotheses:
    - H105
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
    trials: 6
    warmups: 3
    interleaved: true
    control: phase-instrumented streaming control at 80e5897
    candidate: one-shot causal-fragment coalescer
    control_binary:
      name: control
      sha256: 8ed1536071efa35821d449c97279092e9f830c722d6e9480283a856580e4e8e1
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: 1c65d1740e231094c586dda673b5341bce9d36b85f07843794a7a42bb39d7351
      size_bytes: 2156752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h105-coalesced-screen.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 584853375.0
          candidate_median: 580982979.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.03
          change_pct: -0.005
          ci95_low_pct: -2.049
          ci95_high_pct: 1.416
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        component_ns:
          control_median: 341060000.0
          candidate_median: 337474937.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.052
          change_pct: -0.197
          ci95_low_pct: -2.644
          ci95_high_pct: 2.105
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        cpu_ns:
          control_median: 2400452000.0
          candidate_median: 2387478500.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.012
          change_pct: -0.023
          ci95_low_pct: -1.377
          ci95_high_pct: 1.574
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        user_cpu_ns:
          control_median: 440324000.0
          candidate_median: 489165500.0
          control_p95_over_median: 1.198
          candidate_p95_over_median: 1.14
          change_pct: 9.042
          ci95_low_pct: 5.882
          ci95_high_pct: 21.962
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 6
        system_cpu_ns:
          control_median: 1955755000.0
          candidate_median: 1889734000.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.024
          change_pct: -2.67
          ci95_low_pct: -4.194
          ci95_high_pct: -1.406
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 6
        peak_rss_bytes:
          control_median: 77881344.0
          candidate_median: 79609856.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.022
          change_pct: 2.023
          ci95_low_pct: 0.987
          ci95_high_pct: 4.38
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 6
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 362488145.5
          candidate_median: 360405187.5
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.048
          change_pct: 0.126
          ci95_low_pct: -1.08
          ci95_high_pct: 2.288
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        component_ns:
          control_median: 357393458.0
          candidate_median: 355333041.5
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.049
          change_pct: 0.166
          ci95_low_pct: -1.109
          ci95_high_pct: 2.297
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        cpu_ns:
          control_median: 2097619500.0
          candidate_median: 2130547000.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.04
          change_pct: 1.904
          ci95_low_pct: -0.406
          ci95_high_pct: 4.048
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 6
        user_cpu_ns:
          control_median: 205248500.0
          candidate_median: 255417500.0
          control_p95_over_median: 1.518
          candidate_p95_over_median: 1.025
          change_pct: 24.876
          ci95_low_pct: 2.55
          ci95_high_pct: 26.547
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 6
        system_cpu_ns:
          control_median: 1884873000.0
          candidate_median: 1872572500.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.047
          change_pct: -0.321
          ci95_low_pct: -1.579
          ci95_high_pct: 2.71
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        peak_rss_bytes:
          control_median: 89997312.0
          candidate_median: 90931200.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.004
          change_pct: 0.863
          ci95_low_pct: -0.091
          ci95_high_pct: 2.191
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
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
      wall_ns_median: 454135770.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 139
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - The accumulator retains one additional scanner batch until the target or end of stream.
    notes: "The coalescer reduced baseline applications to near the configured minimum, but larger batches made causal-parent preparation more expensive and wall time stayed flat."
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 0.126
    reason: "Default-tree wall changed +0.13% with a 95% interval of -1.08% to +2.29%, while cold-scan-index was flat; the candidate missed the 3% structural gate."
    commit: null
---
# Coalesce causal scanner fragments in the one-shot builder

## Hypothesis

H105: the parent-before-child correctness fix publishes about 2,650 causal scanner
fragments per scan on the 113,794-entry subject, while the configured 1,024-operation
target needs roughly 112 full batches.
Coalescing adjacent fragments only in the one-shot builder should reduce baseline
applications to within 10% of that minimum and improve `default-tree` wall time by at
least 3%, without changing any public streaming publication cadence.

## What was tried

`scan_into_index` and its diagnostic twin accumulated adjacent ordered scanner fragments
up to the existing batch target before invoking the unchanged atomic baseline reducer.
Public `scan`, opened discovery, refresh, and watch retained their existing fragment
boundaries. A focused test proved that coalescing preserved causal operation order, and
the no-feature and all-feature targeted suites plus diagnostic parity passed.

## What the numbers said

The mechanism worked but did not improve the critical path.
Enabled repeat-10 counters reduced baseline applications from about 2,670 to about 124
per scan, close to the configured-batch minimum.
`default-tree` nevertheless changed +0.13%, with a paired 95% interval from -1.08% to
+2.29%. `cold-scan-index` changed -0.01%, with an interval from -2.05% to +1.42%.

Larger prepared batches also exposed a contrary cost.
Scanner preparation increased from about 31.7 ms to 105.7 ms per scan because the
correctness check that locates an earlier parent reverse-scans a larger batch.
Reduction was about 99.9 ms per scan.
Reducing the number of reducer calls therefore moved work into preparation without
shortening overlapped wall time.

## Verdict

**REJECTED.** The candidate missed the 3% gate and established no wall-time improvement
in either job. Remove the coalescer.
Producer-only parity and this result jointly show that causal channel handoffs and
reducer-call count are not the remaining one-shot bottleneck; the next experiment must
attribute per-entry baseline work before changing the streaming boundary again.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
