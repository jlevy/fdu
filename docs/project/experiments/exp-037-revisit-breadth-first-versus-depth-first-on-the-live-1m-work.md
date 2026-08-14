---
title: Revisit breadth-first versus depth-first on the live 1M workspace
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-037
  title: Revisit breadth-first versus depth-first on the live 1M workspace
  date: "2026-08-13"
  hypotheses:
    - H4
  subject:
    tree_label: live-workspace-20260812
    tree_root_id: 585f55000d4d135311f162954e1cc5fe3e0a729823acc02400e1c308d57a2949
    tree_engine_digest: df4efc6129670710450b4fba2e895aab6699c64f288e0f6c680f42f40b80231b
    tree_entries: 1007659
    tree_directories: 113932
    tree_files: 893355
    tree_symlinks: 372
    tree_apparent_bytes: 27887068841
    tree_allocated_bytes: 30288072704
    tree_max_depth: 24
    tree_mutated_during_run: false
    host_cpu: Apple M1 Pro
    host_arch: arm64
    host_cores: 10
    host_performance_cores: 8
    host_efficiency_cores: 2
    host_memory_bytes: 34359738368
    host_system: Darwin 25.5.0
    filesystem: apfs
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: region-scheduled breadth-first default
    candidate: depth-first traversal
    control_binary:
      name: breadth
      sha256: 3da6d0a6c284c6d89958204232a4647e5a9dced0c5316a4060439a2e23f2ff33
      size_bytes: 602192
      args:
        - "--order"
        - breadth-first
    candidate_binary:
      name: depth
      sha256: 3da6d0a6c284c6d89958204232a4647e5a9dced0c5316a4060439a2e23f2ff33
      size_bytes: 602192
      args:
        - "--order"
        - depth-first
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp037-live-order-revisit.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6187496646.0
          candidate_median: 6444590187.0
          change_pct: 3.573
          ci95_low_pct: 2.419
          ci95_high_pct: 5.227
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        component_ns:
          control_median: 3565910750.5
          candidate_median: 3839281562.5
          change_pct: 6.722
          ci95_low_pct: 3.755
          ci95_high_pct: 9.266
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        cpu_ns:
          control_median: 16656342500.0
          candidate_median: 16532786500.0
          change_pct: -0.499
          ci95_low_pct: -1.202
          ci95_high_pct: 0.468
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 4356457000.0
          candidate_median: 4328379000.0
          change_pct: -0.341
          ci95_low_pct: -0.668
          ci95_high_pct: 0.041
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 12323222000.0
          candidate_median: 12186633000.0
          change_pct: -0.67
          ci95_low_pct: -1.726
          ci95_high_pct: 0.844
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 480673792.0
          candidate_median: 475004928.0
          change_pct: -1.03
          ci95_low_pct: -1.492
          ci95_high_pct: -0.553
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: No retained code; one-binary order comparison
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 3.573
    reason: "Depth-first regresses indexed wall 3.57% with a 95% paired interval of 2.42% to 5.23%; the breadth-first region scheduler remains faster as well as preserving progressive shallow coverage"
    commit: null
---
# Revisit breadth-first versus depth-first on the live 1M workspace

## Hypothesis

H4: changes to breadth-first region scheduling may have made its queue-management and
locality costs worse than depth-first traversal on a heterogeneous million-entry tree.
If so, depth-first order should improve indexed wall time by at least 3%; any memory win
must be weighed against fdu’s progressive shallow-coverage guarantee.

## Method

The same immutable binary ran breadth-first and depth-first traversal in twelve
interleaved pairs after three warmups on the 1,007,659-entry workspace.
Traversal order was the only configuration difference.
Every timed scan matched the exact oracle and the tree’s pre/post fingerprints agreed.

## Results

Breadth-first completed in 6.1875 seconds median versus 6.4446 seconds for depth-first.
Depth-first regressed paired wall time 3.57% [2.42%, 5.23%] and the scan/index component
6.72%. It reduced peak RSS by only 1.03% and minor faults by 0.77%, too little to offset
the speed loss or the loss of useful shallow-first progress.

## Verdict

**REJECTED** — Depth-first regresses indexed wall 3.57% with a 95% paired interval of
2.42% to 5.23%; the breadth-first region scheduler remains faster as well as preserving
progressive shallow coverage

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
