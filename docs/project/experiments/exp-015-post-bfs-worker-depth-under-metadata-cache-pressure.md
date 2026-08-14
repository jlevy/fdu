---
title: Post-BFS worker depth under metadata-cache pressure
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-015
  title: Post-BFS worker depth under metadata-cache pressure
  date: "2026-08-12"
  hypotheses:
    - H31
  subject:
    tree_label: cache-pressure-12x
    tree_root_id: ffd40fd8482e8ed64bd19bcd1a724389532ca4889be43adf830122279ac63180
    tree_engine_digest: f2909250591b9b64d98956b0b2d8a9c3bd588b4c23f046a4660f3f174173dc23
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
    control: "six workers, the current automatic ceiling"
    candidate: sixteen workers on a tree larger than the vnode cache
    control_binary:
      name: t6
      sha256: be3349ee5238da00b5bce9ff7f72e68fd3fc0a9f96eae16c969c520f0e90977f
      size_bytes: 535968
      args:
        - "--threads"
        - "6"
    candidate_binary:
      name: t16
      sha256: be3349ee5238da00b5bce9ff7f72e68fd3fc0a9f96eae16c969c520f0e90977f
      size_bytes: 535968
      args:
        - "--threads"
        - "16"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp015-thread-curve-large.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 7238608542.0
          candidate_median: 6231851459.0
          change_pct: -11.723
          ci95_low_pct: -16.829
          ci95_high_pct: -2.417
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 5795667791.0
          candidate_median: 4846901750.0
          change_pct: -16.04
          ci95_low_pct: -20.281
          ci95_high_pct: -5.376
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 22555565500.0
          candidate_median: 32602266500.0
          change_pct: 42.638
          ci95_low_pct: 28.822
          ci95_high_pct: 52.295
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 3079160000.0
          candidate_median: 3353422500.0
          change_pct: 7.795
          ci95_low_pct: 5.969
          ci95_high_pct: 12.623
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 19523935000.0
          candidate_median: 29249942000.0
          change_pct: 47.673
          ci95_low_pct: 31.728
          ci95_high_pct: 59.688
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
          control_median: 325435392.0
          candidate_median: 330432512.0
          change_pct: 1.555
          ci95_low_pct: 1.41
          ci95_high_pct: 1.952
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
          control_median: 12811854146.0
          candidate_median: 11685440646.0
          change_pct: -9.272
          ci95_low_pct: -13.812
          ci95_high_pct: -5.485
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 5618959583.5
          candidate_median: 5057234687.5
          change_pct: -10.539
          ci95_low_pct: -16.778
          ci95_high_pct: -2.727
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 45126188000.0
          candidate_median: 60332157500.0
          change_pct: 34.634
          ci95_low_pct: 29.117
          ci95_high_pct: 41.364
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 4008363500.0
          candidate_median: 4510106500.0
          change_pct: 12.048
          ci95_low_pct: 10.412
          ci95_high_pct: 14.34
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 41241223000.0
          candidate_median: 55747978500.0
          change_pct: 37.381
          ci95_low_pct: 30.453
          ci95_high_pct: 44.407
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
          control_median: 325574656.0
          candidate_median: 331472896.0
          change_pct: 1.684
          ci95_low_pct: 1.611
          ci95_high_pct: 2.094
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: Configuration experiment only; establishes the large-tree target and the need for a scale-sensitive trigger.
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -11.723
    reason: "Sixteen workers improved large-tree end-to-end wall 11.72% and producer wall 9.27%, while the separate 60k run showed why the default must adapt rather than rise globally"
    commit: null
---
# Post-BFS worker depth under metadata-cache pressure

## Hypothesis

H31 revisited the six-worker automatic ceiling after region-scheduled breadth-first
traversal landed. That ceiling came from a fully warm 60k-entry tree, where the single
index consumer becomes the limit.
A tree larger than the metadata cache should instead spend more time waiting for
filesystem metadata, leaving room for a deeper producer pool to reduce wall time even
though it consumes more CPU.

## What was tried

The same release binary ran with explicit `--threads 6` and `--threads 16`; no code
changed between variants.
The immutable subject was built from twelve APFS clones of the pinned 60k reference tree
under independent top-level names.
It has 720,805 entries, well above this host’s 263,168-vnode ceiling, while clone
sharing limits additional physical storage.
Twelve measured pairs followed three warmups, alternating variant order at every
ordinal.

A second claim-grade run repeated the comparison on the original 60,067-entry tree.
Keeping the two scales separate preserves each run’s single immutable subject while
testing both sides of the proposed policy boundary.

## What the numbers said

Under cache pressure, sixteen workers improved cold-index wall by 11.72%
[−16.83%, −2.42%] and its measured scan component by 16.04% [−20.28%, −5.38%]. Producer
wall improved 9.27% [−13.81%, −5.49%]. Queue coordination remained negligible, so the
result survives the breadth-first scheduler change that motivated this revisit.

The speed is a latency trade, not free work elimination.
Cold-index CPU regressed 42.64% [+28.82%, +52.30%], mostly in system time, and peak RSS
rose 1.56%. On the small reference tree, sixteen workers regressed cold-index wall 5.64%
[+2.08%, +7.96%], CPU 42.99%, and peak RSS 14.83%; producer wall there was unclear at
−1.17% [−3.90%, +0.65%]. Raising the default globally would therefore make the common
warm-small case slower and substantially less efficient.

## Verdict

**Accepted as the large-tree target, not as a global default.** An automatic scan should
begin at six workers and unlock up to sixteen only after observed scale establishes that
the tree is not the warm-small case.
That implementation needs its own experiment.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
