---
title: Skip unignored roll-up maintenance in control-free scopes
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-083
  title: Skip unignored roll-up maintenance in control-free scopes
  date: "2026-09-01"
  hypotheses:
    - H97
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
    candidate: optional unignored reducer working-tree spike
    control_binary:
      name: before
      sha256: c313ecd326426dd24fc08ac256ecfb3174624bbe6153ba9beeab8b2681b87018
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: 2f3d2c09c4cbdd2e71ab288536bcafc491c0449050fbb198a3684d5baf2acc97
      size_bytes: 2156752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-exp083-optional-unignored.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 557872083.5
          candidate_median: 557824312.5
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.016
          change_pct: -0.468
          ci95_low_pct: -1.969
          ci95_high_pct: 1.256
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 320684688.0
          candidate_median: 320321249.5
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.01
          change_pct: -0.125
          ci95_low_pct: -1.41
          ci95_high_pct: 0.76
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2248939500.0
          candidate_median: 2240347000.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.011
          change_pct: -0.714
          ci95_low_pct: -2.041
          ci95_high_pct: 0.073
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 411810000.0
          candidate_median: 392982000.0
          control_p95_over_median: 1.028
          candidate_p95_over_median: 1.015
          change_pct: -4.513
          ci95_low_pct: -5.503
          ci95_high_pct: -3.204
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1841054500.0
          candidate_median: 1847498500.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.012
          change_pct: -0.103
          ci95_low_pct: -0.972
          ci95_high_pct: 0.904
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 77316096.0
          candidate_median: 72007680.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.005
          change_pct: -6.731
          ci95_low_pct: -7.202
          ci95_high_pct: -6.369
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
          control_median: 366705604.0
          candidate_median: 363932521.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.025
          change_pct: -1.606
          ci95_low_pct: -2.818
          ci95_high_pct: -0.136
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 361763958.0
          candidate_median: 359119708.5
          control_p95_over_median: 1.03
          candidate_p95_over_median: 1.026
          change_pct: -1.609
          ci95_low_pct: -2.835
          ci95_high_pct: -0.025
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2114035500.0
          candidate_median: 2085166500.0
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.022
          change_pct: -1.992
          ci95_low_pct: -3.181
          ci95_high_pct: 0.816
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 212963500.0
          candidate_median: 189045500.0
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.277
          change_pct: -10.658
          ci95_low_pct: -11.866
          ci95_high_pct: -8.77
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1893727000.0
          candidate_median: 1871890500.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.036
          change_pct: -0.677
          ci95_low_pct: -2.321
          ci95_high_pct: 1.963
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 89767936.0
          candidate_median: 84402176.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.01
          change_pct: -6.119
          ci95_low_pct: -6.519
          ci95_high_pct: -5.003
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
  reference_tools: []
  complexity:
    lines_changed: 127
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Adds lifecycle state, query-boundary projection, and first-control materialization to make the redundant reducer optional; exact semantics are preserved, but the new state is not justified by the observed wall result."
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -1.606
    reason: "Scoped allocation fell by about one event and 323 requested bytes per entry, but default-tree improved only 1.61% and cold-scan-index 0.47%; neither met the 3% rule, and the latter interval crossed zero."
    commit: null
---
# Skip unignored roll-up maintenance in control-free scopes

## Hypothesis

H97: The fixed `all` and `unignored` reducers carry identical facts when the scan scope
disables ignore controls, but every file contribution still clones its extension map and
every ancestor merges both copies.
Maintaining only `all` in that scope, then projecting `unignored = all` at a query
boundary, should remove roughly one allocation per entry and materially reduce
`default-tree` and `cold-scan-index` wall time.

## What was tried

The spike added one index-level state bit derived from the semantic scope.
Control-free mutations maintained only `all`; fixed-partition queries returned an owned
copy of `all` as `unignored`. Control-enabled indexes retained the existing dual
reducers unchanged, and the first public control installed into a control-free index
materialized the second reducer before publishing the commit.

Focused tests covered control-free insert, update, removal, newest-time repair, summary
projection, and first-control materialization in all-feature and no-default-feature
builds. All engine digests matched, and both timed jobs recorded zero invalid samples,
baseline drift, or tree mutation.

## What the numbers said

The allocation diagnosis was exact.
On a direct 113,794-entry scan, component allocations fell by 114,782 and requested
bytes fell by 36,756,544, while reallocations and the engine digest stayed unchanged.
The repeat-40 profile put the resulting candidate within 1.02 times the pre-rewrite
control for allocation events, reallocations, and requested bytes.
Peak RSS also fell 6.12% on `default-tree` and 6.73% on `cold-scan-index`.

Elapsed time did not follow the resource movement closely enough to keep the code.
`default-tree` improved 1.61%, with a paired 95% interval from -2.82% to -0.14%, while
`cold-scan-index` changed 0.47%, with an interval from -1.97% to +1.26%. Both miss the
3% experiment rule. A separate diagnostic against the pinned pre-rewrite control still
found `default-tree` 4.92% slower and `cold-scan-index` 4.01% slower, so eliminating the
clone does not close the campaign gap.

The post-spike profile is useful for the next design even though this edit is rejected.
It leaves only about 0.22 extra allocation and reallocation events per entry, but still
requests roughly 90 extra bytes per entry.
That shape points away from another heap clone and toward the larger always-inline entry
representation: the second `InternedRollUp` occupies every entry even though retained
roll-ups are meaningful only for directories.

## Verdict

**REJECTED.** Scoped allocation fell by about one event and 323 requested bytes per
entry, but `default-tree` improved only 1.61% and `cold-scan-index` 0.47%. Neither met
the 3% rule, and the latter interval crossed zero.
The implementation was removed; its representation evidence carries forward.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
