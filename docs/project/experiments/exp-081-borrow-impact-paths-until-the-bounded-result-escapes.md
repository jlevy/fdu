---
title: Borrow impact paths until the bounded result escapes
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-081
  title: Borrow impact paths until the bounded result escapes
  date: "2026-09-01"
  hypotheses:
    - H95
  subject:
    tree_label: cargo-registry-src-v2
    tree_root_id: 0d6ac3b56b7696752b6af951b3802fd843b8d1235fa49cad9f2a2214cd8e403b
    tree_engine_digest: 1c2f63e8a0cb7ff48e2ba2380715832093ef125973190619c9973a79aebeea63
    tree_provenance: Live local Cargo registry source cache observed in place; package-manager state and exact tree shape are not reconstructible.
    tree_reconstructible: false
    tree_entries: 11141
    tree_directories: 2240
    tree_files: 8901
    tree_symlinks: 0
    tree_apparent_bytes: 179605080
    tree_allocated_bytes: 203902976
    tree_max_depth: 9
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
    control: journal capacity preflight at e2ac4f9
    candidate: borrowed impact-path working-tree spike
    control_binary:
      name: control
      sha256: ed8ebcef415f9f1a7944d57cfa1b84961dde602545404cd8a6c03749421bf870
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: 5be9a00c6ba74628cb4e96c196705ef6eb910d1441a440443ec6bbadd99c9af4
      size_bytes: 2156752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-borrow-impact-paths.json
  results:
    - job: delta-apply-batched
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 565515500.0
          candidate_median: 563853625.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.027
          change_pct: 0.762
          ci95_low_pct: -0.51
          ci95_high_pct: 1.192
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 321984833.0
          candidate_median: 321532188.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.024
          change_pct: 0.966
          ci95_low_pct: 0.254
          ci95_high_pct: 1.614
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 563094000.0
          candidate_median: 559754000.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.027
          change_pct: 0.799
          ci95_low_pct: -0.487
          ci95_high_pct: 1.112
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 546772500.0
          candidate_median: 544795500.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.029
          change_pct: 0.664
          ci95_low_pct: -0.227
          ci95_high_pct: 1.099
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 15505500.0
          candidate_median: 14991000.0
          control_p95_over_median: 1.09
          candidate_p95_over_median: 1.195
          change_pct: 1.542
          ci95_low_pct: -9.232
          ci95_high_pct: 5.503
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 3301875.0
          candidate_median: 3488208.0
          control_p95_over_median: 1.079
          candidate_p95_over_median: 1.38
          change_pct: 4.975
          ci95_low_pct: 0.951
          ci95_high_pct: 101.097
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 119488512.0
          candidate_median: 120217600.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.009
          change_pct: 0.568
          ci95_low_pct: 0.048
          ci95_high_pct: 1.14
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
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
    - job: delta-apply-large
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 643941396.5
          candidate_median: 654022521.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.017
          change_pct: 0.916
          ci95_low_pct: -0.043
          ci95_high_pct: 1.977
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 401569750.5
          candidate_median: 408719083.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.027
          change_pct: 2.312
          ci95_low_pct: 1.5
          ci95_high_pct: 3.499
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 640849500.0
          candidate_median: 651171500.0
          control_p95_over_median: 1.028
          candidate_p95_over_median: 1.016
          change_pct: 1.086
          ci95_low_pct: -0.044
          ci95_high_pct: 1.752
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 624788500.0
          candidate_median: 633619000.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.017
          change_pct: 1.082
          ci95_low_pct: -0.052
          ci95_high_pct: 1.506
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 17628000.0
          candidate_median: 17422500.0
          control_p95_over_median: 1.166
          candidate_p95_over_median: 1.167
          change_pct: -0.786
          ci95_low_pct: -5.317
          ci95_high_pct: 4.829
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 3223500.5
          candidate_median: 3229604.0
          control_p95_over_median: 1.153
          candidate_p95_over_median: 1.036
          change_pct: -1.236
          ci95_low_pct: -11.331
          ci95_high_pct: 2.977
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 149266432.0
          candidate_median: 150650880.0
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.014
          change_pct: 0.939
          ci95_low_pct: -0.578
          ci95_high_pct: 1.252
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
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
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 286836833.5
          candidate_median: 282203187.5
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.096
          change_pct: -1.071
          ci95_low_pct: -3.997
          ci95_high_pct: 1.821
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 144841187.0
          candidate_median: 142765646.0
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.136
          change_pct: -1.493
          ci95_low_pct: -4.552
          ci95_high_pct: 4.316
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 306372500.0
          candidate_median: 300389500.0
          control_p95_over_median: 1.091
          candidate_p95_over_median: 1.097
          change_pct: -1.155
          ci95_low_pct: -3.798
          ci95_high_pct: 0.978
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 146145000.0
          candidate_median: 145355000.0
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.043
          change_pct: -0.95
          ci95_low_pct: -1.798
          ci95_high_pct: 0.262
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 157892500.0
          candidate_median: 154286000.0
          control_p95_over_median: 1.14
          candidate_p95_over_median: 1.154
          change_pct: -1.657
          ci95_low_pct: -6.267
          ci95_high_pct: 3.457
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 36487168.0
          candidate_median: 36716544.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.018
          change_pct: 0.452
          ci95_low_pct: -0.793
          ci95_high_pct: 1.766
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
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
  reference_tools:
    - name: dust
      wall_ns_median: 91708416.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 10
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Replaces temporary owned PathBuf values with borrowed Path references and clones only the bounded escaping set; adds no dependency, unsafe block, public API, or failure mode, but does add lifetime coupling without a demonstrated timing benefit."
  verdict:
    decision: rejected
    primary_job: opened-discovery
    primary_metric: wall_ns
    change_pct: -1.071
    reason: "Opened scoped allocations fell 8.2%, but wall time improved only 1.07% with a paired 95% interval crossing zero; large exact-batch wall time moved 0.92% slower and batched timing was unchanged."
    commit: null
---
# Borrow impact paths until the bounded result escapes

## Hypothesis

H95: Exact impact discovery copies candidate and ancestor paths into a temporary ordered
set even though only the bounded final result escapes publication.
Keeping borrowed paths in that working set and cloning only the escaping result should
reduce opened-discovery allocation and elapsed time without changing impact order or
contents.

## What was tried

The working set in exact impact discovery temporarily changed from `BTreeSet<PathBuf>`
to `BTreeSet<&Path>`. The existing limit, ancestor walk, ordering, and final owned
`Commit` representation were unchanged; only paths admitted to the bounded result were
cloned.

The saved `e2ac4f9` release binary and the working-tree candidate ran in 12 interleaved
pairs on the stable 11,141-entry corpus.
The run covered opened discovery, one oversized 100,001-operation exact commit, and the
same exact work split into 4,096-operation batches.
All engine and commit digests matched, and the run recorded no invalid sample, baseline
drift, or tree mutation.

## What the numbers said

Opened scoped allocations fell from 404,453 to 371,177, an 8.2% reduction, but median
wall time improved only 1.07% and its paired 95% interval ran from -4.00% to +1.82%.
Opened component time improved 1.49%, also with an interval crossing zero.

The exact-update jobs did not benefit.
Large-batch wall time moved 0.92% slower and component time regressed 2.31%, with the
component interval wholly above zero.
Batched wall and component time moved 0.76% and 0.97% slower.
Allocation count alone therefore overstated the cost of owning these bounded paths, and
the borrowed representation introduced lifetime coupling without a supported timing
gain.

## Verdict

**REJECTED.** Opened scoped allocations fell 8.2%, but wall time improved only 1.07%
with a paired 95% interval crossing zero; large exact-batch wall time moved 0.92% slower
and batched timing was unchanged.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
