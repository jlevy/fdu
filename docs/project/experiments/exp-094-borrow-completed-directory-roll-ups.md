---
title: Borrow completed directory roll-ups
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-094
  title: Borrow completed directory roll-ups
  date: "2026-09-01"
  hypotheses:
    - H86
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
    candidate: borrowed final directory roll-up merge
    control_binary:
      name: control
      sha256: dce74be1d042bd3409ca25774c89a3387677ba47607d216fd9d4e2dd1afb8e7a
      size_bytes: 2322304
      args:
        - "--no-controls"
    candidate_binary:
      name: candidate
      sha256: 5383b70a8c3b6dc367588f447f10dd18255b7f1b07b78969724bd639141ca6dc
      size_bytes: 2404928
      args:
        - "--no-controls"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-borrow-merge-controls-disabled-exploratory.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 581020833.5
          candidate_median: 579143021.0
          control_p95_over_median: 1.07
          candidate_p95_over_median: 1.02
          change_pct: 0.189
          ci95_low_pct: -2.963
          ci95_high_pct: 1.777
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 330494916.5
          candidate_median: 330016250.0
          control_p95_over_median: 1.128
          candidate_p95_over_median: 1.042
          change_pct: 0.722
          ci95_low_pct: -4.482
          ci95_high_pct: 3.336
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 2277178000.0
          candidate_median: 2218867500.0
          control_p95_over_median: 1.073
          candidate_p95_over_median: 1.034
          change_pct: -3.117
          ci95_low_pct: -7.937
          ci95_high_pct: 0.849
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 470887500.0
          candidate_median: 388235000.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.052
          change_pct: -16.005
          ci95_low_pct: -16.748
          ci95_high_pct: -14.308
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1802834500.0
          candidate_median: 1832529000.0
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.028
          change_pct: 1.091
          ci95_low_pct: -6.29
          ci95_high_pct: 4.93
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 82157568.0
          candidate_median: 84860928.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.006
          change_pct: 3.04
          ci95_low_pct: 2.687
          ci95_high_pct: 3.684
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: noninferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 844
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - Borrowing two arena entries must preserve parent-before-child allocation and reject stale handles.
    notes: One split-at-mut helper replaces cloning complete directory roll-ups.
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 0.189
    reason: "Borrowing removes a clone from the structural path and is noninferior at +0.19% [-2.96%, +1.78%]."
    commit: null
---
# Borrow Completed Directory Roll-Ups

## Hypothesis

H86/H60: the final directories-only pass can merge a completed child roll-up directly
from the arena instead of cloning its extension maps before every parent merge.

## What was tried

Because cold parents are allocated before descendants, the builder splits the arena at
the child slot and borrows the child roll-up while mutating the earlier parent.
Direct file and directory-self contributions remain folded during the walk.

## What the numbers said

Wall time changed +0.19%, with a paired 95% interval from -2.96% to +1.78%. The result
is noninferior and exact, but it does not establish a standalone speedup.

## Verdict

**ACCEPTED.** Borrowing removes an unnecessary clone from the structural design and
keeps the composite checkpoint inside its noninferiority margin.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
