---
title: Move directory-only state out of line
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-100
  title: Move directory-only state out of line
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
    candidate: out-of-line directory payload with inline arena entries
    control_binary:
      name: control
      sha256: 9f3d5963164e0580d008e6b0861d8b20c7fe2427df42a4e4945b389c398f8549
      size_bytes: 2222832
      args: []
    candidate_binary:
      name: candidate
      sha256: a5a769edb8bd9f3a5b0ad62d63ed95c433c2315344763853b9e107d1913ebd10
      size_bytes: 2222848
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-directory-payload-inline-arena.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 579703271.0
          candidate_median: 564978833.5
          control_p95_over_median: 1.084
          candidate_p95_over_median: 1.046
          change_pct: -3.18
          ci95_low_pct: -6.812
          ci95_high_pct: 0.61
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 334452291.5
          candidate_median: 334696812.5
          control_p95_over_median: 1.189
          candidate_p95_over_median: 1.074
          change_pct: -1.446
          ci95_low_pct: -7.078
          ci95_high_pct: 0.102
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2209722500.0
          candidate_median: 2225988500.0
          control_p95_over_median: 1.172
          candidate_p95_over_median: 1.059
          change_pct: -1.813
          ci95_low_pct: -10.353
          ci95_high_pct: 0.38
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 351126500.0
          candidate_median: 342827000.0
          control_p95_over_median: 1.118
          candidate_p95_over_median: 1.035
          change_pct: -2.557
          ci95_low_pct: -7.36
          ci95_high_pct: 0.329
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1869897500.0
          candidate_median: 1895727000.0
          control_p95_over_median: 1.201
          candidate_p95_over_median: 1.059
          change_pct: -1.382
          ci95_low_pct: -10.997
          ci95_high_pct: 0.219
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 75866112.0
          candidate_median: 53927936.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.016
          change_pct: -28.999
          ci95_low_pct: -29.284
          ci95_high_pct: -28.57
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
          control_median: 355927645.5
          candidate_median: 350591124.5
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.072
          change_pct: -0.829
          ci95_low_pct: -3.326
          ci95_high_pct: 0.741
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 351374291.5
          candidate_median: 346066500.5
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.072
          change_pct: -0.816
          ci95_low_pct: -3.327
          ci95_high_pct: 0.775
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2007039000.0
          candidate_median: 2014340000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.072
          change_pct: 1.149
          ci95_low_pct: -1.711
          ci95_high_pct: 2.773
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 136087500.0
          candidate_median: 129060000.0
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.09
          change_pct: -5.306
          ci95_low_pct: -6.629
          ci95_high_pct: -1.34
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1870535000.0
          candidate_median: 1887362500.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.065
          change_pct: 1.615
          ci95_low_pct: -1.192
          ci95_high_pct: 3.217
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 88260608.0
          candidate_median: 66379776.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.01
          change_pct: -24.634
          ci95_low_pct: -25.244
          ci95_high_pct: -23.844
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
          control_median: 3214412833.0
          candidate_median: 3271962854.0
          control_p95_over_median: 1.615
          candidate_p95_over_median: 1.347
          change_pct: -0.344
          ci95_low_pct: -2.359
          ci95_high_pct: 6.424
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1872991146.0
          candidate_median: 1867449917.0
          control_p95_over_median: 1.926
          candidate_p95_over_median: 1.626
          change_pct: -0.245
          ci95_low_pct: -1.622
          ci95_high_pct: 1.344
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 3549802500.0
          candidate_median: 3539665500.0
          control_p95_over_median: 1.235
          candidate_p95_over_median: 1.181
          change_pct: -0.586
          ci95_low_pct: -2.621
          ci95_high_pct: 2.194
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 2236315000.0
          candidate_median: 2249373500.0
          control_p95_over_median: 1.148
          candidate_p95_over_median: 1.141
          change_pct: -0.691
          ci95_low_pct: -1.518
          ci95_high_pct: 1.47
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1325623500.0
          candidate_median: 1290292000.0
          control_p95_over_median: 1.281
          candidate_p95_over_median: 1.249
          change_pct: -0.75
          ci95_low_pct: -4.619
          ci95_high_pct: 4.243
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 379830272.0
          candidate_median: 335388672.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.005
          change_pct: -11.656
          ci95_low_pct: -11.856
          ci95_high_pct: -11.53
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
      wall_ns_median: 497197708.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - An extra allocation on every directory could trade file density for directory pointer chasing
    notes: Intermediate uncommitted checkpoint; the source line count was not preserved after the next test-first arm superseded it.
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -0.829
    reason: "Default-tree wall changed -0.83% [-3.33%, +0.74%], missing the preregistered 3% gate despite 24.63% lower RSS."
    commit: null
---
# Move directory-only state out of line

## Hypothesis

H86: the 280-byte common entry and its separate arena allocation account for enough of
the retained memory regression that moving directory-only state behind one directory
pointer and storing entries inline should recover at least 3% on `default-tree` while
reducing peak RSS by at least 20%.

## What was tried

Child topology, both roll-up partitions, child revision, and discovery completion moved
from every entry into a `DirectoryEntry` allocated only for directories.
The slot arena stores its `Entry` inline instead of retaining one `Box<Entry>` per live
entry.
The index still uses its existing ordered child map, public mutation path, roll-up
model, and observation contract.

## What the numbers said

Across twelve paired uncontrolled trials against the exact `88304cb` control,
`cold-scan-index` wall changed -3.18%, with a paired 95% interval from -6.81% to +0.61%.
`default-tree` changed only -0.83%, with an interval from -3.33% to +0.74%. Peak RSS
fell 29.00% and 24.63%, respectively, and minor faults fell in the same direction.
Opened discovery wall was inconclusive at -0.34%, with an interval from -2.36% to
+6.42%, while its peak RSS fell 11.66%.

The memory attribution was correct, but the common-layout change alone did not produce
the required default-command wall improvement.
The host was outside the quiet-load limit, which prevents a claim-grade timing verdict
but does not turn an interval crossing zero into evidence for the preregistered 3% gain.

## Verdict

**REJECTED.** The arm misses the wall gate despite a material memory improvement.
It is retained only as a constituent of
[exp-101](exp-101-compact-detached-child-topology-with-local-promotion.md), where the
second measured mechanism is evaluated against the original control.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
