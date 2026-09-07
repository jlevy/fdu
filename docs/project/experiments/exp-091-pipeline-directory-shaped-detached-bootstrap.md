---
title: Pipeline directory-shaped detached bootstrap
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-091
  title: Pipeline directory-shaped detached bootstrap
  date: "2026-09-01"
  hypotheses:
    - H86
    - S1b
    - H60
  subject:
    tree_label: metabrowser-current-h86
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 891c45c10c305b792aaef2d962b154cf785621856d2af5fa1240953efdf6bd48
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
    control: c6380f7 controls-disabled scanner reducer
    candidate: pipelined directory-group builder
    control_binary:
      name: control
      sha256: dce74be1d042bd3409ca25774c89a3387677ba47607d216fd9d4e2dd1afb8e7a
      size_bytes: 2322304
      args:
        - "--no-controls"
    candidate_binary:
      name: candidate
      sha256: c1314ef9cbdffcd4874e61185b7f26cc330e00488a5f954830168c9d951d4b58
      size_bytes: 2421440
      args:
        - "--no-controls"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-pipelined-controls-disabled-exploratory.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 561120229.0
          candidate_median: 580239646.5
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.018
          change_pct: 2.477
          ci95_low_pct: 0.699
          ci95_high_pct: 4.226
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 321570833.0
          candidate_median: 329682938.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.016
          change_pct: 2.236
          ci95_low_pct: 1.437
          ci95_high_pct: 4.369
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 2261913500.0
          candidate_median: 2296618500.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.016
          change_pct: 1.365
          ci95_low_pct: 0.232
          ci95_high_pct: 3.479
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 412006500.0
          candidate_median: 403907500.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.024
          change_pct: -1.949
          ci95_low_pct: -3.32
          ci95_high_pct: -0.4
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1849656000.0
          candidate_median: 1886012500.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.018
          change_pct: 1.904
          ci95_low_pct: 0.851
          ci95_high_pct: 3.955
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 81641472.0
          candidate_median: 84910080.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.002
          change_pct: 3.898
          ci95_low_pct: 3.48
          ci95_high_pct: 4.291
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
          - "minor_faults exceeds its +10% regression limit"
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
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 846
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - A detached builder can accidentally serialize construction after the walk and destroy producer-consumer overlap.
    notes: One private builder path; the later shared-walker checkpoint removes duplicate filesystem traversal logic.
  verdict:
    decision: superseded
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 2.477
    reason: "The first pipelined form regressed wall 2.48% [+0.70%, +4.23%]; later ownership and borrow changes retained the pipeline while recovering noninferiority."
    commit: null
---
# Pipeline Directory-Shaped Detached Bootstrap

## Hypothesis

H86/S1b/H60: a detached cold scan can replace one path-bearing observation per entry
with directory-shaped groups while filesystem workers continue, then finish roll-ups in
one directories-only pass.

## What was tried

Workers published one parent path and component-only child facts before making child
directories claimable.
The main thread built the ordinary mutable index concurrently, folded direct
contributions immediately, and propagated completed directory roll-ups in reverse
allocation order after the walk.

An earlier join-then-build spike took roughly 410 ms where the pipelined path had been
about 270 ms. That prototype was removed because it serialized work the existing scan
overlapped. This recorded run measures the first pipelined form.

## What the numbers said

Against the immediate controls-disabled scanner reducer, wall time regressed 2.48%, with
a paired 95% interval from +0.70% to +4.23%. The mechanism removed per-file paths and
most ancestor roll-up merges, but the first implementation still paid avoidable
ownership and consolidation costs.

## Verdict

**SUPERSEDED.** Keep the pipelined shape, but not this implementation checkpoint.
Later experiments remove duplicate ownership and roll-up work before extending the
builder to fixed controls.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
