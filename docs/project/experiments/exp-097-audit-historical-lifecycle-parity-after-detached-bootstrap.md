---
title: Audit historical lifecycle parity after detached bootstrap
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-097
  title: Audit historical lifecycle parity after detached bootstrap
  date: "2026-09-01"
  hypotheses:
    - H86
  subject:
    tree_label: metabrowser-h86-lifecycle-f41
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: f41f3744caa3e59a7331e3e48ac31915c182718967fad0766ba16a17cd811ff9
    tree_provenance: Clean live github.com/jlevy/metabrowser checkout at revision 2d920d60fe3dfc0e17a4fd2cafa08292e60b3de4; ignored build outputs make exact filesystem metadata and shape non-reconstructible.
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
    control: pre-rewrite b75bf85 historical probe
    candidate: controls-aware detached bootstrap candidate
    control_binary:
      name: control
      sha256: a8192e6426bf5d358a803ce66c2ba845bd18f0d3ab57025d37cfd59149205b8d
      size_bytes: 1561440
      args: []
    candidate_binary:
      name: candidate
      sha256: 42d98010f2f4c442ef89ed644fdc634a12b3a19c75a219f60e5c5f302e0d078b
      size_bytes: 2239360
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-historical-lifecycle-breakdown-valid-exploratory.json
  results:
    - job: cold-open-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 609820645.5
          candidate_median: 621653604.5
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.015
          change_pct: 1.111
          ci95_low_pct: -0.525
          ci95_high_pct: 3.402
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 367566875.5
          candidate_median: 371327375.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.034
          change_pct: 1.253
          ci95_low_pct: -1.926
          ci95_high_pct: 3.765
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 2147281500.0
          candidate_median: 2160448000.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.052
          change_pct: 2.749
          ci95_low_pct: -3.513
          ci95_high_pct: 6.277
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 422511500.0
          candidate_median: 406505500.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.014
          change_pct: -3.224
          ci95_low_pct: -5.645
          ci95_high_pct: -1.772
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1721420000.0
          candidate_median: 1752787500.0
          control_p95_over_median: 1.075
          candidate_p95_over_median: 1.063
          change_pct: 4.445
          ci95_low_pct: -4.306
          ci95_high_pct: 9.309
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 74260480.0
          candidate_median: 88686592.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.016
          change_pct: 19.432
          ci95_low_pct: 18.14
          ci95_high_pct: 20.55
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes exceeds its +5% regression limit"
          - "minor_faults exceeds its +10% regression limit"
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 559183771.0
          candidate_median: 574690666.5
          control_p95_over_median: 1.15
          candidate_p95_over_median: 1.099
          change_pct: 0.927
          ci95_low_pct: -5.626
          ci95_high_pct: 3.833
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 321652937.5
          candidate_median: 327202895.5
          control_p95_over_median: 1.154
          candidate_p95_over_median: 1.067
          change_pct: -0.394
          ci95_low_pct: -3.018
          ci95_high_pct: 4.042
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 2164375500.0
          candidate_median: 2180948000.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.02
          change_pct: 2.003
          ci95_low_pct: -0.876
          ci95_high_pct: 3.007
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 388478000.0
          candidate_median: 364333500.0
          control_p95_over_median: 1.071
          candidate_p95_over_median: 1.073
          change_pct: -6.304
          ci95_low_pct: -8.206
          ci95_high_pct: -4.324
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1779121500.0
          candidate_median: 1806321500.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.025
          change_pct: 2.923
          ci95_low_pct: 0.275
          ci95_high_pct: 5.384
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 61988864.0
          candidate_median: 75653120.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.004
          change_pct: 21.752
          ci95_low_pct: 20.673
          ci95_high_pct: 22.402
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
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
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 595272041.5
          candidate_median: 640944104.0
          control_p95_over_median: 1.293
          candidate_p95_over_median: 1.237
          change_pct: 5.353
          ci95_low_pct: 1.26
          ci95_high_pct: 9.574
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 30022125.0
          candidate_median: 34348895.5
          control_p95_over_median: 1.465
          candidate_p95_over_median: 1.42
          change_pct: 16.583
          ci95_low_pct: -20.426
          ci95_high_pct: 32.37
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 2110253500.0
          candidate_median: 2229341000.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.047
          change_pct: 6.419
          ci95_low_pct: -2.765
          ci95_high_pct: 9.084
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 415514500.0
          candidate_median: 418270500.0
          control_p95_over_median: 1.221
          candidate_p95_over_median: 1.113
          change_pct: -2.532
          ci95_low_pct: -5.927
          ci95_high_pct: -0.134
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1687303500.0
          candidate_median: 1819992500.0
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.049
          change_pct: 7.484
          ci95_low_pct: -3.057
          ci95_high_pct: 12.403
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 74489856.0
          candidate_median: 89260032.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.012
          change_pct: 19.988
          ci95_low_pct: 18.537
          ci95_high_pct: 21.224
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes exceeds its +5% regression limit"
          - "minor_faults exceeds its +10% regression limit"
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 645580583.5
          candidate_median: 620507124.5
          control_p95_over_median: 1.823
          candidate_p95_over_median: 1.55
          change_pct: -0.843
          ci95_low_pct: -19.317
          ci95_high_pct: 10.079
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 629246542.0
          candidate_median: 607816666.5
          control_p95_over_median: 1.841
          candidate_p95_over_median: 1.531
          change_pct: -0.579
          ci95_low_pct: -20.321
          ci95_high_pct: 10.886
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1971964500.0
          candidate_median: 1865942000.0
          control_p95_over_median: 1.194
          candidate_p95_over_median: 1.308
          change_pct: 1.757
          ci95_low_pct: -3.851
          ci95_high_pct: 9.551
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 205469500.0
          candidate_median: 182286500.0
          control_p95_over_median: 1.074
          candidate_p95_over_median: 1.086
          change_pct: -9.93
          ci95_low_pct: -16.097
          ci95_high_pct: -4.23
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1754164000.0
          candidate_median: 1679478000.0
          control_p95_over_median: 1.233
          candidate_p95_over_median: 1.332
          change_pct: 3.481
          ci95_low_pct: -2.946
          ci95_high_pct: 11.563
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 78921728.0
          candidate_median: 89546752.0
          control_p95_over_median: 1.152
          candidate_p95_over_median: 1.028
          change_pct: 16.747
          ci95_low_pct: 7.834
          ci95_high_pct: 18.73
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes exceeds its +5% regression limit"
          - "minor_faults straddles its +10% regression limit"
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
          minor_faults: inconclusive
          peak_rss_bytes: rejected
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 974571103.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 1031
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - An uncontrolled host can make a lifecycle residual look like serialization or destruction cost.
    notes: "This is an attribution audit, not another production mechanism."
  verdict:
    decision: in-progress
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 0.927
    reason: "Construction and save/join medians reach practical parity, but quiet-host noninferiority and the 17-22% retained-memory gap remain open."
    commit: null
---
# Audit Historical Lifecycle Parity After Detached Bootstrap

## Hypothesis

H86: once controls-rich cold construction no longer uses the scanner reducer, any
remaining gap to the pre-rewrite binary should be attributable to a later lifecycle
stage: save handoff, serialization, report construction, or retained-index destruction.

## What was tried

One interleaved run compared the preserved pre-rewrite binary with the controls-aware
candidate across `cold-scan-index`, `cold-open-save`, `cold-snapshot-save`, and
`default-tree`. A clean counter- and oracle-disabled sampling profile separately
measured cold construction and the default command.

## What the numbers said

Cold construction changed +0.93% by wall and -0.39% by component; both intervals include
zero. Save/join wall changed +1.11%, also including zero.
The host ended at 100% CPU, so the default-tree interval widened to -19.32% through
+10.08% and cannot decide parity.
The snapshot-save component interval also includes zero; its wall includes an untimed
setup scan exposed to the same load change.

Peak RSS remains 17–22% above the historical binary across the jobs, with roughly 19%
more minor faults. The clean profile is dominated by `open` and `getattrlistbulk`; it
shows about 6% allocator self time and less than 0.5% index self time, not a remaining
3%-class CPU hotspot.

## Verdict

**IN PROGRESS.** Construction and save/join medians have reached practical historical
parity, but quiet-host noninferiority and the retained-memory target remain open.
These results do not justify a dual compact/mutable representation as an emergency
wall-time fix.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
