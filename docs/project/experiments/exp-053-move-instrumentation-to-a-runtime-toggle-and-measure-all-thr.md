---
title: Move instrumentation to a runtime toggle and measure all three of its costs
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-053
  title: Move instrumentation to a runtime toggle and measure all three of its costs
  date: "2026-08-14"
  hypotheses: []
  subject:
    tree_label: meta450k
    tree_root_id: 49fd7bea88ee0b90566f01fb140e86707e2dd6979dd01af97f30cdb13c535aee
    tree_engine_digest: 5b894066affe6de8c7f0ba44d00eccd9bc3b7e5d0cf146786f3ae860f4b0c3bc
    tree_entries: 450463
    tree_directories: 28630
    tree_files: 421690
    tree_symlinks: 143
    tree_apparent_bytes: 3000524491
    tree_allocated_bytes: 747966464
    tree_max_depth: 20
    tree_mutated_during_run: false
    host_cpu: Linux
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 0
    host_system: Linux 6.18.5-fc-v20
    filesystem: ""
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: probe with no instrumentation at all
    candidate: probe with fdu counters compiled in and recording off
    control_binary:
      name: control
      sha256: 96fa343e4a6f15474d2cfc3f0dba21dc9eea7d026d84d8f3a2c5050a6e5cdeb3
      size_bytes: 1444752
      args: []
    candidate_binary:
      name: candidate
      sha256: c1e3762c9519ed7ac5d725b0160243fb77f246ea7bfa3ed1591f6ddf7bc36374
      size_bytes: 1521808
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: /tmp/fdu-perf/results/run-counters-idle-cost.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1858796248.0
          candidate_median: 1847042262.5
          change_pct: -1.257
          ci95_low_pct: -2.961
          ci95_high_pct: 1.398
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        component_ns:
          control_median: 774119269.0
          candidate_median: 757068626.5
          change_pct: -3.451
          ci95_low_pct: -5.996
          ci95_high_pct: -0.042
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        cpu_ns:
          control_median: 2834067000.0
          candidate_median: 2815443000.0
          change_pct: -0.669
          ci95_low_pct: -2.728
          ci95_high_pct: 0.423
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        user_cpu_ns:
          control_median: 1880907500.0
          candidate_median: 1884906000.0
          change_pct: -0.514
          ci95_low_pct: -1.539
          ci95_high_pct: 0.911
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        system_cpu_ns:
          control_median: 953075000.0
          candidate_median: 942365000.0
          change_pct: -0.151
          ci95_low_pct: -5.83
          ci95_high_pct: 3.138
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        peak_rss_bytes:
          control_median: 285827072.0
          candidate_median: 276846592.0
          change_pct: -2.73
          ci95_low_pct: -4.485
          ci95_high_pct: 0.24
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
  reference_tools: []
  complexity:
    lines_changed: 690
    new_dependencies: []
    new_unsafe_blocks: 1
    new_failure_modes: []
    notes: "The unsafe impl GlobalAlloc is confined to fdu's counters/alloc.rs module. Its sink fields are private and custom construction is unsafe, so safe callers cannot violate the allocator's non-unwind and non-reentrancy contract."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -1.257
    reason: "Idle cost -1.26% [-2.96%, +1.40%] and recording cost +0.64% [-0.68%, +2.13%], both spanning zero, so counters can stay compiled in and be switched on per run instead of per build. Bounds recording below about 2.1% rather than establishing zero."
    commit: null
---
## What changed

`exp-052` put the counters behind a build feature.
That was the wrong switch.
A build flag means two binaries, a rebuild to see anything, and a standing risk that the
measured build and the shipped build differ in ways nobody tracks — friction in exactly
the loop that is supposed to be frictionless.

They are now always compiled in and toggled at runtime, with the reusable mechanism and
fdu-specific names kept together under `fdu::counters`. A separate crate would add a
publication boundary before any external consumer exists.

## Three costs, measured separately

A runtime toggle raises a question a build flag does not: what does the instrumentation
cost when it is *present but off*? That is the number that justifies leaving the code
in, and it is the one most easily forgotten.

| Question | Comparison | Result |
| --- | --- | --- |
| **Idle cost** | no instrumentation vs. compiled-in but off | −1.26% [−2.96%, +1.40%] |
| **Recording cost** | off vs. on, same binary | +0.64% [−0.68%, +2.13%] |
| **Build-flag era** | uninstrumented vs. feature-on (`exp-052`) | +0.03% [−3.31%, +3.76%] |

Every interval spans zero.
Recording ~13 million events across three layers, plus a counting global allocator on
every allocation, is not visible above the noise of the machine.

Stated precisely: this bounds recording below about 2.1% and the idle branch below about
1.4%. It does not establish that either is zero, and tightening it further would cost
more trials than the answer is worth.

## Why it is this cheap

The counters are thread-local and non-atomic, so the parallel walk pays no contention.
Per-event paths are counted rather than timed, because a clock read costs an order of
magnitude more than an integer add.
Allocation counting rides on an operation that already costs tens of nanoseconds.
And the enable check is a relaxed load of a `static` that the branch predictor gets
right every time.

## The three tiers, and one that does not do what it looks like

`fdu::counters::process` provides the process tier: Linux reads `/proc/self/io` and
`/proc/self/stat`, while macOS uses `proc_pidinfo`. Both are sampled once per phase, and
unsupported metrics remain absent rather than appearing as zero.

One finding is worth more than the code: **`syscr` is not a syscall count.** It counts
the read and write families only.
A walk over 17,128 directory entries — every one a `getdents64` or a `statx` — moved it
by **30**.

So there is no cheap in-process source for enumeration or stat syscall counts on Linux.
Application counters at the call site are the correct instrument for those, with
`strace -c` as the periodic ground truth.
Assuming the kernel tier covered them would have produced a confidently wrong
conclusion, and the table in the playbook exists to stop the next person making that
assumption.

## What is reusable

The reusable parts are thread-local storage with deterministic and destructor-backed
global folding, the runtime toggle, a generic `CountingAlloc` with certified
function-pointer sinks for a `const` initializer, and a shared process snapshot whose
platform capabilities are explicit.

The method is written up separately in
`docs/project/guides/performance-instrumentation-playbook.md`, including the failures
that produced each rule — which is the part that transfers.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
