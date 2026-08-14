---
title: Reject inline basic content analysis
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-047
  title: Reject inline basic content analysis
  date: "2026-08-13"
  hypotheses:
    - H79
  subject:
    tree_label: selfhost-content
    tree_root_id: 0d8bca813ccc20705265ad42baad61b86c412e7927cf3fd4b8703be5e93c1f57
    tree_engine_digest: 98360347c76f3db629e4f96dd15f450e66f529b1c34de19138d6b222a392518e
    tree_entries: 307
    tree_directories: 74
    tree_files: 233
    tree_symlinks: 0
    tree_apparent_bytes: 3175738
    tree_allocated_bytes: 3760128
    tree_max_depth: 8
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
    control: automatic bounded worker pool
    candidate: inline analysis at or below 512 files and 8 MiB
    control_binary:
      name: control
      sha256: 61e3053f92157bd3c44ab36e7eca0cd4f75f1dc6f3ac56d7a5164af766c7d2cf
      size_bytes: 850624
      args: []
    candidate_binary:
      name: candidate
      sha256: 79085c0c4cadc44f161391bff5b61e6e144de1407a287c758b752327026fd729
      size_bytes: 850656
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: null
  results:
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 16684667.0
          candidate_median: 27814979.0
          change_pct: 66.338
          ci95_low_pct: 61.505
          ci95_high_pct: 80.483
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        component_ns:
          control_median: 9117437.5
          candidate_median: 17623187.0
          change_pct: 92.927
          ci95_low_pct: 79.027
          ci95_high_pct: 109.279
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        cpu_ns:
          control_median: 85860500.0
          candidate_median: 33111500.0
          change_pct: -61.524
          ci95_low_pct: -65.641
          ci95_high_pct: -56.767
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 17359000.0
          candidate_median: 15045500.0
          change_pct: -13.068
          ci95_low_pct: -14.475
          ci95_high_pct: -11.223
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 68210500.0
          candidate_median: 18030000.0
          change_pct: -73.792
          ci95_low_pct: -76.849
          ci95_high_pct: -69.868
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
          control_median: 5472256.0
          candidate_median: 8544256.0
          change_pct: 56.437
          ci95_low_pct: 55.456
          ci95_high_pct: 58.144
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6358062.5
          candidate_median: 9700916.5
          change_pct: 50.294
          ci95_low_pct: 46.269
          ci95_high_pct: 60.107
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        component_ns:
          control_median: 1983917.0
          candidate_median: 2003583.5
          change_pct: -0.044
          ci95_low_pct: -2.852
          ci95_high_pct: 6.72
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 4894000.0
          candidate_median: 7185500.0
          change_pct: 44.333
          ci95_low_pct: 40.909
          ci95_high_pct: 51.775
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 3358000.0
          candidate_median: 4752000.0
          change_pct: 40.119
          ci95_low_pct: 37.156
          ci95_high_pct: 46.979
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 1544000.0
          candidate_median: 2462500.0
          change_pct: 56.669
          ci95_low_pct: 47.805
          ci95_high_pct: 64.625
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 1442145.5
          candidate_median: 2480229.5
          change_pct: 69.877
          ci95_low_pct: 62.573
          ci95_high_pct: 83.336
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 3448832.0
          candidate_median: 7856128.0
          change_pct: 125.89
          ci95_low_pct: 125.117
          ci95_high_pct: 129.904
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: content-disabled
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 7040333.5
          candidate_median: 10415000.5
          change_pct: 44.341
          ci95_low_pct: 37.393
          ci95_high_pct: 56.851
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        component_ns:
          control_median: 1792.0
          candidate_median: 1708.5
          change_pct: -4.509
          ci95_low_pct: -14.171
          ci95_high_pct: 15.085
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 13225000.0
          candidate_median: 15315500.0
          change_pct: 14.648
          ci95_low_pct: 12.285
          ci95_high_pct: 19.469
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 2386000.0
          candidate_median: 3706000.0
          change_pct: 53.152
          ci95_low_pct: 47.584
          ci95_high_pct: 58.412
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 10767000.0
          candidate_median: 11664000.0
          change_pct: 6.395
          ci95_low_pct: 3.226
          ci95_high_pct: 9.525
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
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
          control_median: 2932736.0
          candidate_median: 7462912.0
          change_pct: 155.154
          ci95_low_pct: 153.338
          ci95_high_pct: 156.023
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: content-query
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 72634812.5
          candidate_median: 84530396.0
          change_pct: 15.758
          ci95_low_pct: 12.827
          ci95_high_pct: 17.728
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        component_ns:
          control_median: 56620937.0
          candidate_median: 56189937.5
          change_pct: -1.565
          ci95_low_pct: -2.783
          ci95_high_pct: 1.234
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 143431000.0
          candidate_median: 89669000.0
          change_pct: -37.448
          ci95_low_pct: -41.467
          ci95_high_pct: -33.535
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 73827000.0
          candidate_median: 71044000.0
          change_pct: -4.05
          ci95_low_pct: -5.053
          ci95_high_pct: -2.241
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 69390500.0
          candidate_median: 18566500.0
          change_pct: -72.8
          ci95_low_pct: -76.724
          ci95_high_pct: -69.962
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
          control_median: 5832704.0
          candidate_median: 9043968.0
          change_pct: 53.931
          ci95_low_pct: 52.764
          ci95_high_pct: 57.61
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 54
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - serial file reads on small repositories
    notes: One bounded dispatch branch and one unit test; production change reverted
  verdict:
    decision: rejected
    primary_job: content-basic
    primary_metric: wall_ns
    change_pct: 66.338
    reason: "Inline self-host analysis regressed wall 66.34% and component 92.93%; the worker pool was doing useful parallel I/O, so the change is reverted"
    commit: null
---
# Reject inline basic content analysis

## Hypothesis

H79: the fixed cost of starting a scoped worker pool may dominate basic content analysis
on a small repository.
Auto-selected workloads of at most 512 files and 8 MiB should therefore run inline,
improving both end-to-end wall time and the measured analysis component by at least 3%
without changing results.

## What was tried

The candidate added one bounded dispatch branch before the existing worker pool.
An explicit single worker still selected serial execution, explicit parallel settings
still selected the pool, and larger automatic workloads remained parallel.
The two release probes ran 12 interleaved pairs after three warmups on an immutable
archive of this repository: 233 files, 74 directories, and 3.18 MB of apparent data.

Every valid basic-analysis sample emitted the same content digest in both variants:
`3af57a513812aeeb72be9b1df58f33572fa8e84790a65f71d255ba78946a6a16`.

## What the numbers said

Inline analysis regressed median end-to-end wall time from 16.68 ms to 27.81 ms, or
66.34%, with the paired interval wholly above zero.
The measured content component regressed from 9.12 ms to 17.62 ms, or 92.93%. Serial
execution reduced aggregate CPU, but that trade would make the interactive operation
nearly twice as slow and increased measured peak RSS. The worker frames in the profile
represented useful concurrent file I/O rather than removable startup overhead.

The additional cache-hit, query, and disabled-boundary jobs were diagnostic only.
Their content-cache and query components were neutral; the disabled boundary itself
remained approximately 2 microseconds.
Whole-process differences in those jobs reflect binary layout and startup effects and do
not rescue the primary regression.

## Verdict

**REJECTED** — Inline self-host analysis regressed wall 66.34% and component 92.93%; the
worker pool was doing useful parallel I/O, so the production change was reverted.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
