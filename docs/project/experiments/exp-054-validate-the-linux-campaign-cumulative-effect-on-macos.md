---
title: Validate the Linux campaign's cumulative effect on macOS
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-054
  title: Validate the Linux campaign's cumulative effect on macOS
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
    control: main at 26280e4
    candidate: PR 22 after cross-platform review fixes
    control_binary:
      name: control
      sha256: 27c2e3994638217e2b61cb79e6cfa7d2a7e4f1d671badec12039cc09dcd3ddc5
      size_bytes: 1148240
      args: []
    candidate_binary:
      name: candidate
      sha256: 696e08acce8c26a50b6fd780fb57dc2108f6459d1336894624695addb0173785
      size_bytes: 1445664
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-pr22-current-vs-main-macos.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 298127167.0
          candidate_median: 306243333.5
          change_pct: 1.393
          ci95_low_pct: -0.186
          ci95_high_pct: 3.895
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 167720687.0
          candidate_median: 174783062.5
          change_pct: 1.707
          ci95_low_pct: -0.833
          ci95_high_pct: 5.837
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1152477000.0
          candidate_median: 1187892000.0
          change_pct: 0.435
          ci95_low_pct: -1.077
          ci95_high_pct: 5.249
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 230308500.0
          candidate_median: 213774000.0
          change_pct: -6.62
          ci95_low_pct: -9.266
          ci95_high_pct: -4.917
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 919217000.0
          candidate_median: 972184000.0
          change_pct: 3.227
          ci95_low_pct: -0.331
          ci95_high_pct: 8.56
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 37773312.0
          candidate_median: 37289984.0
          change_pct: -1.335
          ci95_low_pct: -2.721
          ci95_high_pct: 0.743
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
          control_median: 392990812.0
          candidate_median: 335746916.5
          change_pct: -15.682
          ci95_low_pct: -16.286
          ci95_high_pct: -13.993
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 168045833.0
          candidate_median: 168623125.0
          change_pct: -1.198
          ci95_low_pct: -3.852
          ci95_high_pct: 1.724
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 844314500.0
          candidate_median: 803420000.0
          change_pct: -6.132
          ci95_low_pct: -8.432
          ci95_high_pct: -0.836
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 271144000.0
          candidate_median: 217639500.0
          change_pct: -20.137
          ci95_low_pct: -21.288
          ci95_high_pct: -19.381
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 573071500.0
          candidate_median: 586073500.0
          change_pct: 0.626
          ci95_low_pct: -2.839
          ci95_high_pct: 8.589
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 35930112.0
          candidate_median: 34603008.0
          change_pct: -3.278
          ci95_low_pct: -4.047
          ci95_high_pct: -1.583
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools:
    - name: dust
      wall_ns_median: 246306562.5
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
    notes: Validation-only experiment over the cumulative PR; no additional implementation was made for this run.
  verdict:
    decision: accepted
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -15.682
    reason: "Warm revalidation improves 15.68% [13.99%, 16.29%], while cold scan moves +1.39% [-0.19%, +3.90%] and remains statistically neutral; keep the campaign but do not generalize its Linux cold-scan gain to macOS."
    commit: null
---
## Question

Does the cumulative Linux performance campaign still help on macOS, and which parts of
the result transfer across platforms?

The control is `main` at `26280e4`. The candidate is PR #22 after its cross-platform
review fixes. Both immutable release probes walked the same 60,993-entry APFS tree on an
Apple M1 Pro, with twelve paired and interleaved trials after three warmups.

## Result

Warm revalidation improved by 15.68% [13.99%, 16.29%] and its CPU total improved by
6.13% [0.84%, 8.43%]. That is a macOS result in its own right, though it is smaller than
the campaign’s Linux headline.

Cold scan wall time moved by +1.39% [-0.19%, +3.90%]. The interval includes no change,
so the Linux cold-scan gain did not reproduce on this tree and host.
It is neither a proven macOS gain nor a proven regression.

## Decision

Keep the cumulative campaign because its primary warm-revalidation result transfers to
macOS with a clear improvement.
Document cold scanning as platform-neutral here rather than generalizing Linux evidence.
The benchmark verified the tree before and after the run and its independent oracle
passed for both variants.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
