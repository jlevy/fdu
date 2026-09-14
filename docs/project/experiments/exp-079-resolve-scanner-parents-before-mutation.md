---
title: Resolve scanner parents before mutation
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-079
  title: Resolve scanner parents before mutation
  date: "2026-09-01"
  hypotheses:
    - H93
  subject:
    tree_label: resolved-parent-proof-final
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
    control: single exact Commit representation at db18e5e
    candidate: owned scanner batch with resolved parent proof at d9979aa
    control_binary:
      name: control
      sha256: ac295d44435c37dd333c954e700c83efb95241a1a20f759442d515432f6b99f8
      size_bytes: 2123712
      args: []
    candidate_binary:
      name: candidate
      sha256: 70e21532117b18b0bba7e3b52dfcbc1ae9e66b1b7bbebacd2058848bbedeaeec
      size_bytes: 2156752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-resolved-parent-proof-final.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 69108979.0
          candidate_median: 68215500.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.051
          change_pct: 0.299
          ci95_low_pct: -3.414
          ci95_high_pct: 1.799
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 49419479.0
          candidate_median: 49561396.0
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.064
          change_pct: 2.272
          ci95_low_pct: -2.344
          ci95_high_pct: 4.8
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 331658500.0
          candidate_median: 320993500.0
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.082
          change_pct: -3.466
          ci95_low_pct: -6.274
          ci95_high_pct: 1.033
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 53169500.0
          candidate_median: 35312000.0
          control_p95_over_median: 1.273
          candidate_p95_over_median: 1.299
          change_pct: -32.162
          ci95_low_pct: -34.554
          ci95_high_pct: -28.544
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 278332000.0
          candidate_median: 286045500.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.054
          change_pct: 3.752
          ci95_low_pct: -1.127
          ci95_high_pct: 6.912
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 12623872.0
          candidate_median: 11894784.0
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.014
          change_pct: -5.749
          ci95_low_pct: -7.764
          ci95_high_pct: -5.012
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 56265146.0
          candidate_median: 55845417.0
          control_p95_over_median: 1.242
          candidate_p95_over_median: 1.336
          change_pct: -0.423
          ci95_low_pct: -3.811
          ci95_high_pct: 4.076
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 51849499.5
          candidate_median: 51476541.5
          control_p95_over_median: 1.264
          candidate_p95_over_median: 1.333
          change_pct: -0.642
          ci95_low_pct: -4.133
          ci95_high_pct: 4.208
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 329299500.0
          candidate_median: 305561000.0
          control_p95_over_median: 1.285
          candidate_p95_over_median: 1.374
          change_pct: -4.55
          ci95_low_pct: -8.415
          ci95_high_pct: -0.186
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 53910500.0
          candidate_median: 31867500.0
          control_p95_over_median: 1.07
          candidate_p95_over_median: 1.107
          change_pct: -39.595
          ci95_low_pct: -41.772
          ci95_high_pct: -36.167
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 278069500.0
          candidate_median: 282262000.0
          control_p95_over_median: 1.302
          candidate_p95_over_median: 1.368
          change_pct: 2.21
          ci95_low_pct: -4.096
          ci95_high_pct: 5.889
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 14483456.0
          candidate_median: 13746176.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.021
          change_pct: -5.234
          ci95_low_pct: -5.842
          ci95_high_pct: -3.341
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
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 308953979.0
          candidate_median: 280750854.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.01
          change_pct: -9.5
          ci95_low_pct: -10.895
          ci95_high_pct: -8.136
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 156307083.5
          candidate_median: 146397958.5
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.019
          change_pct: -6.81
          ci95_low_pct: -7.787
          ci95_high_pct: -5.559
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 324693500.0
          candidate_median: 295454500.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.012
          change_pct: -9.187
          ci95_low_pct: -10.783
          ci95_high_pct: -7.94
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 173778500.0
          candidate_median: 143231000.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.008
          change_pct: -17.835
          ci95_low_pct: -18.516
          ci95_high_pct: -17.268
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 151249000.0
          candidate_median: 152193000.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.016
          change_pct: 0.487
          ci95_low_pct: -1.237
          ci95_high_pct: 2.984
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 37822464.0
          candidate_median: 37167104.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.004
          change_pct: -2.146
          ci95_low_pct: -2.55
          ci95_high_pct: -1.561
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
      wall_ns_median: 95584750.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 572
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Adds one private owned-batch/proof lane with no dependency, unsafe block, public trust flag, or alternate commit/fact reducer; public, refresh, and watch observations retain general atomic preflight."
  verdict:
    decision: accepted
    primary_job: opened-discovery
    primary_metric: wall_ns
    change_pct: -9.5
    reason: "Opened discovery improved 9.50% with a paired 95% interval of -10.89% to -8.14%; default and cold one-shot paths were noninferior, scoped allocations fell, and the final stack matched or beat the pre-rewrite control."
    commit: d9979aa
---
# Resolve scanner parents before mutation

## Hypothesis

H93: Scanner output is already canonical, parent-first, and owned, but detached and
opened discovery convert it to the general public observation form and rebuild a
path-keyed `StructuralOverlay` before mutation.
Preparing numeric parent references while the index is locked should remove that
duplicate path ownership and ancestry work, reducing allocation and opened-discovery
component time without changing one-shot performance or exact streaming results.

## What was tried

The scanner now produces a private owned `ScannerBatch`. Public `scan()` can still
return an `Observation` by moving the operations without copying their paths, while cold
construction and opened discovery pass the private batch directly to index preparation.

Scanner preparation accepts only canonical, unconditional discovery upserts and resolves
each parent to either an existing `EntryId` or an earlier operation in the same batch.
It rejects missing parents and entry-kind replacement before mutation.
Public observations, refresh, and watch retain the general atomic preflight; there is no
public trust flag. Both prepared forms enter the same fact reducer, control-state
handling, commit builder, impact derivation, and bounded journal.

The first timing attempt accidentally named the Cargo registry parent rather than the
fingerprinted source tree and was discarded in full.
The recorded run compares the saved `db18e5e` control and `d9979aa` candidate over 12
interleaved pairs on the exact stable 11,141-entry corpus.
Every engine and commit oracle passed, with no invalid sample, baseline drift, or tree
mutation.

## What the numbers said

Opened-discovery wall time improved 9.50%, with a paired 95% interval from -10.89% to
-8.14%; its measured engine component improved 6.81%. Scoped allocation events fell from
489,514 to 405,751. The candidate recorded zero ancestry-overlay insertions and zero
second-stage parent resolutions on this parent-first stream.

Cold scoped allocations fell from 162,071 to 109,568. Its wall result changed by +0.30%,
with an interval from -3.41% to +1.80%, and the default-path result changed by -0.42%,
with an interval crossing zero.
The optimization therefore removed material work without moving either one-shot path
outside its noninferiority bound.

A separate 12-pair comparison against the preserved pre-rewrite binary checked the
campaign goal rather than this experiment’s incremental effect.
The complete stack was 6.07% faster on `default-tree`, with a paired 95% interval from
-7.68% to -0.59%. `cold-scan-index` was 1.50% faster by median, with an interval from
-5.64% to +0.17%, which establishes wall-time parity on this corpus.

## Verdict

**ACCEPTED.** The private proof removes the general ancestry overlay only where scanner
provenance makes the stronger invariant checkable.
It materially improves opened discovery, reduces one-shot allocation, and preserves the
general public mutation contract.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
