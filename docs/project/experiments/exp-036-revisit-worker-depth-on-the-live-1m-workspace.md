---
title: Revisit worker depth on the live 1M workspace
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-036
  title: Revisit worker depth on the live 1M workspace
  date: "2026-08-12"
  hypotheses:
    - H57
  checkpoint:
    profile: index-core-v1
    kept_variant: control
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
    control: automatic adaptive worker policy
    candidate: "fixed 8, 10, 12, and 16-worker sweep"
    control_binary:
      name: auto
      sha256: 3da6d0a6c284c6d89958204232a4647e5a9dced0c5316a4060439a2e23f2ff33
      size_bytes: 602192
      args: []
    candidate_binary:
      name: t8
      sha256: 3da6d0a6c284c6d89958204232a4647e5a9dced0c5316a4060439a2e23f2ff33
      size_bytes: 602192
      args:
        - "--threads"
        - "8"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp036-live-worker-sweep.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6188867667.0
          candidate_median: 6113609083.5
          change_pct: -1.302
          ci95_low_pct: -2.062
          ci95_high_pct: -0.92
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 3578920792.0
          candidate_median: 3490176104.0
          change_pct: -2.43
          ci95_low_pct: -2.843
          ci95_high_pct: -1.745
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 16488256000.0
          candidate_median: 22152392500.0
          change_pct: 34.465
          ci95_low_pct: 33.256
          ci95_high_pct: 35.571
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 4333503500.0
          candidate_median: 4419337000.0
          change_pct: 1.795
          ci95_low_pct: 1.337
          ci95_high_pct: 2.603
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 12160054500.0
          candidate_median: 17702267000.0
          change_pct: 46.156
          ci95_low_pct: 44.3
          ci95_high_pct: 47.891
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 478535680.0
          candidate_median: 479617024.0
          change_pct: -0.014
          ci95_low_pct: -0.274
          ci95_high_pct: 0.386
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: No retained code; one-binary configuration sweep
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -1.302
    reason: "Eight workers improve wall only 1.30%, below the 3% bar while CPU rises 33.5%; 12 and 16 workers regress wall 2.46% and 10.65%, so the automatic policy remains the best complexity/resource tradeoff"
    commit: null
---
# Revisit worker depth on the live 1M workspace

## Hypothesis

H57: after breadth-first scheduling and macOS bulk enumeration changed the service-time
distribution, the automatic cold-worker depth might be too conservative on a one-million
entry heterogeneous tree.
A fixed 8, 12, or 16 workers should improve indexed wall time by at least 3% without a
disproportionate CPU or memory increase.

## Method

One immutable candidate binary ran automatic, 6-, 8-, 12-, and 16-worker configurations
in twelve interleaved pairs after three warmups.
Configuration was the only variable; all runs used the same 1,007,659-entry fingerprint
and exact oracle. No code was retained.

## Results

Automatic and six workers were indistinguishable: 6.1889 versus 6.1903 seconds.
Eight workers reached 6.1136 seconds, a statistically clear but only 1.30% wall
improvement [0.92%, 2.06%], while total CPU rose 33.5% and system CPU rose 46.2%. Twelve
workers regressed 2.46%; sixteen regressed 10.65%, more than doubled CPU, and raised RSS
28.66%.

The result also explains why simply copying the much higher concurrency limits in
`diskus` or `gdu` is not a promising fdu experiment on this machine: the metadata path
is already beyond its efficient parallelism point.

## Verdict

**REJECTED** — Eight workers improve wall only 1.30%, below the 3% bar while CPU rises
33.5%; 12 and 16 workers regress wall 2.46% and 10.65%, so the automatic policy remains
the best complexity/resource tradeoff

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
