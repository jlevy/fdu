---
title: Fuse detached control-free scanner preparation and reduction
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-087
  title: Fuse detached control-free scanner preparation and reduction
  date: "2026-09-01"
  hypotheses:
    - H104
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
    control: phase-instrumented streaming control at 80e5897
    candidate: fused detached reducer plus compact optional partitions
    control_binary:
      name: control
      sha256: 8ed1536071efa35821d449c97279092e9f830c722d6e9480283a856580e4e8e1
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: 95db7967e160c11fdefa979ed10851a9308d9c52efa7371d3aba356d10c9fbb4
      size_bytes: 2173280
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h104-composite.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 560513562.5
          candidate_median: 559673812.5
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.058
          change_pct: -0.197
          ci95_low_pct: -1.191
          ci95_high_pct: 2.12
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 321180396.0
          candidate_median: 324498771.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.1
          change_pct: 0.675
          ci95_low_pct: -0.111
          ci95_high_pct: 2.807
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2257359000.0
          candidate_median: 2250341000.0
          control_p95_over_median: 1.08
          candidate_p95_over_median: 1.1
          change_pct: -0.408
          ci95_low_pct: -1.588
          ci95_high_pct: 0.624
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 409367000.0
          candidate_median: 384564000.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.066
          change_pct: -6.089
          ci95_low_pct: -6.335
          ci95_high_pct: -4.086
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1850686500.0
          candidate_median: 1864123000.0
          control_p95_over_median: 1.083
          candidate_p95_over_median: 1.108
          change_pct: 1.077
          ci95_low_pct: -0.581
          ci95_high_pct: 1.359
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 77103104.0
          candidate_median: 60342272.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.007
          change_pct: -21.898
          ci95_low_pct: -22.053
          ci95_high_pct: -21.373
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 355338958.5
          candidate_median: 349293916.5
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.043
          change_pct: -1.115
          ci95_low_pct: -2.468
          ci95_high_pct: 0.4
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 350343687.5
          candidate_median: 344196416.5
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.043
          change_pct: -1.057
          ci95_low_pct: -2.477
          ci95_high_pct: 0.309
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2063542000.0
          candidate_median: 2013086500.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.031
          change_pct: -1.36
          ci95_low_pct: -2.907
          ci95_high_pct: -1.021
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 202119000.0
          candidate_median: 174343500.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.063
          change_pct: -13.604
          ci95_low_pct: -14.175
          ci95_high_pct: -13.093
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1856869000.0
          candidate_median: 1837174500.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.023
          change_pct: -0.725
          ci95_low_pct: -1.733
          ci95_high_pct: 0.299
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 89645056.0
          candidate_median: 72851456.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.008
          change_pct: -18.77
          ci95_low_pct: -19.039
          ci95_high_pct: -18.283
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
      wall_ns_median: 505652562.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 460
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - The fused lane is intentionally non-atomic and safe only while an exclusively owned index can be discarded.
      - Optional partition storage adds a materialization and rebuild transition for the first late control.
    notes: "The composite removed preparation and restored allocation ratios, but its wall-time result did not justify 460 changed lines and two internal representations."
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -1.115
    reason: "Default-tree wall improved 1.11% with a 95% interval of -2.47% to +0.40%, missing the 3% structural gate; cold-scan-index was flat."
    commit: null
---
# Fuse detached control-free scanner preparation and reduction

## Hypothesis

H104: phase counters attribute about 28.5 ms per scan to preparing trusted scanner
batches. A detached, control-free index can resolve each parent and mutate immediately
because the caller discards the owned index on any internal failure.
Combining that fused lane with exp-084’s compact optional partition representation
should improve `default-tree` by at least 3%, keep cold wall moving in the same
direction, and restore whole-process allocation, reallocation, and requested bytes to
within 1.05 times the pre-rewrite control.

## What was tried

The one-shot builder selected the fused lane once from the index scope.
It retained the prepared reducer for control-aware scans, public observations, opened
discovery, refresh, and watch.
The fused lane consumed the walker’s existing operation vector in one pass, cached the
last resolved parent, and applied directly without allocating a parallel parent vector
or projecting an empty control table.

The composite also stored the all-entry reducer inline and the unignored reducer as an
optional box allocated only on directories in control-capable indexes.
Control-free queries projected `unignored = all`. A late first control materialized
every directory reducer and rebuilt the partition before exposing the new state.

The no-feature and all-feature library suites passed with 467 and 555 tests,
respectively; one manual evidence test remained intentionally ignored.
Focused tests covered fused/prepared equivalence across batches, control-scope
selection, control-free projection, directory-only allocation, and first-control
materialization.

## What the numbers said

A six-pair fused-only diagnostic was neutral: `default-tree` changed +1.31%, with a 95%
interval from -1.70% to +7.46%, and `cold-scan-index` changed +0.19%, with an interval
from -0.16% to +1.40%. Eliminating preparation alone did not shorten the critical path.

The preregistered 12-pair composite improved `default-tree` by 1.11%, with a 95%
interval from -2.47% to +0.40%. `cold-scan-index` changed -0.20%, with an interval from
-1.19% to +2.12%. Neither result established a wall-time improvement, and the primary
result missed the 3% structural gate.

The mechanism worked mechanically.
Enabled repeat-10 counters reported zero scanner preparation and control-projection
time; fused reduction took 833,702 microseconds, versus 317,343 microseconds preparing
and 900,736 microseconds reducing the unchanged control.
Relative to the pre-rewrite control, whole-process allocation, reallocation, and
requested-byte ratios were 1.016, 1.020, and 1.005. Peak RSS improved 18.77% on
`default-tree` and 21.90% on `cold-scan-index`, while user CPU improved 13.60% and
6.09%. Those gains did not move wall time because producer I/O overlaps the removed
consumer work.

## Verdict

**REJECTED.** The composite met its representation and phase-attribution goals but did
not meet the primary wall-time gate.
Remove both parts: retaining a second reducer representation and a non-atomic internal
application lane for an unproven critical-path gain would increase the design burden
without restoring parity.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
