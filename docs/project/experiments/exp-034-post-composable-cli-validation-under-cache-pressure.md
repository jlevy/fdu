---
title: Post-composable-CLI validation under cache pressure
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-034
  title: Post-composable-CLI validation under cache pressure
  date: 2026-08-12
  hypotheses:
    - H3
    - H31
    - H53
    - H12
    - H9
  subject:
    tree_label: post-cli-cache-pressure-12x
    tree_root_id: aa81f45748e048288dde3ceb302680753b75633a5bad09643d4fd1195aeae5ab
    tree_engine_digest: 5afd8022acef9f5c6547f8aba90d307a6fa07cc996cc2c84eca9a76db0ff7483
    tree_entries: 720805
    tree_directories: 88201
    tree_files: 632340
    tree_symlinks: 264
    tree_apparent_bytes: 13021004064
    tree_allocated_bytes: 14760886272
    tree_max_depth: 20
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
    run_artifact: benchmarks/results/realtree/run-exp034-post-cli-cache-pressure.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6699913416.5
          candidate_median: 5153956395.5
          change_pct: -30.458
          ci95_low_pct: -37.711
          ci95_high_pct: -18.198
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 5141065417.0
          candidate_median: 3517541604.0
          change_pct: -38.116
          ci95_low_pct: -49.363
          ci95_high_pct: -24.139
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 25915983000.0
          candidate_median: 24018713500.0
          change_pct: -10.39
          ci95_low_pct: -49.03
          ci95_high_pct: 10.209
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 3343781500.0
          candidate_median: 2910896000.0
          change_pct: -14.158
          ci95_low_pct: -17.851
          ci95_high_pct: -11.02
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 22601502000.0
          candidate_median: 21042838000.0
          change_pct: -9.759
          ci95_low_pct: -53.665
          ci95_high_pct: 13.765
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
          control_median: 331677696.0
          candidate_median: 437788672.0
          change_pct: 32.265
          ci95_low_pct: 0.155
          ci95_high_pct: 44.49
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 20531968583.5
          candidate_median: 6056398229.0
          change_pct: -70.916
          ci95_low_pct: -71.858
          ci95_high_pct: -69.565
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 17623227479.0
          candidate_median: 3191772750.0
          change_pct: -82.078
          ci95_low_pct: -83.675
          ci95_high_pct: -80.95
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 13500367500.0
          candidate_median: 10120036000.0
          change_pct: -28.678
          ci95_low_pct: -29.592
          ci95_high_pct: -21.758
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 3537981000.0
          candidate_median: 3448318500.0
          change_pct: -3.045
          ci95_low_pct: -3.886
          ci95_high_pct: -2.036
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 9966974500.0
          candidate_median: 6673107000.0
          change_pct: -37.638
          ci95_low_pct: -39.11
          ci95_high_pct: -28.247
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 6892119583.5
          candidate_median: 0.0
          change_pct: -100.0
          ci95_low_pct: -100.0
          ci95_high_pct: -100.0
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 363773952.0
          candidate_median: 359964672.0
          change_pct: -1.03
          ci95_low_pct: -1.206
          ci95_high_pct: -0.966
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools:
    - name: dust
      wall_ns_median: 5439152250.5
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
    notes: Scale-validation anchor only; exp-015 through exp-030 own the implementation complexity and individual decisions
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -30.458
    reason: "On the 720,805-entry pressure tree, candidate improves cold index 30.46% and warm revalidation 70.92%; all paired intervals exclude zero, all samples pass the independent oracle, and the tree stayed unchanged"
    commit: null
---
# Post-composable-CLI validation under cache pressure

## Question

Do the post-CLI integration gains survive at a size large enough to pressure metadata
caches and expose scaling costs?
This reproduces the accepted H3, H31, H53, H12, and H9 stack without changing code.

## Method

The same immutable binaries as exp-033 ran twelve interleaved pairs after three warmups
on a generated 720,805-entry APFS tree.
Cold indexed construction and compatible-snapshot warm revalidation were checked against
the independent exact oracle on every sample.
The tree remained unchanged throughout the run.

## Results

Cold indexed wall improved from 6.700 seconds to 5.154 seconds, a paired 30.46% win
[18.20%, 37.71%]; the measured scan/index component improved 38.12%. Peak RSS rose from
316.3 MiB to 417.5 MiB, a 32.27% cost that is significant operationally even though the
wall-time hypothesis succeeds.

Warm revalidation improved from 20.532 seconds to 6.056 seconds, or 70.92%
[69.57%, 71.86%]. Its reconciliation component improved 82.08%, total CPU fell 28.68%,
and peak RSS fell 1.03%. The larger tree therefore strengthens, rather than reverses,
the merged branch’s elapsed-time result while identifying full-index memory as the next
large cost.

## Verdict

**ACCEPTED** — On the 720,805-entry pressure tree, candidate improves cold index 30.46%
and warm revalidation 70.92%; all paired intervals exclude zero, all samples pass the
independent oracle, and the tree stayed unchanged

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
