---
title: Compact optional fixed-partition storage
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-084
  title: Compact optional fixed-partition storage
  date: "2026-09-01"
  hypotheses:
    - H98
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
    candidate: compact optional partition working-tree composite
    control_binary:
      name: control
      sha256: c313ecd326426dd24fc08ac256ecfb3174624bbe6153ba9beeab8b2681b87018
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: 463fea39c109e32f04a79f492db024667d3911ebcf4a81f0fb6162f38520dd38
      size_bytes: 2156768
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h98-compact-partition-screen.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 581449500.0
          candidate_median: 582270562.5
          control_p95_over_median: 1.161
          candidate_p95_over_median: 1.224
          change_pct: -0.999
          ci95_low_pct: -2.953
          ci95_high_pct: 11.367
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 327495479.5
          candidate_median: 337329854.0
          control_p95_over_median: 1.168
          candidate_p95_over_median: 1.211
          change_pct: 1.116
          ci95_low_pct: -0.84
          ci95_high_pct: 4.721
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 2295225000.0
          candidate_median: 2236707000.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.121
          change_pct: -1.849
          ci95_low_pct: -6.011
          ci95_high_pct: 0.192
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 426583500.0
          candidate_median: 404511500.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.067
          change_pct: -6.609
          ci95_low_pct: -8.181
          ci95_high_pct: -3.805
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1872998500.0
          candidate_median: 1852506500.0
          control_p95_over_median: 1.092
          candidate_p95_over_median: 1.12
          change_pct: -1.087
          ci95_low_pct: -4.869
          ci95_high_pct: 1.878
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 77512704.0
          candidate_median: 61136896.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.039
          change_pct: -21.199
          ci95_low_pct: -21.51
          ci95_high_pct: -20.419
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
          control_median: 358514541.5
          candidate_median: 350030896.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.085
          change_pct: -2.628
          ci95_low_pct: -3.172
          ci95_high_pct: -1.188
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 353515166.5
          candidate_median: 345188666.5
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.085
          change_pct: -2.483
          ci95_low_pct: -3.169
          ci95_high_pct: -1.241
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2065851000.0
          candidate_median: 2033793000.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.091
          change_pct: -1.811
          ci95_low_pct: -2.186
          ci95_high_pct: -0.204
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 206888500.0
          candidate_median: 183942500.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.058
          change_pct: -10.847
          ci95_low_pct: -11.435
          ci95_high_pct: -8.765
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1859234500.0
          candidate_median: 1850360000.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.094
          change_pct: -0.881
          ci95_low_pct: -1.266
          ci95_high_pct: 0.813
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 89767936.0
          candidate_median: 73482240.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.005
          change_pct: -18.159
          ci95_low_pct: -18.58
          ci95_high_pct: -17.43
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
  reference_tools: []
  complexity:
    lines_changed: 260
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - "The representation saved one duplicate reducer per entry, but the resulting 2.628% default-tree speedup did not justify 260 changed lines and cold-scan-index remained inconclusive."
    notes: "Adds optional boxed directory state, dynamic materialization, and partition-aware merge paths; no dependencies or unsafe code."
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -2.628
    reason: "Default-tree wall improved 2.628% with a 95% interval of -3.172% to -1.188%, which is real but below the pre-registered 3% structural-change gate; cold wall did not establish improvement."
    commit: null
---
# Compact optional fixed-partition storage

## Hypothesis

H98: the control-free form retained two `InternedRollUp` values inline on every entry,
even though the values are identical without ignore controls and files do not retain
child roll-ups at all.
Combining the exp-083 single-reducer mutation lane with optional, directory-only storage
for the second reducer should remove the remaining representation overhead and improve
`default-tree` by at least 3% without regressing the control-rich streaming path.

## What was tried

The spike stored `all` inline and represented `unignored` as an optional boxed reducer.
A control-free index projected `unignored = all`; a control-enabled index allocated the
second reducer only for directories.
Installing the first public control materialized directory reducers and rebuilt them
before exposing the new partition state.

Focused all-feature and no-default-feature tests covered control-free projection,
directory-only allocation, first-control materialization, insert, update, removal,
newest-time repair, and exact digest preservation.
The implementation changed 260 lines and introduced an additional optional state and
materialization path, so the pre-registered structural gate required a 3% wall-time
improvement.

## What the numbers said

The representation diagnosis was correct, but its wall-time value was smaller than the
complexity cost. A direct 113,794-entry scan used the same 1,007,663 component
allocations and 195,856 reallocations as exp-083 while removing another 6,372,464
requested bytes—exactly 56 bytes per entry.
Against the pre-rewrite control, whole-process allocation, reallocation, and requested
byte ratios were all within 1.02; requested bytes were 6.2 MB lower.

`default-tree` improved 2.63%, with a paired 95% interval from -3.17% to -1.19%, and
component time improved 2.48%. Peak RSS improved 18.16%. Those are real gains, but the
primary result is below the 3% threshold.
`cold-scan-index` remained inconclusive: its median wall change was -1.00%, with an
interval from -2.95% to +11.37% during a noisy run.
The run had zero invalid samples and preserved the exact tree digest.

Allocation-stack traces show that the remaining retained allocations are the same entry
arena, path, extension interning, child-map, and roll-up map structures present in the
pre-rewrite engine. They do not identify another duplicate clone large enough to justify
extending this representation.

## Verdict

**REJECTED.** `default-tree` wall improved 2.63% with a 95% interval from -3.17% to
-1.19%, which is real but below the pre-registered 3% structural-change gate;
`cold-scan-index` did not establish improvement.
The implementation was removed.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
