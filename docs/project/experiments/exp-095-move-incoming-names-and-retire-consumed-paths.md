---
title: Move incoming names and retire consumed paths
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-095
  title: Move incoming names and retire consumed paths
  date: "2026-09-01"
  hypotheses:
    - H86
    - S1b
    - S2
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
    candidate: moved names and bounded transient directory paths
    control_binary:
      name: control
      sha256: dce74be1d042bd3409ca25774c89a3387677ba47607d216fd9d4e2dd1afb8e7a
      size_bytes: 2322304
      args:
        - "--no-controls"
    candidate_binary:
      name: candidate
      sha256: 738f49ada37fe64c6413fd34d9f7f0b581ae7357a17d1496c4d549fe3ba87839
      size_bytes: 2404912
      args:
        - "--no-controls"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-own-names-drain-paths-controls-disabled-exploratory.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 573986208.5
          candidate_median: 572079375.0
          control_p95_over_median: 1.175
          candidate_p95_over_median: 1.01
          change_pct: -0.307
          ci95_low_pct: -7.17
          ci95_high_pct: 1.215
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 323120583.5
          candidate_median: 324091999.5
          control_p95_over_median: 1.297
          candidate_p95_over_median: 1.014
          change_pct: 0.493
          ci95_low_pct: -7.41
          ci95_high_pct: 2.104
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2197733500.0
          candidate_median: 2200644500.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.015
          change_pct: 0.105
          ci95_low_pct: -1.356
          ci95_high_pct: 3.157
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 448835000.0
          candidate_median: 361624000.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.08
          change_pct: -18.594
          ci95_low_pct: -22.137
          ci95_high_pct: -16.254
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1762063500.0
          candidate_median: 1839341000.0
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.006
          change_pct: 4.185
          ci95_low_pct: 2.338
          ci95_high_pct: 10.83
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 82386944.0
          candidate_median: 80109568.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.008
          change_pct: -2.646
          ci95_low_pct: -3.074
          ci95_high_pct: -2.159
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
  reference_tools: []
  complexity:
    lines_changed: 857
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - Retiring a directory lookup before its listing arrives must fail as unknown ancestry.
    notes: Ownership follows the listing lifecycle; the ordinary retained representation still requires one name clone.
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -0.307
    reason: "Timing is noninferior at -0.31% [-7.17%, +1.22%], while scoped allocations fall by about one per entry and allocated bytes fall 24%."
    commit: null
---
# Move Incoming Names and Retire Consumed Paths

## Hypothesis

H86/S1b/S2: each directory group already owns its child names, and a directory path is
needed only until that listing is consumed.
Moving names into entries and removing consumed lookup keys should eliminate one
allocation per entry and bound transient path retention.

## What was tried

The builder moves each incoming name into the retained entry and clones it only once for
the parent’s ordered child key.
It removes a directory from the transient path table as soon as the listing arrives,
while retaining only newly discovered descendants.

## What the numbers said

Wall time changed -0.31%, with a paired 95% interval from -7.17% to +1.22%. Scoped
allocations fell to 923,671 from 1,107,018, reallocations to 101,952 from 212,083, and
allocated bytes to 164,601,289 from 217,146,323. Roll-up merges fell to 129,013 from
1,217,448.

## Verdict

**ACCEPTED.** Timing is noninferior, while the ownership change removes one allocation
per entry and makes the transient lifetime explicit.
The retained duplicate name is a property of the ordinary mutable representation and
remains for the later compact layout decision.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
