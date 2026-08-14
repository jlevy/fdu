---
title: Open macOS directories relative to one retained root fd
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-024
  title: Open macOS directories relative to one retained root fd
  date: "2026-08-12"
  hypotheses:
    - H2
    - H24
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
    control: exp-022 macOS bulk reader with absolute directory opens
    candidate: one retained root fd per worker and root-relative openat
    control_binary:
      name: control
      sha256: 52e0b303402ac0eafa11b06013b731126d81bef482acc962cca3ad9fa2ebc879
      size_bytes: 552576
      args: []
    candidate_binary:
      name: candidate
      sha256: f854174c15d483c2478e35293d994a4d89d4e9990970f913a301328d10459310
      size_bytes: 552624
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp024-root-relative-openat-large-final.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3685746250.0
          candidate_median: 3625590354.0
          change_pct: -0.075
          ci95_low_pct: -4.059
          ci95_high_pct: 1.534
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 2335379416.5
          candidate_median: 2251456187.5
          change_pct: -0.087
          ci95_low_pct: -6.88
          ci95_high_pct: 1.055
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 11396792000.0
          candidate_median: 10831639000.0
          change_pct: -0.418
          ci95_low_pct: -2.788
          ci95_high_pct: 1.069
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 2420344000.0
          candidate_median: 2415922000.0
          change_pct: 1.373
          ci95_low_pct: -0.685
          ci95_high_pct: 1.992
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 8938723000.0
          candidate_median: 8428123000.0
          change_pct: -1.033
          ci95_low_pct: -3.754
          ci95_high_pct: 0.847
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
          control_median: 326230016.0
          candidate_median: 327401472.0
          change_pct: 0.301
          ci95_low_pct: 0.175
          ci95_high_pct: 0.439
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
          control_median: 6539804687.5
          candidate_median: 6211882916.5
          change_pct: -6.348
          ci95_low_pct: -13.059
          ci95_high_pct: -1.979
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 2639808562.0
          candidate_median: 2453246229.0
          change_pct: -6.157
          ci95_low_pct: -17.569
          ci95_high_pct: -1.563
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 20293426000.0
          candidate_median: 20081197000.0
          change_pct: 0.657
          ci95_low_pct: -6.388
          ci95_high_pct: 4.236
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 2962561500.0
          candidate_median: 2942869000.0
          change_pct: -1.184
          ci95_low_pct: -3.81
          ci95_high_pct: 1.844
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 17472416500.0
          candidate_median: 17228587500.0
          change_pct: 1.441
          ci95_low_pct: -6.783
          ci95_high_pct: 4.916
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
          control_median: 327139328.0
          candidate_median: 327262208.0
          change_pct: 0.045
          ci95_low_pct: -0.115
          ci95_high_pct: 0.223
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 69
    new_dependencies: []
    new_unsafe_blocks: 1
    new_failure_modes:
      - "root open, path conversion, or openat failure sends the directory through the portable reread path"
      - one additional root directory descriptor remains live per active worker
    notes: "macOS only; 50 insertions and 19 deletions, one retained fd per worker, one CString conversion per directory, and a second unsafe block"
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -0.075
    reason: "Indexed wall was neutral at -0.07% [-4.06%, +1.53%] and the pre-registered system-CPU signal was also neutral; a producer-only wall win did not justify the extra descriptor, conversion, and unsafe boundary"
    commit: null
---
# Open macOS directories relative to one retained root fd

## Hypothesis

H2/H24 predicted that retaining the scan root as a directory descriptor and opening
every claimed directory with `openat` would avoid repeated absolute-path prefix
resolution. The post-exp-022 profile attributed 33.86% of cold self time to `open`, so
the pre-registered signal was lower system CPU, with the largest wall-time effect on the
deep 720,805-entry cache-pressure tree.

## What was tried

Each macOS scan worker retained one `File` for the walk root.
The bulk metadata reader used that descriptor directly for the root and
`openat(root_fd, relative_path, ...)` for all descendants.
Failure to open the root, encode a relative path without an interior NUL, or open a
descendant caused the existing portable directory reread.
The change added one audited unsafe block for the `openat` call and transfer of its
returned owned descriptor into `File`; other platforms were unchanged.

This is deliberately the smallest H24 implementation.
It retains one bounded descriptor per worker, rather than keeping parent descriptors
with a breadth-first frontier or adding the H29 ancestor-descriptor cache.

## What the numbers said

The full gate used twelve interleaved pairs on the immutable 720,805-entry APFS subject.
End-to-end cold-index wall was neutral at -0.07% with a 95% interval of
[-4.06%, +1.53%]. Its component, total CPU, and system CPU intervals all included zero.
Peak RSS regressed 0.30% and minor faults 0.28%, although both absolute changes were
small.

Producer-only wall improved 6.35% [-13.06%, -1.98%], and its scan component improved
6.16% [-17.57%, -1.56%]. That result did not carry the predicted mechanism evidence:
producer system CPU was +1.44% [-6.78%, +4.92%]. An earlier six-pair run on the same
large subject had the opposite split--cold-index improved 5.47% while producer wall was
neutral--which is further evidence that the wall-only differences were sensitive to host
load. A six-pair exploratory run on the 60,067-entry subject also left cold-index,
component, CPU, and system CPU neutral; its producer wall result was not corroborated by
component or CPU.

## Verdict

**Rejected.** The user-visible indexed scan missed the default wall-time gate and the
pre-registered system-CPU signal did not move.
A producer-only wall result without that mechanism evidence does not justify extra path
conversion, a retained descriptor per worker, or a second unsafe block.
Root-relative `openat` is therefore a measured dead end for the current breadth-first
bulk walker. H24’s more elaborate parent- or ancestor-dirfd variants remain distinct
hypotheses, but they must account for descriptor bounds and BFS lifetime before
implementation.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
