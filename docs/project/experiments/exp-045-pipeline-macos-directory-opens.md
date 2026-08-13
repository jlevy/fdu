---
title: Pipeline macOS directory opens
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-045
  title: Pipeline macOS directory opens
  date: 2026-08-13
  hypotheses:
    - H67
    - H69
  subject:
    tree_label: h69-self-contained-901k
    tree_root_id: c95b1edda5762c399d4eaaf8494b1e1866f5554814d9db5c3fe353a5a13bc7a0
    tree_engine_digest: 26b604b60a15209483bba5e89b41b0dd4493aaf3b7d28104fdb2c088d8b3fdd6
    tree_entries: 901963
    tree_directories: 110369
    tree_files: 791261
    tree_symlinks: 333
    tree_apparent_bytes: 16537501222
    tree_allocated_bytes: 18714251264
    tree_max_depth: 23
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
    trials: 5
    warmups: 2
    interleaved: true
    control: current exact rich-summary path with its automatic six-worker operating point
    candidate: six scan and parser workers plus two bounded directory-open helpers
    control_binary:
      name: control
      sha256: dc0bb7ccbb29ff32b270e91abd9baca980fe572cc456bc04b65c0f37ff37bf60
      size_bytes: 1299856
      args:
        - --cache
        - off
        - --view
        - summary
        - --format
        - json
        - --color
        - never
    candidate_binary:
      name: open-pipeline-6p2
      sha256: 4dcc6f8750bace9cd42e27a5983ef0ed2fdde059237e900b44292ca9b7310a81
      size_bytes: 1332928
      args:
        - --cache
        - off
        - --view
        - summary
        - --format
        - json
        - --color
        - never
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: null
  results:
    - job: rich-summary-open-pipeline
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3468263916.0
          candidate_median: 3325422334.0
          change_pct: -4.465
          ci95_low_pct: -31.042
          ci95_high_pct: 33.914
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
        cpu_ns:
          control_median: 12623310000.0
          candidate_median: 12437884000.0
          change_pct: 1.469
          ci95_low_pct: -26.378
          ci95_high_pct: 6.758
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
        user_cpu_ns:
          control_median: 563935000.0
          candidate_median: 635275000.0
          change_pct: 8.62
          ci95_low_pct: -8.652
          ci95_high_pct: 15.675
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
        system_cpu_ns:
          control_median: 12059375000.0
          candidate_median: 11812351000.0
          change_pct: 1.212
          ci95_low_pct: -27.075
          ci95_high_pct: 6.326
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
        peak_rss_bytes:
          control_median: 14303232.0
          candidate_median: 14712832.0
          change_pct: 2.864
          ci95_low_pct: -21.626
          ci95_high_pct: 5.164
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
        minor_faults:
          control_median: 1158.0
          candidate_median: 1152.0
          change_pct: 0.345
          ci95_low_pct: -19.276
          ci95_high_pct: 3.097
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
        involuntary_context_switches:
          control_median: 63283.0
          candidate_median: 81623.0
          change_pct: 26.41
          ci95_low_pct: -20.385
          ci95_high_pct: 67.59
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
        voluntary_context_switches:
          control_median: 73705.0
          candidate_median: 73724.0
          change_pct: -0.011
          ci95_low_pct: -0.117
          ci95_high_pct: 0.137
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
  reference_tools:
    - name: dumac
      wall_ns_median: 2985920208.0
      argv:
        - "{binary}"
        - "{root}"
  complexity:
    lines_changed: 78
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - Open helpers can accidentally compose with the adaptive reserve and over-thread the scan
      - Bounded request and response channels add shutdown and panic paths
      - A static opener count can overfit one topology or host-load regime
    notes: The first prototype activated the existing reserve and ran 18 threads; the measured candidate fixed the experiment at six scan workers plus two open-only helpers, and no production code is retained while confirmation remains open.
  verdict:
    decision: superseded
    primary_job: rich-summary-open-pipeline
    primary_metric: wall_ns
    change_pct: -4.465
    reason: "The corrected pairwise-helper screen had a promising -4.47% point estimate but an unusable [-31.04%, +33.91%] interval; H70 supersedes it with one shared bounded opener pool, and no H69 code is retained"
    commit: null
---
# Pipeline macOS Directory Opens

## Profile Result

The exact current FDU and dumac binaries first ran in twelve adjacent pairs on the
901,963-entry tree. In this busy interactive-host regime, FDU took a 3.595-second median
and dumac took 2.986 seconds, a clear 16.19% paired gap [12.23%, 19.10%]. Dumac spent
34.88% more aggregate CPU and 231.70% more peak RSS. Its wall advantage therefore came
from greater concurrency, not less machine work: FDU sustained 3.46 aggregate
core-equivalents and dumac sustained 5.64.

Replaying the exact binaries from the published comparison on the same current host also
produced an 11.1% five-pair dumac lead.
That rules out the intervening FDU reconciliation-only code change as the cause.
It also prevents replacing the published quiet-host result with this run: the current
machine had high and variable background load, and control samples later ranged from
3.29 to 5.20 seconds.
The durable conclusion is that the FDU–dumac wall relationship is sensitive to available
concurrency and host pressure.

Two-second `sample` profiles localized both programs to the same boundary.
FDU had six active workers; 96.10% of their sampled top frames were `open` (39.56%) or
`getattrlistbulk` (56.54%). Dumac had ten active workers; 94.21% were in the same calls
(`open` 50.88%, `getattrlistbulk` 43.32%). Both main threads waited for workers.
These are within-process stack-residency shares, not syscall-duration estimates, and raw
sample counts are not comparable across the two processes.
They are nevertheless enough to exclude report reduction, index construction, and the
observation consumer as the current wall boundary.

`fs_usage` could not collect call-level durations without elevated privileges, and the
installed Xcode `File Activity` recorder crashed before starting.
Source and tree oracles still establish the required call shape: each implementation
opens every one of the 110,369 directories and issues one or more synchronous bulk calls
for it.
macOS has no vector form that accepts several directories, so independent threads
are the only available way to put several of these operations in flight.

## Bounded Opener Experiment

Static full-worker depth was already a poor answer.
Exp-043 and the selected-total screens in exp-044 found the same curve again: eight
workers can look 4–5% faster on this tree, but the independent 720,805-entry run was
neutral or slower, while CPU and context switches rose sharply at deeper pools.

H69 therefore separated syscall overlap from parsing.
Two helper threads open the next claimed directories while the accepted six workers
parse and enumerate current directories.
At most eight directory syscalls can be outstanding; paths, region order, bulk parsing,
complete-directory fallback, and report reduction remain unchanged.

The first implementation exposed an interaction the experiment had to fix.
Waiting for prefetched descriptors raised the existing service-time calibration above
its threshold, which activated ten reserve workers.
A profile found eighteen active threads rather than the intended six plus two.
That run is not verdict evidence.
The corrected experimental binary fixed the parser pool at six, and a second profile
confirmed exactly eight active threads.

The corrected five-pair screen preserved one stable summary digest, matched every
independent tally, and observed no tree mutation.
Its paired wall point estimate was 4.47% faster, but the 95% interval spanned −31.04% to
+33.91% under severe host noise.
CPU and memory were statistically unclear; the involuntary-context-switch point estimate
rose 26.41%.

## Verdict

**SUPERSEDED** — the mechanism is distinct and the point estimate clears the screen, but
the evidence does not.
No pipeline code is in the production branch.
H70 replaces pairwise helper ownership with one shared bounded opener pool and owns the
quiet-host and independent-topology confirmation.
Any retained design must keep the normal adaptive reserve from stacking with opener
helpers.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
