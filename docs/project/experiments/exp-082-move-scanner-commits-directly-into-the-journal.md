---
title: Move scanner commits directly into the journal
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-082
  title: Move scanner commits directly into the journal
  date: "2026-09-01"
  hypotheses:
    - H96
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
    candidate: journal-owned scanner commit working-tree spike
    control_binary:
      name: control
      sha256: ed8ebcef415f9f1a7944d57cfa1b84961dde602545404cd8a6c03749421bf870
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: candidate
      sha256: af4d7bf2fae32b0467c03c1e1fca27e7f08ffc388f947c8fff29febc84231147
      size_bytes: 2156752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-move-scanner-commits-to-journal.json
  results:
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 284497021.0
          candidate_median: 281179854.5
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.179
          change_pct: -0.013
          ci95_low_pct: -2.152
          ci95_high_pct: 5.773
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 145539416.5
          candidate_median: 144610895.5
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.085
          change_pct: -0.363
          ci95_low_pct: -1.598
          ci95_high_pct: 4.847
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 299197000.0
          candidate_median: 294319000.0
          control_p95_over_median: 1.05
          candidate_p95_over_median: 1.105
          change_pct: -0.383
          ci95_low_pct: -2.706
          ci95_high_pct: 4.27
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 142434000.0
          candidate_median: 140947000.0
          control_p95_over_median: 1.039
          candidate_p95_over_median: 1.039
          change_pct: -0.741
          ci95_low_pct: -1.814
          ci95_high_pct: 1.238
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 155341500.0
          candidate_median: 152933000.0
          control_p95_over_median: 1.103
          candidate_p95_over_median: 1.203
          change_pct: 0.695
          ci95_low_pct: -4.071
          ci95_high_pct: 7.046
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 36937728.0
          candidate_median: 37912576.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.01
          change_pct: 2.765
          ci95_low_pct: 1.79
          ci95_high_pct: 4.675
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
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
      wall_ns_median: 101152208.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 53
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Adds an internal scanner receipt and separate owned journal-retention route so discovery can move rather than clone commits; no public API, dependency, unsafe block, or semantic oracle changes, but 31 net lines and another result form are not justified by the timing."
  verdict:
    decision: rejected
    primary_job: opened-discovery
    primary_metric: wall_ns
    change_pct: -0.013
    reason: "Opened scoped allocations fell roughly 10.3% and scanner journal clones nearly disappeared, but wall time changed -0.01% with a paired 95% interval of -2.15% to +5.77%; component time was also unchanged and the candidate added a second private result form."
    commit: null
---
# Move scanner commits directly into the journal

## Hypothesis

H96: Opened discovery returns each exact scanner commit and then clones it into the
journal even though the caller only needs aggregate apply statistics.
Moving scanner commit ownership directly into the journal should remove one path clone
per discovered entry and improve opened-discovery elapsed time.

## What was tried

The scanner-only path temporarily returned a private receipt containing apply statistics
rather than a public `Commit`. Publication moved the owned scanner commit directly into
the journal, while public mutation, refresh, and watch observations kept the existing
exact returned commit.
The spike added a second private result form and a separate owned retention route.

The saved `e2ac4f9` release binary and the working-tree candidate ran in 12 interleaved
opened-discovery pairs on the stable 11,141-entry corpus.
All engine and commit digests matched, and the run recorded no invalid sample, baseline
drift, or tree mutation.
The candidate also passed the all-feature and no-default-feature core suites before the
spike was removed.

## What the numbers said

Scoped allocations fell from roughly 405,000 to 363,863, and scanner journal clones fell
from 2,242 to zero; two non-scanner lifecycle clones remained.
Despite that reduction, median opened wall time changed by only -0.01%, with a paired
95% interval from -2.15% to +5.77%. Component time changed by -0.36%, also with an
interval crossing zero, and peak RSS moved 2.77% higher.

The removed ownership was real but was not a leading elapsed-time cost.
Keeping a 31-net-line scanner-specific receipt and second publication route would have
made the ownership model harder to reason about without a measured benefit.

## Verdict

**REJECTED.** Opened scoped allocations fell roughly 10.3% and scanner journal clones
nearly disappeared, but wall time changed -0.01% with a paired 95% interval of -2.15% to
+5.77%; component time was also unchanged and the candidate added a second private
result form.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
