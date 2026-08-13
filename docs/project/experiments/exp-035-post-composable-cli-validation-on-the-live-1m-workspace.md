---
title: Post-composable-CLI validation on the live 1M workspace
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-035
  title: Post-composable-CLI validation on the live 1M workspace
  date: 2026-08-12
  hypotheses:
    - H3
    - H31
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
    control: "origin/main dc56f77 after merged composable CLI PR #5"
    candidate: "PR #8 after merging origin/main and correctness review"
    control_binary:
      name: control
      sha256: 9d41606709d53bd13ea5311b4a33b796a43e39ba3f3c7fc50def2f80964091f3
      size_bytes: 552480
      args: []
    candidate_binary:
      name: candidate
      sha256: 3da6d0a6c284c6d89958204232a4647e5a9dced0c5316a4060439a2e23f2ff33
      size_bytes: 602192
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp035-live-workspace.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 9849708562.5
          candidate_median: 7332302000.0
          change_pct: -31.349
          ci95_low_pct: -39.13
          ci95_high_pct: -24.455
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 7195580999.5
          candidate_median: 4667378958.0
          change_pct: -42.381
          ci95_low_pct: -49.471
          ci95_high_pct: -33.734
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 34712099000.0
          candidate_median: 34847901000.0
          change_pct: -0.574
          ci95_low_pct: -13.774
          ci95_high_pct: 2.53
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 5219246500.0
          candidate_median: 4773014500.0
          change_pct: -8.609
          ci95_low_pct: -13.024
          ci95_high_pct: -7.117
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 29509146000.0
          candidate_median: 30064341500.0
          change_pct: 0.705
          ci95_low_pct: -14.015
          ci95_high_pct: 4.168
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 458907648.0
          candidate_median: 661749760.0
          change_pct: 44.32
          ci95_low_pct: 37.014
          ci95_high_pct: 47.714
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 16633770438.0
          candidate_median: 10578638104.0
          change_pct: -36.595
          ci95_low_pct: -37.8
          ci95_high_pct: -34.781
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 6915204062.5
          candidate_median: 4001618187.5
          change_pct: -43.488
          ci95_low_pct: -46.461
          ci95_high_pct: -42.108
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 65951389500.0
          candidate_median: 50554558500.0
          change_pct: -24.138
          ci95_low_pct: -25.172
          ci95_high_pct: -10.996
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 6506375500.0
          candidate_median: 5365462000.0
          change_pct: -17.948
          ci95_low_pct: -20.627
          ci95_high_pct: -15.574
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 59445014000.0
          candidate_median: 45458217500.0
          change_pct: -24.551
          ci95_low_pct: -25.894
          ci95_high_pct: -10.426
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 459726848.0
          candidate_median: 616194048.0
          change_pct: 34.049
          ci95_low_pct: 6.593
          ci95_high_pct: 36.609
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools:
    - name: dust
      wall_ns_median: 9148347395.5
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
    new_failure_modes: []
    notes: Live-tree scale-validation anchor only; prior experiment records own implementation complexity
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -31.349
    reason: "On the heterogeneous 1,007,659-entry workspace, candidate improves cold indexed wall 31.35% and producer wall 36.59% with valid exact digests and no tree mutation; the 44.32% indexed RSS increase remains a documented optimization target"
    commit: null
---
# Post-composable-CLI validation on the live 1M workspace

## Question

Does the rebased performance branch remain faster than merged `origin/main` on the real,
heterogeneous workspace the user actually experiences, once that workspace crosses one
million entries? This is a scale-validation anchor for H3 and H31.

## Method

The exact binaries from exp-033 ran twelve interleaved pairs after three warmups on the
live 1,007,659-entry APFS workspace.
The subject included source checkouts, build output, dependency trees, benchmark
corpora, and Git metadata to a maximum depth of 24. Both cold indexed construction and
producer-only scan were checked against the independent digest, count, and byte oracle.
The redacted pre/post fingerprints matched.

## Results

Cold indexed wall improved from 9.850 seconds to 7.332 seconds, a paired 31.35% win
[24.46%, 39.13%]; its component improved 42.38%. Producer wall improved from 16.634
seconds to 10.579 seconds, or 36.59% [34.78%, 37.80%], while its timed producer
component improved 43.49% and CPU fell 24.14%. Producer wall includes the harness’s
untimed validation scan, so the component is the cleaner scan-engine measure.

The gain has a substantial memory cost.
Indexed peak RSS rose from 437.6 MiB to 631.1 MiB (44.32%), and producer peak RSS rose
34.05%. This does not invalidate the speed result, but it moves compact full-index
storage to the front of the next experiment queue.

## Verdict

**ACCEPTED** — On the heterogeneous 1,007,659-entry workspace, candidate improves cold
indexed wall 31.35% and producer wall 36.59% with valid exact digests and no tree
mutation; the 44.32% indexed RSS increase remains a documented optimization target

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
