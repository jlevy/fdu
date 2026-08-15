---
title: Parent-relative openat frontier on the live 1M workspace
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-038
  title: Parent-relative openat frontier on the live 1M workspace
  date: "2026-08-13"
  hypotheses:
    - H24
    - H29
  checkpoint:
    profile: index-core-v1
    kept_variant: control
  subject:
    tree_label: live-workspace-exp038
    tree_root_id: 585f55000d4d135311f162954e1cc5fe3e0a729823acc02400e1c308d57a2949
    tree_engine_digest: 6b33356e83ab04d5b8cf9e149641fd1a1fd368b3a038220403263192611f2fd9
    tree_entries: 1008723
    tree_directories: 113948
    tree_files: 894403
    tree_symlinks: 372
    tree_apparent_bytes: 28063637643
    tree_allocated_bytes: 30466846720
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
    trials: 6
    warmups: 2
    interleaved: true
    control: absolute directory opens
    candidate: bounded parent-relative openat frontier
    control_binary:
      name: control
      sha256: 3da6d0a6c284c6d89958204232a4647e5a9dced0c5316a4060439a2e23f2ff33
      size_bytes: 602192
      args: []
    candidate_binary:
      name: candidate
      sha256: c32dae5df133f96fb7f163254b43e0a036fe3812b7e52dcb0e492dfd8423d741
      size_bytes: 602256
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp038-parent-openat-final.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6315833625.5
          candidate_median: 6272944042.0
          change_pct: -0.694
          ci95_low_pct: -1.49
          ci95_high_pct: 0.491
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        component_ns:
          control_median: 3613718395.5
          candidate_median: 3560869146.0
          change_pct: -1.898
          ci95_low_pct: -2.57
          ci95_high_pct: 1.129
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        cpu_ns:
          control_median: 16966009500.0
          candidate_median: 17105670500.0
          change_pct: 1.234
          ci95_low_pct: -2.011
          ci95_high_pct: 2.485
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        user_cpu_ns:
          control_median: 4446376500.0
          candidate_median: 4457557000.0
          change_pct: 0.249
          ci95_low_pct: -0.752
          ci95_high_pct: 1.491
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        system_cpu_ns:
          control_median: 12528803500.0
          candidate_median: 12657146000.0
          change_pct: 1.439
          ci95_low_pct: -2.443
          ci95_high_pct: 2.982
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        peak_rss_bytes:
          control_median: 479166464.0
          candidate_median: 481804288.0
          change_pct: 0.527
          ci95_low_pct: 0.312
          ci95_high_pct: 1.114
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 6
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - A retained parent descriptor can become stale or exhaust the bounded frontier unless fallback is exact
    notes: Prototype reverted; no dependency or unsafe block retained
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -0.694
    reason: "The final paired screen changes indexed wall by -0.69% with an interval crossing zero; it does not justify descriptor-lifetime machinery and is reverted"
    commit: null
---
# Parent-relative openat frontier on the live 1M workspace

## Hypothesis

H24/H29: now that each directory is enumerated in bulk, repeated absolute-path opens may
be a material remainder.
Carrying a bounded parent directory descriptor and opening children with `openat` should
reduce component and indexed wall time by at least 3% on the live one-million-entry
tree.

## Method

A minimal prototype added a bounded descriptor frontier while retaining the exact
absolute-path fallback.
It ran a six-pair, two-warmup exploratory screen against the unchanged baseline on a
1,008,723-entry fingerprint.
Six pairs are intentionally below the final acceptance gate: the screen was only meant
to decide whether the mechanism showed enough signal to justify its descriptor-lifetime
complexity.

## Results

The candidate changed indexed wall by -0.69% with a 95% interval of -1.49% to +0.49%.
Its component estimate was -1.90% but also crossed zero; total CPU rose 1.23%, RSS rose
0.53%, and minor faults rose 0.87%. The hypothesized open-path saving is not material in
the current macOS bulk path.

## Verdict

**REJECTED** — The final paired screen changes indexed wall by -0.69% with an interval
crossing zero; it does not justify descriptor-lifetime machinery and is reverted

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
