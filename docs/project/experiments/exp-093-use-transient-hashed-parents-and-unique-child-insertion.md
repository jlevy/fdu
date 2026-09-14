---
title: Use transient hashed parents and unique child insertion
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-093
  title: Use transient hashed parents and unique child insertion
  date: "2026-09-01"
  hypotheses:
    - H86
    - S1b
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
    candidate: hashed transient parent table and unique child insertion
    control_binary:
      name: control
      sha256: dce74be1d042bd3409ca25774c89a3387677ba47607d216fd9d4e2dd1afb8e7a
      size_bytes: 2322304
      args:
        - "--no-controls"
    candidate_binary:
      name: candidate
      sha256: 31967b18db6e65b42cf3a83c7f7589c0a3a9ab6700b14056cf91cf59a15367f4
      size_bytes: 2404928
      args:
        - "--no-controls"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-hash-unique-controls-disabled-exploratory.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 573264416.5
          candidate_median: 578685145.5
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.06
          change_pct: 0.831
          ci95_low_pct: -0.841
          ci95_high_pct: 1.552
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 321402999.5
          candidate_median: 330229270.5
          control_p95_over_median: 1.132
          candidate_p95_over_median: 1.076
          change_pct: 2.303
          ci95_low_pct: -1.04
          ci95_high_pct: 3.848
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 2218568000.0
          candidate_median: 2210686500.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.026
          change_pct: -0.22
          ci95_low_pct: -2.43
          ci95_high_pct: 1.352
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 456140000.0
          candidate_median: 391879000.0
          control_p95_over_median: 1.069
          candidate_p95_over_median: 1.09
          change_pct: -12.984
          ci95_low_pct: -16.244
          ci95_high_pct: -11.069
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1761601500.0
          candidate_median: 1821529500.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.012
          change_pct: 3.458
          ci95_low_pct: 0.324
          ci95_high_pct: 4.838
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 82124800.0
          candidate_median: 85106688.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.006
          change_pct: 3.776
          ci95_low_pct: 2.001
          ci95_high_pct: 4.036
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "voluntary_context_switches straddles its +50% regression limit"
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
    lines_changed: 829
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - Duplicate child facts must fail closed instead of silently replacing an EntryId.
    notes: The retained child map remains ordered; only the private point-lookup table is hashed.
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 0.831
    reason: "The transient contract is explicit and noninferior at +0.83% [-0.84%, +1.55%], with exact digests."
    commit: null
---
# Use Transient Hashed Parents and Unique Child Insertion

## Hypothesis

H86/S1b: the detached builder needs point lookup for incomplete directory paths but
never observes their order, and each sibling name is known to be unique.
A transient hash map and one vacant-entry insertion should remove ordered-map and
double-search work without changing the retained index.

## What was tried

Only the builder’s temporary path-to-directory table changed to hashed lookup.
The retained child map stayed ordered for deterministic queries and snapshots.
Child insertion now uses one entry lookup and fails closed on a duplicate.

## What the numbers said

Wall time changed +0.83%, with a paired 95% interval from -0.84% to +1.55%. Exact
digests matched and the candidate remained within the +3% noninferiority margin.

## Verdict

**ACCEPTED.** The transient structure now states its actual contract and avoids work,
with no measurable regression.
This checkpoint is a constituent of H86 rather than a standalone speed claim.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
