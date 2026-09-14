---
title: Monomorphize shared concurrent-walk consumption
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-099
  title: Monomorphize shared concurrent-walk consumption
  date: "2026-09-01"
  hypotheses:
    - H86
  subject:
    tree_label: metabrowser-h86-orchestration-cleanup
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
    control: controls-aware detached builder before pool-orchestration cleanup
    candidate: shared generic runner with monomorphized message consumption
    control_binary:
      name: control
      sha256: 42d98010f2f4c442ef89ed644fdc634a12b3a19c75a219f60e5c5f302e0d078b
      size_bytes: 2239360
      args: []
    candidate_binary:
      name: candidate
      sha256: 660291b2fe947013acc3e3f388cee4a92b11e8b722c1335770577b798aeeea3e
      size_bytes: 2222832
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h86-shared-orchestration-monomorphized-exploratory.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 569339624.5
          candidate_median: 565988708.0
          control_p95_over_median: 1.034
          candidate_p95_over_median: 1.048
          change_pct: 0.159
          ci95_low_pct: -1.455
          ci95_high_pct: 0.808
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 330741750.0
          candidate_median: 332154437.5
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.061
          change_pct: 0.158
          ci95_low_pct: -1.558
          ci95_high_pct: 1.058
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2252968500.0
          candidate_median: 2259254000.0
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.056
          change_pct: 0.021
          ci95_low_pct: -1.127
          ci95_high_pct: 0.778
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 343342500.0
          candidate_median: 340001500.0
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.029
          change_pct: -1.11
          ci95_low_pct: -2.739
          ci95_high_pct: 2.133
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1906609000.0
          candidate_median: 1916214000.0
          control_p95_over_median: 1.048
          candidate_p95_over_median: 1.063
          change_pct: 0.171
          ci95_low_pct: -1.014
          ci95_high_pct: 0.957
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 75530240.0
          candidate_median: 75448320.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.006
          change_pct: -0.098
          ci95_low_pct: -0.304
          ci95_high_pct: 0.325
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
  reference_tools:
    - name: dust
      wall_ns_median: 483445583.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 7
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - A future trait-object consumer at the handoff boundary could restore the rejected per-message cost.
    notes: "One generic bound keeps termination, scaling, diagnostics, joins, and error ordering singular while monomorphizing the two consumers."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 0.159
    reason: "The shared source is noninferior at +0.16% [-1.46%, +0.81%] after removing per-message dynamic dispatch."
    commit: null
---
# Monomorphize shared concurrent-walk consumption

## Hypothesis

H86: a generic consumer bound can preserve the singular pool implementation while
letting Rust specialize the streaming and detached message paths.
If the shared structure is sound, the final candidate should remain within the +3%
noninferiority bound against the preserved pre-cleanup binary.

## What was tried

`run_concurrent_walk` now accepts `C: FnMut(WalkMessage)` instead of a trait object.
The worker entry remains a function pointer invoked only when a worker starts; message
consumption is monomorphized for each of the two callers.
Termination, adaptive scaling, diagnostics, panic handling, joins, and deterministic
error ordering remain in one source implementation.

## What the numbers said

Across twelve fresh interleaved pairs, wall time changed +0.16%, with a paired 95%
interval from -1.46% to +0.81%. The scan component also changed +0.16%, with an interval
from -1.56% to +1.06%; CPU, system time, and peak RSS were noninferior.
Every exact oracle passed and the 113,794-entry tree remained unchanged.

## Verdict

**ACCEPTED.** This is a complexity acceptance under the predeclared noninferiority
margin, not a speed claim.
The generic boundary keeps the orchestration singular without retaining the rejected
trait-object form.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
