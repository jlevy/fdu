---
title: Share pool orchestration through a dynamic consumer
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-098
  title: Share pool orchestration through a dynamic consumer
  date: "2026-09-01"
  hypotheses:
    - H86
  subject:
    tree_label: metabrowser-h86-orchestration-cleanup
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 9110fef5c40618446c1d9daf2128b27dc10d2cba5ea04294932f5242d46fcbcc
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
    control: controls-aware detached builder before pool-orchestration cleanup
    candidate: shared runner with dynamically dispatched message consumption
    control_binary:
      name: control
      sha256: 42d98010f2f4c442ef89ed644fdc634a12b3a19c75a219f60e5c5f302e0d078b
      size_bytes: 2239360
      args: []
    candidate_binary:
      name: candidate
      sha256: 4fd695ff941e9213c2d2ceaa6d46df6ae87e1bd6a5b781c631d2b4dbc9474fe9
      size_bytes: 2206320
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-shared-orchestration-cleanup-exploratory.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 552162417.0
          candidate_median: 557674333.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.026
          change_pct: 0.816
          ci95_low_pct: 0.451
          ci95_high_pct: 3.002
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 323089000.0
          candidate_median: 323134520.5
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.016
          change_pct: -0.06
          ci95_low_pct: -0.464
          ci95_high_pct: 0.635
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2199470000.0
          candidate_median: 2208609000.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.015
          change_pct: 0.308
          ci95_low_pct: -0.108
          ci95_high_pct: 0.975
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 334746500.0
          candidate_median: 340589500.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.032
          change_pct: 1.437
          ci95_low_pct: 0.659
          ci95_high_pct: 3.577
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1864178500.0
          candidate_median: 1866644500.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.013
          change_pct: -0.136
          ci95_low_pct: -0.556
          ci95_high_pct: 0.612
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 75481088.0
          candidate_median: 75767808.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.009
          change_pct: 0.206
          ci95_low_pct: 0.033
          ci95_high_pct: 1.034
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
          control_median: 3168188229.5
          candidate_median: 3134218917.0
          control_p95_over_median: 1.066
          candidate_p95_over_median: 1.058
          change_pct: -1.226
          ci95_low_pct: -2.567
          ci95_high_pct: 0.37
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 1868780896.0
          candidate_median: 1850931979.0
          control_p95_over_median: 1.037
          candidate_p95_over_median: 1.029
          change_pct: -2.057
          ci95_low_pct: -2.776
          ci95_high_pct: 0.468
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 3506164000.0
          candidate_median: 3470212000.0
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.062
          change_pct: -1.352
          ci95_low_pct: -1.917
          ci95_high_pct: -0.023
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 2244436500.0
          candidate_median: 2209114000.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.039
          change_pct: -0.94
          ci95_low_pct: -1.318
          ci95_high_pct: -0.167
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1264782000.0
          candidate_median: 1261884500.0
          control_p95_over_median: 1.125
          candidate_p95_over_median: 1.102
          change_pct: -1.429
          ci95_low_pct: -2.735
          ci95_high_pct: 0.233
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 378707968.0
          candidate_median: 379183104.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.007
          change_pct: 0.078
          ci95_low_pct: -0.076
          ci95_high_pct: 0.305
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
      wall_ns_median: 503614208.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 140
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - Dynamic dispatch at every directory-group handoff can turn source deduplication into runtime work.
    notes: "Removed about 60 duplicated orchestration lines, but routed every received work message through a dyn FnMut boundary."
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 0.816
    reason: "Whole-process wall regressed 0.82% [0.45%, 3.00%] while component time was unchanged; the source reduction does not justify a supported regression."
    commit: null
---
# Share pool orchestration through a dynamic consumer

## Hypothesis

H86: the streaming and detached walkers duplicated worker-pool setup, adaptive scaling,
termination, diagnostics, panic handling, joins, and error ordering.
One shared runner with a mode-specific message consumer should remove that maintenance
risk without moving `cold-scan-index` beyond the +3% noninferiority bound.

## What was tried

The two wrappers supplied their existing worker function and a `dyn FnMut(WalkMessage)`
consumer to one pool runner.
Filesystem enumeration remained monomorphized through `WalkEmission`, and public,
opened, refresh, and mutation semantics did not change.
The candidate removed about 60 lines of duplicated orchestration but introduced an
indirect call for each received work message.

## What the numbers said

Across twelve interleaved pairs, whole-process wall time regressed 0.82%, with a paired
95% interval from +0.45% to +3.00%. The measured scan component changed -0.06%, with an
interval from -0.46% to +0.64%, so this run does not establish that the dynamic call
itself caused the wall result.
The opened-discovery placebo was noninferior, and every exact oracle and tree-drift
check passed.

## Verdict

**REJECTED.** The source reduction does not justify retaining a form with a supported
whole-process regression.
The follow-up keeps the shared runner but removes its trait-object consumer.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
