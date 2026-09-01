---
title: Compact scanner batches and optional fixed partitions
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-085
  title: Compact scanner batches and optional fixed partitions
  date: "2026-09-01"
  hypotheses:
    - H99
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
    control: streaming allocation guards at 3c0e1a2
    candidate: compact scanner batches plus optional fixed-partition storage
    control_binary:
      name: control
      sha256: c313ecd326426dd24fc08ac256ecfb3174624bbe6153ba9beeab8b2681b87018
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: 7370085136c39307e651593a32d6fd7461915e28cc5711833a43c59a8acb87af
      size_bytes: 2156768
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h99-compact-batches-partitions.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 569399395.5
          candidate_median: 563592021.0
          control_p95_over_median: 1.052
          candidate_p95_over_median: 1.047
          change_pct: -1.631
          ci95_low_pct: -2.744
          ci95_high_pct: 0.341
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 326765250.0
          candidate_median: 335367959.0
          control_p95_over_median: 1.071
          candidate_p95_over_median: 1.065
          change_pct: 0.825
          ci95_low_pct: -0.578
          ci95_high_pct: 4.355
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 2296159000.0
          candidate_median: 2333150000.0
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.063
          change_pct: -0.352
          ci95_low_pct: -1.415
          ci95_high_pct: 2.415
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 416367000.0
          candidate_median: 383718500.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.096
          change_pct: -6.874
          ci95_low_pct: -9.098
          ci95_high_pct: -4.483
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1879168000.0
          candidate_median: 1931223000.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.066
          change_pct: 1.476
          ci95_low_pct: -0.013
          ci95_high_pct: 4.418
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 77389824.0
          candidate_median: 59973632.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.009
          change_pct: -22.468
          ci95_low_pct: -23.134
          ci95_high_pct: -22.134
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 356155354.0
          candidate_median: 346465896.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.042
          change_pct: -2.561
          ci95_low_pct: -3.328
          ci95_high_pct: -0.132
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 351288833.0
          candidate_median: 341718708.5
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.042
          change_pct: -2.562
          ci95_low_pct: -3.422
          ci95_high_pct: -0.034
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2053308000.0
          candidate_median: 2016709000.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.035
          change_pct: -1.408
          ci95_low_pct: -2.353
          ci95_high_pct: 0.664
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 206722500.0
          candidate_median: 182320000.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.016
          change_pct: -12.274
          ci95_low_pct: -12.806
          ci95_high_pct: -11.046
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1845129000.0
          candidate_median: 1836672500.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.03
          change_pct: -0.246
          ci95_low_pct: -1.142
          ci95_high_pct: 2.076
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 89874432.0
          candidate_median: 72794112.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.002
          change_pct: -19.106
          ci95_low_pct: -19.31
          ci95_high_pct: -18.921
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
      wall_ns_median: 494276646.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 490
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - The complete composite fails the preregistered keep threshold despite lower allocation and retained-memory cost.
    notes: Adds a second prepared-batch representation and optional boxed partition state with first-control materialization.
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -2.561
    reason: "The default path improved 2.56% with CI [-3.33%, -0.13%], below the preregistered 3% complexity bar; cold indexing improved 1.63% with CI [-2.74%, +0.34%], so the composite gate failed."
    commit: 3c0e1a2
---
# Compact scanner batches and optional fixed partitions

## Hypothesis

H99: private scanner batches retained `Vec<ObservationOp>` even though every scanner
operation had an unconditional expectation.
The completed-index allocation trace attributed 25.6 MB of high-water storage to those
buffers. Keeping scanner-owned input as compact `Vec<Op>` until the public streaming
boundary, combined with exp-084’s compact control-free partition state, should clear the
3% `default-tree` keep gate while preserving public and opened-root behavior.

## What was tried

`ScannerBatch` transported `Vec<Op>` and widened it to `ObservationOp` only when the
public `scan` API returned an observation.
The index accepted a private prepared-scanner variant with resolved parents, while
arbitrary public observations kept their conditional representation and complete
preflight.

The composite also reapplied exp-084: control-free indexes retained one fixed reducer,
projected `unignored = all` at query boundaries, and materialized boxed directory-only
`unignored` reducers before the first control reclassification.
New representation tests covered control-free projection, directory-only storage, and
first-control materialization.
The complete all-feature and no-default-feature core matrices passed before timing.

## What the numbers said

The compact scanner transport did not move the primary job by itself.
A six-pair diagnostic measured `default-tree` at +0.11%, with a 95% interval from -0.88%
to +1.46%, and `cold-scan-index` at -1.24%, with an interval from -2.22% to +0.69%. The
profiled buffer was real retained memory, but not a leading elapsed-time cost.

The complete composite improved `default-tree` wall time 2.56%, with a paired 95%
interval from -3.33% to -0.13%, and component time 2.56%. Peak RSS fell 19.11% and user
CPU time fell 12.27%. `cold-scan-index` improved 1.63% by median, but its interval from
-2.74% to +0.34% crossed zero.
Both runs preserved the frozen tree fingerprint and recorded no invalid samples.

The result closely repeats exp-084’s 2.63% default-path improvement.
Adding compact scanner transport did not supply the missing margin, so the optional
partition representation remains a real memory improvement whose wall benefit is too
small for its state and materialization paths.

## Verdict

**REJECTED.** The default path improved 2.56% with a 95% interval from -3.33% to -0.13%,
below the preregistered 3% complexity bar.
Cold indexing improved 1.63% with an interval from -2.74% to +0.34%, so the composite
gate failed. The implementation was removed.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
