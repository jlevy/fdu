---
title: Validate review fixes on macOS
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-055
  title: Validate review fixes on macOS
  date: "2026-08-14"
  hypotheses: []
  subject:
    tree_label: pr22-macos-benchmarks
    tree_root_id: c95b1edda5762c399d4eaaf8494b1e1866f5554814d9db5c3fe353a5a13bc7a0
    tree_engine_digest: f708694be70261d65046e934ef03aed21a52bfed19fe456a11e18f9305b62ca4
    tree_entries: 60993
    tree_directories: 7466
    tree_files: 53502
    tree_symlinks: 25
    tree_apparent_bytes: 1111236927
    tree_allocated_bytes: 1258037248
    tree_max_depth: 22
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
    control: reviewed PR head fb99812
    candidate: PR 22 with R1-R8 addressed
    control_binary:
      name: control
      sha256: b9a91242418133deca7891c0e419d3b8f88c7b7b18a555aa4cc2e3e7aee8b200
      size_bytes: 1247520
      args: []
    candidate_binary:
      name: candidate
      sha256: 696e08acce8c26a50b6fd780fb57dc2108f6459d1336894624695addb0173785
      size_bytes: 1445664
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-pr22-current-vs-reviewed-head-macos.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 304895375.5
          candidate_median: 297538750.0
          change_pct: -0.949
          ci95_low_pct: -3.074
          ci95_high_pct: 0.976
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 173008063.0
          candidate_median: 165386187.5
          change_pct: -1.868
          ci95_low_pct: -5.908
          ci95_high_pct: 0.809
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1176212500.0
          candidate_median: 1138855000.0
          change_pct: -2.312
          ci95_low_pct: -4.678
          ci95_high_pct: 0.829
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 218718500.0
          candidate_median: 216994000.0
          change_pct: 0.39
          ci95_low_pct: -2.934
          ci95_high_pct: 1.26
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 957861500.0
          candidate_median: 924103000.0
          change_pct: -2.468
          ci95_low_pct: -6.111
          ci95_high_pct: 0.481
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 36978688.0
          candidate_median: 37199872.0
          change_pct: 1.15
          ci95_low_pct: -0.133
          ci95_high_pct: 2.191
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 344680396.0
          candidate_median: 340199062.5
          change_pct: -1.598
          ci95_low_pct: -4.778
          ci95_high_pct: 2.678
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 176497458.0
          candidate_median: 171485187.5
          change_pct: -3.433
          ci95_low_pct: -7.932
          ci95_high_pct: 6.276
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 827309000.0
          candidate_median: 813088000.0
          change_pct: -1.674
          ci95_low_pct: -8.626
          ci95_high_pct: 4.272
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 219312000.0
          candidate_median: 218044000.0
          change_pct: -0.91
          ci95_low_pct: -1.999
          ci95_high_pct: 0.959
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 607940000.0
          candidate_median: 594629500.0
          change_pct: -1.908
          ci95_low_pct: -10.741
          ci95_high_pct: 5.406
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 34390016.0
          candidate_median: 34594816.0
          change_pct: 0.285
          ci95_low_pct: 0.071
          ci95_high_pct: 0.788
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools:
    - name: dust
      wall_ns_median: 241065645.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 2796
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: Cross-platform correctness and soundness review rewrite; no new dependency and no net new unsafe implementation boundary.
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -0.949
    reason: "Cold wall time moves -0.95% [-3.07%, +0.98%] and warm wall time moves -1.60% [-4.78%, +2.68%]; both span zero, so no macOS wall-time regression is detected from the correctness and safety fixes."
    commit: null
---
## Question

Did addressing the PR review’s packaging, allocator-safety, worker-folding, runtime, and
platform-capability findings create a measurable macOS performance regression?

The control is the originally reviewed PR head, `fb99812`. The candidate contains all
R1–R8 fixes. Both immutable release probes walked the same 60,993-entry APFS tree on an
Apple M1 Pro, with twelve paired and interleaved trials after three warmups.

## Result

Cold scan wall time moved by -0.95% [-3.07%, +0.98%], and warm revalidation moved by
-1.60% [-4.78%, +2.68%]. Both intervals include no change.
The run therefore detects no wall-time regression from the review fixes; it bounds the
upper end at about 1.0% cold and 2.7% warm on this host rather than establishing zero
cost.

Peak RSS and minor faults moved slightly upward on the warm job, but the absolute
medians changed by only about 0.2 MiB and fourteen faults.
Those secondary signals do not outweigh neutral wall and CPU intervals for correctness
and soundness fixes.

## Decision

Keep the fixes. They close release-blocking correctness and safety gaps without a
detected macOS wall-time regression.
The benchmark verified the tree before and after the run and its independent oracle
passed for both variants.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
