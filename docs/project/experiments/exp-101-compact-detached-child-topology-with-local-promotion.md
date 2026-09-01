---
title: Compact detached child topology with local promotion
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-101
  title: Compact detached child topology with local promotion
  date: "2026-09-01"
  hypotheses:
    - H86
  subject:
    tree_label: metabrowser-113794
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
    control: exact 88304cb detached builder
    candidate: "inline entries, directory payloads, sorted detached children, and per-parent promotion"
    control_binary:
      name: control
      sha256: 9f3d5963164e0580d008e6b0861d8b20c7fe2427df42a4e4945b389c398f8549
      size_bytes: 2222832
      args: []
    candidate_binary:
      name: candidate
      sha256: 33611f4c9207bce2c7d4970d0b15a4ea7bc4d99748593c15ffaa411cafc0bf29
      size_bytes: 2255888
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-sorted-child-promotion.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 686211125.5
          candidate_median: 619575666.5
          control_p95_over_median: 1.553
          candidate_p95_over_median: 1.211
          change_pct: -5.866
          ci95_low_pct: -15.858
          ci95_high_pct: -3.157
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 420932812.5
          candidate_median: 361876771.0
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.184
          change_pct: -5.306
          ci95_low_pct: -18.452
          ci95_high_pct: 0.96
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2101695000.0
          candidate_median: 2027982500.0
          control_p95_over_median: 1.246
          candidate_p95_over_median: 1.098
          change_pct: -2.515
          ci95_low_pct: -4.197
          ci95_high_pct: 3.295
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 383113000.0
          candidate_median: 364630000.0
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.099
          change_pct: -5.32
          ci95_low_pct: -7.836
          ci95_high_pct: 4.579
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1714217500.0
          candidate_median: 1662504500.0
          control_p95_over_median: 1.296
          candidate_p95_over_median: 1.125
          change_pct: -1.91
          ci95_low_pct: -5.952
          ci95_high_pct: 5.65
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 76251136.0
          candidate_median: 41992192.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.014
          change_pct: -45.034
          ci95_low_pct: -45.88
          ci95_high_pct: -44.73
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: superior
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
          control_median: 392015458.5
          candidate_median: 361375333.5
          control_p95_over_median: 1.247
          candidate_p95_over_median: 1.063
          change_pct: -7.701
          ci95_low_pct: -10.159
          ci95_high_pct: -3.774
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 386350604.5
          candidate_median: 356123375.0
          control_p95_over_median: 1.236
          candidate_p95_over_median: 1.064
          change_pct: -7.711
          ci95_low_pct: -10.19
          ci95_high_pct: -3.787
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 1967754000.0
          candidate_median: 1981348000.0
          control_p95_over_median: 1.109
          candidate_p95_over_median: 1.066
          change_pct: 3.302
          ci95_low_pct: -2.006
          ci95_high_pct: 8.046
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 184665000.0
          candidate_median: 169675000.0
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.029
          change_pct: -9.356
          ci95_low_pct: -11.311
          ci95_high_pct: -6.205
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1777210000.0
          candidate_median: 1812667500.0
          control_p95_over_median: 1.124
          candidate_p95_over_median: 1.068
          change_pct: 4.411
          ci95_low_pct: -0.677
          ci95_high_pct: 9.783
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 88236032.0
          candidate_median: 54558720.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.022
          change_pct: -37.787
          ci95_low_pct: -38.384
          ci95_high_pct: -37.409
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
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3827630729.5
          candidate_median: 3772089958.0
          control_p95_over_median: 1.743
          candidate_p95_over_median: 1.138
          change_pct: -1.398
          ci95_low_pct: -3.51
          ci95_high_pct: 1.949
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 2248952020.5
          candidate_median: 2188055563.0
          control_p95_over_median: 2.107
          candidate_p95_over_median: 1.174
          change_pct: -1.859
          ci95_low_pct: -6.702
          ci95_high_pct: 2.116
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 4311169000.0
          candidate_median: 4183045000.0
          control_p95_over_median: 1.236
          candidate_p95_over_median: 1.066
          change_pct: -1.248
          ci95_low_pct: -5.256
          ci95_high_pct: 1.555
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 2508986000.0
          candidate_median: 2454774000.0
          control_p95_over_median: 1.215
          candidate_p95_over_median: 1.031
          change_pct: -2.05
          ci95_low_pct: -4.247
          ci95_high_pct: 0.796
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1787154000.0
          candidate_median: 1736070000.0
          control_p95_over_median: 1.277
          candidate_p95_over_median: 1.111
          change_pct: -1.263
          ci95_low_pct: -6.661
          ci95_high_pct: 3.018
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 380747776.0
          candidate_median: 322232320.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.005
          change_pct: -15.372
          ci95_low_pct: -15.481
          ci95_high_pct: -15.194
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
          - major_faults does not establish non-regression
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
          major_faults: inconclusive
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 431777770.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 634
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - The first arbitrary mutation of a compact parent clones that parent’s child names into a keyed map
      - Compact lookup and iteration rely on detached child names being unique and sorted in native OsStr order
    notes: "No new dependency or unsafe block; one retained fact model, two child-storage states, and local one-time promotion only on mutation."
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -7.701
    reason: "Exploratory default-tree wall improved 7.70% [-10.16%, -3.77%] and RSS fell 37.79%; cold scan moved the same way and opened discovery remained within +3%, pending quiet-host confirmation."
    commit: null
---
# Compact detached child topology with local promotion

## Hypothesis

H86: after moving directory-only state out of the common entry, the remaining detached
index still stores every child name twice and allocates ordered-map nodes for immutable
topology. Keeping one name on the entry, sorting compact child identifiers once, and
promoting only a parent that is later mutated should clear the `default-tree` 3% wall
target and 20% RSS target without slowing opened discovery beyond its +3% bound.

## What was tried

The detached builder now reserves a child vector for each completed listing, moves every
name into its entry, sorts the identifiers by native name order, and rejects adjacent
duplicates. Lookup binary-searches that vector, and ordered iteration borrows names from
the child entries through an allocation-free exact-size iterator.
The first arbitrary insertion or removal under a compact parent clones that parent’s
names once into the existing `BTreeMap`; untouched parents remain compact.

Opened discovery and ordinary public indexes begin in the mutable representation.
Both modes use the same `Index`, entries, roll-ups, query APIs, and mutation reducer.
The composite also includes exp-100’s inline arena entries and out-of-line directory
payload.

## What the numbers said

Across twelve paired uncontrolled trials against the exact `88304cb` control,
`default-tree` wall fell 7.70%, with a paired 95% interval from -10.16% to -3.77%; its
measured component fell 7.71%. `cold-scan-index` wall fell 5.87%, with an interval from
-15.86% to -3.16%. Peak RSS fell 37.79% and 45.03%, respectively.
Opened discovery changed -1.40%, with an interval from -3.51% to +1.95%, and its peak
RSS fell 15.37%.

The exact one-shot counter run removed 206,188 scoped allocations and 23,353,892
allocated bytes.
An exact opened counter run recorded a 0.979 allocation ratio, identical
engine and commit digests, and zero detached builds in the opened route.
The deterministic 2,080-entry fixture measures 10,671 added detached allocations, or
5.13 per entry, and 50,413 opened allocations, or 24.24 per entry.
Its platform ceilings now remove two detached representation allocations and one opened
arena allocation from the prior slopes; the injected one-allocation-per-entry check
proves each runner’s ceiling remains tight.
The ordering, first-mutation, focused index, detached-scanner, and opened-engine tests
all pass.

The host exceeded the quiet-load limit during measurement, so these results select the
implementation for validation but do not close H86. A clean post-change sampling profile
reduced allocator attribution from 5.63% to 3.82% on `cold-scan-index` and from 5.90% to
3.94% on `default-tree`; filesystem work remains about 71%, and direct index frames
remain below 0.4%. The final source also changes a freshly allocated directory payload
to compact storage in place; the measured binary briefly replaced that box, so the
recorded result is conservative but does not substitute for measuring the final binary
in the quiet stage.

## Verdict

**ACCEPTED for the exploratory stage.** The composite clears the local wall, memory,
opened-noninferiority, and allocation screens while preserving one engine and public
mutation contract. Quiet-host macOS confirmation and the separate Linux H86 stage remain
mandatory. Because the measured representation clears the targets, an additional
conditional roll-up representation is not justified by current evidence.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
