---
title: Region-scheduled breadth-first traversal
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-013
  title: Region-scheduled breadth-first traversal
  date: 2026-08-11
  hypotheses:
    - "H49: exp-012's RSS and CPU costs came from the global FIFO, not from preferring shallow work; per-region buckets with round-robin hand-off recover memory and locality while strengthening the shallow preference"
  subject:
    tree_label: metabrowser
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: c631fbf39d7c7adace225d5c9935aaf991176d05da800abd7a69c56ceb0f3b0e
    tree_entries: 60067
    tree_directories: 7350
    tree_files: 52695
    tree_symlinks: 22
    tree_apparent_bytes: 1085083672
    tree_allocated_bytes: 1230073856
    tree_max_depth: 19
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
    control: global FIFO breadth-first (bbc9cca)
    candidate: per-region LIFO buckets with a round-robin ready ring and worker affinity
    control_binary:
      name: control
      sha256: 9798917959662333159205a10d8587b74672f5c00e2376d0c2fdf10653d24192
      size_bytes: 535872
      args: []
    candidate_binary:
      name: candidate
      sha256: 887e041666a54e64deacd9b59f695fdc0975a9ae0a98b021737555c3f4e3c3a5
      size_bytes: 535872
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-region-scheduler.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 314412542.0
          candidate_median: 306709187.0
          change_pct: -1.914
          ci95_low_pct: -6.529
          ci95_high_pct: 9.968
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 198790625.0
          candidate_median: 192564166.5
          change_pct: -3.636
          ci95_low_pct: -9.809
          ci95_high_pct: 14.771
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1235635000.0
          candidate_median: 1174873000.0
          change_pct: -5.463
          ci95_low_pct: -9.511
          ci95_high_pct: 0.238
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 245193000.0
          candidate_median: 230405000.0
          change_pct: -5.323
          ci95_low_pct: -7.683
          ci95_high_pct: -0.869
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 989485500.0
          candidate_median: 941671500.0
          change_pct: -5.356
          ci95_low_pct: -10.48
          ci95_high_pct: 1.397
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
          control_median: 35135488.0
          candidate_median: 33767424.0
          change_pct: -3.889
          ci95_low_pct: -4.903
          ci95_high_pct: -3.148
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 667388854.0
          candidate_median: 661617000.0
          change_pct: -0.734
          ci95_low_pct: -1.458
          ci95_high_pct: 1.003
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 453096333.5
          candidate_median: 447414313.0
          change_pct: 0.413
          ci95_low_pct: -2.013
          ci95_high_pct: 1.912
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 657977000.0
          candidate_median: 648106000.0
          change_pct: -0.467
          ci95_low_pct: -1.777
          ci95_high_pct: 3.128
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 250962000.0
          candidate_median: 249795500.0
          change_pct: -0.938
          ci95_low_pct: -1.779
          ci95_high_pct: 0.83
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 407015000.0
          candidate_median: 397730500.0
          change_pct: -0.327
          ci95_low_pct: -2.096
          ci95_high_pct: 4.817
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 17181583.0
          candidate_median: 12153208.0
          change_pct: 1.068
          ci95_low_pct: -34.938
          ci95_high_pct: 38.446
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 32989184.0
          candidate_median: 32538624.0
          change_pct: -0.349
          ci95_low_pct: -1.95
          ci95_high_pct: 0.534
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools:
    - name: dust
      wall_ns_median: 229444270.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 286
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "One queue, two shapes: DepthFirst keeps the single stack, BreadthFirst uses per-region buckets plus a ready ring and an enqueued flag array. No barrier, no new dependency, both claim paths O(1)."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: peak_rss_bytes
    change_pct: -3.889
    reason: "Peak RSS -3.89% [-4.90%, -3.15%] on cold-scan-index, the only interval clear of zero, reversing exp-012's +1.51%. Wall unchanged (-1.91% [-6.53%, +9.97%]); no metric regressed. The deep-spur share of early work drops to 0-4% at every worker count against depth-first's 4-23%, so the orientation property now survives parallelism"
    commit: null
---
# Region-scheduled breadth-first traversal

## Hypothesis

H49: breadth-first's costs in exp-012 came from *how* it was implemented, not from
preferring shallow work. A single global FIFO makes the pending set hold an entire
level of the tree (the memory) and lets workers roam the full width with poor path
locality (the CPU), while still not spreading workers across distinct subtrees — the
one property a progressive consumer actually wants.

Predicted: bucketing work per top-level subtree and handing each free worker a
*different* bucket round-robin, LIFO within a bucket, recovers depth-first's memory
profile and locality while making the shallow preference stronger. Peak RSS should
fall; wall time should not move; a deep spur should stop crowding out shallow siblings.

## What was tried

`DirectoryQueueState` gains per-region buckets keyed by depth-1 ancestor, a round-robin
ready ring, and an `enqueued` flag array so a region appears in the ring at most once.
Each of the root's children seeds a region; every deeper directory inherits its
parent's, so membership costs one integer copy and never inspects a path. A worker
keeps affinity to its region while that region has work (O(1), no coordination) and
takes the next region off the ring when it runs dry (also O(1) — no scan under the
lock). `DepthFirst` keeps the single stack. There is no barrier anywhere: if only one
region has work, every worker takes it.

## What the numbers said

**Peak RSS fell measurably, which was the point.** On `cold-scan-index`, −3.89% with a
95% interval of [−4.90%, −3.15%] — the only metric whose interval clears zero, and more
than a reversal of the +1.51% [+0.85%, +2.88%] that exp-012 paid.

**Wall time did not move**, as predicted: −1.91% [−6.53%, +9.97%] cold, −0.73%
[−1.46%, +1.00%] warm. CPU trends down (−5.46% [−9.51%, +0.24%]) without clearing zero.
No metric regressed on either job. Warm revalidation is unchanged throughout because
its sweep does not use this queue.

**The orientation property now holds under parallelism.** On a skewed fixture — one
40-level spur holding most of the files beside eleven shallow siblings — the spur's
share of the first quarter of the walk:

| workers | region breadth-first | depth-first |
| ---: | ---: | ---: |
| 1 | 0% | 23% |
| 2 | 0% | 9% |
| 4 | 4% | 4% |
| 6 | 1% | 4% |

Breadth-first holds at 0–4% at every worker count. Depth-first's apparent convergence
at higher worker counts is accidental: extra workers dilute the spur because each one
happens to drill a different subtree, not because anything is preferring shallow work.

Two things the measurement corrected along the way. The first implementation resolved
the "allocate me a region" sentinel when choosing a bucket but never wrote it back into
the item, so children inherited the sentinel and every directory allocated its own
region — a scheduler degenerate into round-robin over the whole frontier. It passed
every correctness test, because per-entry results do not depend on scheduling; only the
invariant test written against the queue itself caught it. Second, the metric used in
exp-012 (`distinct top-level subtrees started at the halfway point`) turns out to be
saturated on the uniform fixture: each region holds 80 files and a quarter of the walk
is 520 files, so ~6.5 regions is an arithmetic ceiling and 7 was already optimal. It
also rewards a scheduler that *starts* many subtrees and finishes none, which is why
depth-first scored higher on it with more workers. The spur-share metric replaces it.

## Limitations

The reference tree is 60,067 entries on one host, and its top level is not wide enough
to stress the ready ring. A home folder with a million directories has a far larger
region count, and while the ring is O(1) per claim, the region table grows with the
number of top-level subtrees rather than with the tree.

Region granularity is fixed at depth 1. A tree whose entire content sits under a single
top-level directory collapses to one region, and the scheduler degenerates to LIFO
within it — correct and no worse than depth-first, but no orientation benefit either.
Adaptive granularity (deepen the region key when the top level is narrow) is the
obvious follow-up and is unmeasured.

Affinity is unbounded: a worker keeps a region until it runs dry. That is what
preserves locality, and it is also why the number of regions in flight is bounded by
the worker count.

## Verdict

**ACCEPTED.** Wall time is unchanged, peak RSS is measurably better, and the ordering
property that justified the breadth-first default now survives parallelism instead of
holding only in the single-worker case.
