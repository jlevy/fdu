---
title: Scanner phase counters expose preparation without observer cost
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-086
  title: Scanner phase counters expose preparation without observer cost
  date: "2026-09-01"
  hypotheses:
    - H103
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
    control: streaming allocation guards at 3c0e1a2
    candidate: off-by-default scanner phase timing counters
    control_binary:
      name: control
      sha256: c313ecd326426dd24fc08ac256ecfb3174624bbe6153ba9beeab8b2681b87018
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: 8ed1536071efa35821d449c97279092e9f830c722d6e9480283a856580e4e8e1
      size_bytes: 2156752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h100-phase-instrumentation-overhead.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 583616500.0
          candidate_median: 572792270.5
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.019
          change_pct: -2.252
          ci95_low_pct: -3.475
          ci95_high_pct: -1.072
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 6
        component_ns:
          control_median: 345231791.5
          candidate_median: 334579271.0
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.008
          change_pct: -2.854
          ci95_low_pct: -4.727
          ci95_high_pct: -0.578
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 6
        cpu_ns:
          control_median: 2453074000.0
          candidate_median: 2365100000.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.019
          change_pct: -2.848
          ci95_low_pct: -4.069
          ci95_high_pct: -1.195
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 6
        user_cpu_ns:
          control_median: 430009000.0
          candidate_median: 430240000.0
          control_p95_over_median: 1.246
          candidate_p95_over_median: 1.156
          change_pct: -2.153
          ci95_low_pct: -5.321
          ci95_high_pct: 1.915
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        system_cpu_ns:
          control_median: 1975094000.0
          candidate_median: 1906153000.0
          control_p95_over_median: 1.037
          candidate_p95_over_median: 1.026
          change_pct: -2.936
          ci95_low_pct: -4.949
          ci95_high_pct: -0.711
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 6
        peak_rss_bytes:
          control_median: 77594624.0
          candidate_median: 77479936.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.014
          change_pct: -0.41
          ci95_low_pct: -0.908
          ci95_high_pct: 0.592
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 364334041.5
          candidate_median: 362853500.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.035
          change_pct: -0.12
          ci95_low_pct: -3.06
          ci95_high_pct: 2.396
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        component_ns:
          control_median: 359189874.5
          candidate_median: 357317645.5
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.036
          change_pct: -0.185
          ci95_low_pct: -3.22
          ci95_high_pct: 2.4
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        cpu_ns:
          control_median: 2088989500.0
          candidate_median: 2088437000.0
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.021
          change_pct: 0.097
          ci95_low_pct: -2.173
          ci95_high_pct: 1.763
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        user_cpu_ns:
          control_median: 210378500.0
          candidate_median: 209755000.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.247
          change_pct: -2.388
          ci95_low_pct: -6.202
          ci95_high_pct: 12.832
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 6
        system_cpu_ns:
          control_median: 1880095500.0
          candidate_median: 1875925500.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.021
          change_pct: -0.12
          ci95_low_pct: -2.161
          ci95_high_pct: 1.52
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        peak_rss_bytes:
          control_median: 89759744.0
          candidate_median: 89825280.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.006
          change_pct: 0.0
          ci95_low_pct: -1.177
          ci95_high_pct: 0.819
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
      wall_ns_median: 498544646.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 66
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: Adds three runtime-gated elapsed counters and no production dependency or unsafe code.
  verdict:
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -0.12
    reason: "With counters disabled, default-tree changed -0.12% with CI [-3.06%, +2.40%]; cold-scan-index changed -2.25%, below the 3% structural threshold. Enabled repeat-10 attribution measured 28.5 ms preparation and 82.3 ms reduction per scan, naming preparation as a viable next target."
    commit: c7b2120
---
# Scanner phase counters expose preparation without observer cost

## Hypothesis

H103: the sampling profile assigns only about 1% of self time to the index, but inlining
hides the inclusive cost of scanner preparation and reduction.
Runtime-gated elapsed counters around those phase boundaries should identify whether a
detached-only path can plausibly clear the 3% experiment threshold without measurably
changing ordinary runs while counters are disabled.

## What was tried

Three counters measure scanner-batch preparation, control projection, and reduction in
microseconds. They read the clock only when `FDU_COUNTERS=1`; an ordinary run performs
the same disabled check as the existing logical counters and no timing call.

A six-pair interleaved screen compared the disabled instrument with the immutable
pre-instrumentation binary on the 113,794-entry subject.
An enabled repeat-10 profile then accumulated the three phase totals over ten identical
scans.

## What the numbers said

With counters disabled, `default-tree` changed -0.12%, with a paired 95% interval from
-3.06% to +2.40%. `cold-scan-index` changed -2.25%; the result was below the 3% rule and
in the favorable direction.
The screen found no observer cost that would invalidate later timing.

With counters enabled, ten scans spent 285,370 microseconds preparing scanner batches,
822,869 microseconds reducing them, and 7 microseconds projecting the empty control
table. That is about 28.5 ms of preparation and 82.3 ms of reduction per scan.
The last repeated component took 210.4 ms, so preparation is large enough to justify a
detached-only experiment; control projection is not.

## Verdict

**BASELINE.** Retain the off-by-default timing counters.
They are neutral within this screen and replace an inlining-blind profile with exact
phase attribution. Use scanner preparation—not control projection—as the next
preregistered target.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
