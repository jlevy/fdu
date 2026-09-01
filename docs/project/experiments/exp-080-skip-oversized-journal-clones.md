---
title: Skip oversized journal clones
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-080
  title: Skip oversized journal clones
  date: "2026-09-01"
  hypotheses:
    - H94
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
    control: resolved-parent proof at d9979aa
    candidate: journal capacity preflight at e2ac4f9
    control_binary:
      name: control
      sha256: 70e21532117b18b0bba7e3b52dfcbc1ae9e66b1b7bbebacd2058848bbedeaeec
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: ed8ebcef415f9f1a7944d57cfa1b84961dde602545404cd8a6c03749421bf870
      size_bytes: 2156752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-journal-preflight-clone.json
  results:
    - job: delta-apply-batched
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 579570312.0
          candidate_median: 570456729.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.034
          change_pct: -1.627
          ci95_low_pct: -1.755
          ci95_high_pct: -1.073
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 330695333.5
          candidate_median: 321882333.5
          control_p95_over_median: 1.028
          candidate_p95_over_median: 1.033
          change_pct: -2.495
          ci95_low_pct: -2.828
          ci95_high_pct: -2.095
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 576174000.0
          candidate_median: 567374500.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.028
          change_pct: -1.609
          ci95_low_pct: -2.074
          ci95_high_pct: -1.072
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 559567000.0
          candidate_median: 550196000.0
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.026
          change_pct: -1.676
          ci95_low_pct: -1.821
          ci95_high_pct: -1.189
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 16578500.0
          candidate_median: 17253000.0
          control_p95_over_median: 1.112
          candidate_p95_over_median: 1.066
          change_pct: -0.07
          ci95_low_pct: -5.416
          ci95_high_pct: 7.272
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 3415312.5
          candidate_median: 3440729.0
          control_p95_over_median: 1.098
          candidate_p95_over_median: 1.152
          change_pct: -0.783
          ci95_low_pct: -5.043
          ci95_high_pct: 50.563
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 120987648.0
          candidate_median: 119562240.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.01
          change_pct: -0.986
          ci95_low_pct: -1.633
          ci95_high_pct: -0.369
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
          control_median: 677610187.5
          candidate_median: 654883187.5
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.014
          change_pct: -3.46
          ci95_low_pct: -4.323
          ci95_high_pct: -2.607
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 427688604.0
          candidate_median: 407545708.5
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.013
          change_pct: -4.517
          ci95_low_pct: -5.427
          ci95_high_pct: -3.99
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 674479000.0
          candidate_median: 651480500.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.015
          change_pct: -3.468
          ci95_low_pct: -4.393
          ci95_high_pct: -2.642
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 654600000.0
          candidate_median: 632279500.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.012
          change_pct: -3.514
          ci95_low_pct: -3.836
          ci95_high_pct: -2.68
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 20908500.0
          candidate_median: 19648500.0
          control_p95_over_median: 1.226
          candidate_p95_over_median: 1.124
          change_pct: -6.288
          ci95_low_pct: -16.863
          ci95_high_pct: 1.442
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        blocked_ns:
          control_median: 3230874.5
          candidate_median: 3296938.0
          control_p95_over_median: 1.516
          candidate_p95_over_median: 1.309
          change_pct: 1.315
          ci95_low_pct: -8.173
          ci95_high_pct: 20.249
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 166764544.0
          candidate_median: 149299200.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.031
          change_pct: -10.49
          ci95_low_pct: -11.563
          ci95_high_pct: -7.714
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 278685770.5
          candidate_median: 280409521.0
          control_p95_over_median: 1.039
          candidate_p95_over_median: 1.034
          change_pct: 0.073
          ci95_low_pct: -1.115
          ci95_high_pct: 0.831
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 146153708.0
          candidate_median: 145960771.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.023
          change_pct: 0.467
          ci95_low_pct: -1.002
          ci95_high_pct: 1.075
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 293863500.0
          candidate_median: 295032000.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.051
          change_pct: -0.132
          ci95_low_pct: -1.593
          ci95_high_pct: 1.151
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 143025000.0
          candidate_median: 143026500.0
          control_p95_over_median: 1.03
          candidate_p95_over_median: 1.049
          change_pct: -0.247
          ci95_low_pct: -1.177
          ci95_high_pct: 0.688
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 151009000.0
          candidate_median: 151604500.0
          control_p95_over_median: 1.07
          candidate_p95_over_median: 1.047
          change_pct: -0.216
          ci95_low_pct: -2.031
          ci95_high_pct: 1.121
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 37085184.0
          candidate_median: 37036032.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.006
          change_pct: -0.485
          ci95_low_pct: -0.664
          ci95_high_pct: -0.066
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
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
      wall_ns_median: 95524062.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 12
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Moves the existing journal capacity decision before cloning, deletes two net lines, and adds no dependency, unsafe block, public API, alternate representation, or failure mode."
  verdict:
    decision: accepted
    primary_job: delta-apply-large
    primary_metric: wall_ns
    change_pct: -3.46
    reason: "Large exact-batch wall time improved 3.46% with a paired 95% interval of -4.32% to -2.61%; 100003 scoped allocations disappeared, batched exact updates also improved, and opened discovery was unchanged."
    commit: e2ac4f9
---
# Skip oversized journal clones

## Hypothesis

H94: Exact publication clones every returned `Commit` before the journal checks whether
its retained cost exceeds capacity.
The 100,001-change large-batch job must reject that clone immediately, so moving the
existing capacity decision ahead of cloning should remove one path allocation per change
and improve `delta-apply-large` without affecting retained history.

## What was tried

`retain_commit` now borrows the returned commit, computes its existing retained cost,
and handles the oversized case before making an owned journal copy.
Only commits that fit the journal are cloned and counted as cloned.
The returned exact commit, journal floor, eviction rules, and cost model are unchanged.

The saved `d9979aa` release binary and the `e2ac4f9` candidate ran in 12 interleaved
pairs on the stable 11,141-entry corpus.
The run covered one oversized 100,001-operation commit, the same work in 4,096-operation
batches, and opened discovery, where every small commit is retained.
All engine and commit digests matched, and the run recorded no invalid sample, baseline
drift, or tree mutation.

## What the numbers said

The large exact batch eliminated 100,003 scoped allocations and 15.7 MB of scoped
allocation. Median wall time improved 3.46%, with a paired 95% interval from -4.32% to
-2.61%; the measured engine component improved 4.71%.

The batched case, whose commits all fit and therefore still require clones, improved
1.63%, with an interval entirely below zero.
Opened discovery changed by +0.07%, with an interval from -1.11% to +0.83%, and remained
noninferior. The counters distinguish the mechanism directly: the oversized case changed
from one clone and one rejection to zero clones and one rejection, while the retained
cases kept their exact journal counts.

## Verdict

**ACCEPTED.** The existing journal bound now prevents work instead of discarding it.
The change deletes two net lines, clears the timing threshold, and preserves exact
history semantics.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
