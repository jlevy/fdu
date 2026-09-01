---
title: Apply fixed controls once per detached directory
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-096
  title: Apply fixed controls once per detached directory
  date: "2026-09-01"
  hypotheses:
    - H86
  subject:
    tree_label: metabrowser-current-h86-controls
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: f73a8a3d0e0b6364b05791abd21ca764b9f621de2ecbb0b21e27bcfba5119087
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
    control: c6380f7 exact scanner reducer with controls enabled
    candidate: controls-aware directory-group builder
    control_binary:
      name: control
      sha256: dce74be1d042bd3409ca25774c89a3387677ba47607d216fd9d4e2dd1afb8e7a
      size_bytes: 2322304
      args: []
    candidate_binary:
      name: candidate
      sha256: 2ce3a4215fedd55d9d3f296af4615b238ab81c5f91fe5ef7e19e440a4f2cddc5
      size_bytes: 2404896
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-directory-controls-exploratory.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 868020917.0
          candidate_median: 574376250.0
          control_p95_over_median: 1.084
          candidate_p95_over_median: 1.073
          change_pct: -33.553
          ci95_low_pct: -36.408
          ci95_high_pct: -33.141
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 622519708.0
          candidate_median: 325703729.0
          control_p95_over_median: 1.11
          candidate_p95_over_median: 1.108
          change_pct: -47.427
          ci95_low_pct: -49.769
          ci95_high_pct: -46.32
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 2654195500.0
          candidate_median: 2159804000.0
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.02
          change_pct: -19.747
          ci95_low_pct: -21.82
          ci95_high_pct: -17.322
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 897733500.0
          candidate_median: 375000000.0
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.044
          change_pct: -58.592
          ci95_low_pct: -59.579
          ci95_high_pct: -57.946
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1748886500.0
          candidate_median: 1781134500.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.025
          change_pct: 0.686
          ci95_low_pct: -2.766
          ci95_high_pct: 3.115
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 101605376.0
          candidate_median: 75218944.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.01
          change_pct: -25.881
          ci95_low_pct: -27.905
          ci95_high_pct: -24.966
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
  reference_tools: []
  complexity:
    lines_changed: 1025
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - A control applied after siblings or descendant publication would classify retained entries against incomplete state.
    notes: The private one-shot builder shares filesystem traversal and pool orchestration with streaming; public and opened consumers retain the causal reducer.
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -33.553
    reason: "Wall falls 33.55% [-36.41%, -33.14%], component falls 47.43%, peak RSS falls 25.88%, and exact control and mutation oracles pass."
    commit: null
---
# Apply Fixed Controls Once Per Detached Directory

## Hypothesis

H86: a complete directory group can carry its verified fixed-control operation.
If the consumer installs that state before siblings and descendants become visible, it
can classify every entry once instead of repeatedly projecting scanner batches and
reclassifying subtrees.

## What was tried

Workers retain an optional `ControlUpsert` or `ControlRemove` beside each complete
directory listing. The main-thread builder applies it before inserting siblings, then
classifies children against the complete ancestor control table.
Public scan, opened discovery, reconciliation, and arbitrary mutation keep the causal
scanner reducer.

Differential tests cover worker counts one through four, root and nested rules,
negation, an ignored parent, a non-file `.gitignore`, source and pattern limits, the
capability-disabled build, and the first exact public mutation after bootstrap.
The work also fixed the scanner oracle’s rejection of the documented `ControlRemove` for
a non-file control path.

## What the numbers said

Controls-rich `cold-scan-index` wall time fell 33.55%, with a paired 95% interval from
-36.41% to -33.14%; component time fell 47.43%. Allocations fell from 6,024,294 to
987,134, allocated bytes from 491,242,604 to 133,101,059, and peak RSS 25.88%. Scanner
projection, preparation, and reduction counters were zero on the private path, and the
exact tree digest matched.

## Verdict

**ACCEPTED.** The result exceeds its 25% prediction, preserves the streaming APIs, and
removes the measured repeated-projection mechanism.
Compact retained storage and the quiet historical/RSS verdict remain separate H86 work.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
