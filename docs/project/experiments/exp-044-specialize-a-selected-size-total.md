---
title: Specialize a selected size total
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-044
  title: Specialize a selected size total
  date: 2026-08-13
  hypotheses:
    - H64
  subject:
    tree_label: h64-self-contained-901k
    tree_root_id: c95b1edda5762c399d4eaaf8494b1e1866f5554814d9db5c3fe353a5a13bc7a0
    tree_engine_digest: e7ed1ac6334eb80379d3a8b259188115462014f247c147b5560682cbb27d1fca
    tree_entries: 901963
    tree_directories: 110369
    tree_files: 791261
    tree_symlinks: 333
    tree_apparent_bytes: 16537459815
    tree_allocated_bytes: 18714202112
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
    trials: 16
    warmups: 3
    interleaved: true
    control: selected allocated total reduced through the existing generic H59 transient scan
    candidate: selected allocated total folded inside a strict requirement-derived macOS bulk reader
    control_binary:
      name: generic
      sha256: aae55521cececb29be6f11abf0edb5727f30c78ba6f38d5c879b6f44b37dac46
      size_bytes: 1299840
      args:
        - --cache
        - off
        - --view
        - total
        - --size
        - allocated
        - --format
        - json
        - --color
        - never
    candidate_binary:
      name: specialized
      sha256: 321deb4f5175213a78f20213dfd84ef2492186c9a8cb36a0c716f2075d2b4ebe
      size_bytes: 1316384
      args:
        - --cache
        - off
        - --view
        - total
        - --size
        - allocated
        - --format
        - json
        - --color
        - never
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: null
  results:
    - job: selected-allocated-total
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2980142145.5
          candidate_median: 2954663541.5
          change_pct: -1.147
          ci95_low_pct: -2.239
          ci95_high_pct: 0.436
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 16
        cpu_ns:
          control_median: 10641776000.0
          candidate_median: 10340346500.0
          change_pct: -2.704
          ci95_low_pct: -3.488
          ci95_high_pct: -1.704
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        user_cpu_ns:
          control_median: 485781000.0
          candidate_median: 234006000.0
          change_pct: -51.543
          ci95_low_pct: -52.745
          ci95_high_pct: -50.84
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        system_cpu_ns:
          control_median: 10163978000.0
          candidate_median: 10110289500.0
          change_pct: -0.397
          ci95_low_pct: -1.145
          ci95_high_pct: 0.786
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 16
        peak_rss_bytes:
          control_median: 14262272.0
          candidate_median: 8658944.0
          change_pct: -39.185
          ci95_low_pct: -39.761
          ci95_high_pct: -38.659
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        minor_faults:
          control_median: 1118.5
          candidate_median: 762.0
          change_pct: -31.921
          ci95_low_pct: -32.489
          ci95_high_pct: -31.478
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        involuntary_context_switches:
          control_median: 26328.0
          candidate_median: 24485.0
          change_pct: -7.412
          ci95_low_pct: -11.481
          ci95_high_pct: -5.298
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        voluntary_context_switches:
          control_median: 73486.5
          candidate_median: 73473.5
          change_pct: -0.013
          ci95_low_pct: -0.029
          ci95_high_pct: 0.009
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 16
  reference_tools:
    - name: dumac
      wall_ns_median: 2889612021.0
      argv:
        - "{binary}"
        - "{root}"
  complexity:
    lines_changed: 636
    new_dependencies: []
    new_unsafe_blocks: 1
    new_failure_modes:
      - A second macOS walker and parser could drift from the full reader's fallback and scope semantics
      - A new public total view would enlarge the CLI, report, Python, and test contracts
    notes: The prototype included a typed total view, exact portable fallback, strict macOS parser, worker-local scalar reduction, and in-buffer file folding; every production change was reverted after measurement
  verdict:
    decision: rejected
    primary_job: selected-allocated-total
    primary_metric: wall_ns
    change_pct: -1.147
    reason: "The complete specialization improved paired wall only 1.15% [-2.24%, +0.44%], did not beat dumac, and required a second unsafe parser plus a new public view; all prototype code was reverted"
    commit: null
---
# Specialize a Selected Size Total

## Hypothesis

H64: the existing transient summary still gathers five tallies and uses the generic
observation representation.
A requirement-derived total plan might match dumac’s selected-size workload closely
enough to beat it while preserving FDU’s exact path accounting, scope, partial-result,
symlink, and fallback semantics.

This was a deliberately isolated prototype.
It added a typed `total` report only on the experimental branch; rich `summary` and the
accepted indexed path remained unchanged.

## Method

The prototype first implemented the selected total through the generic H59 transient
scanner, establishing exact report and independent allocated-byte oracle parity.
The candidate then requested only name, type, flags, mount status, conditionally device,
and the selected size from `getattrlistbulk`. It folded regular-file sizes while parsing
each buffer and retained names only for directories that still had to be descended into.
A portable exact fallback and strict complete-directory fallback preserved the normal
scan contract.

Sixteen adjacent pairs after three warmups compared immutable generic and specialized
binaries on the self-contained 901,963-entry APFS tree.
All samples matched the independent total oracle; there were no invalid samples,
baseline drift, or tree mutation.
Separate screens retested 64 versus 128 KiB buffers, breadth-first versus depth-first
order, claim sizes 1/2/4/8, and 6/8/10/12 workers.
Any promising arm was confirmed on the independent 720,805-entry cache-pressure tree.

## Results

The complete narrow-reader and in-buffer-folding composition improved paired wall time
only 1.15%, with a 95% interval from 2.24% faster to 0.44% slower.
The representation did what it was designed to do: user CPU fell 51.54%, peak RSS
39.19%, minor faults 31.92%, and aggregate CPU 2.70%. System CPU was unchanged, so the
eliminated user-space work remained hidden beneath the directory-open and bulk-syscall
floor.

The specialized total’s 2.984-second median was statistically tied with dumac’s
2.890-second median in a separate sixteen-pair run: dumac was 2.63% faster, with an
interval from 6.72% faster to 0.18% slower.
It therefore did not meet the stated challenge.
The current richer H59 summary was independently measured at 2.957 seconds versus
dumac’s 2.915 seconds in an earlier twelve-pair calibration, reinforcing that a new
scalar surface buys little wall time on this topology.

No tuning arm rescued the design.
Eight workers looked 4.4% faster in a five-pair screen but regressed 2.0% in the
independent sixteen-pair confirmation.
A 128 KiB buffer was neutral, depth-first was 4.7% slower, claim size one was 9.0%
slower, and claim sizes two and eight were unclear.
The accepted 64 KiB buffer, breadth-first region scheduler, claim size four, and
six-worker policy remain the measured operating point.

## Verdict

**REJECTED** — A 1.15% unclear wall improvement does not justify 636 changed production
lines, a second unsafe macOS parser, or a new public CLI/report/Python surface.
The prototype is fully reverted.
H59 remains the smallest useful execution tier: it already delivers the richer exact
summary near the same APFS syscall floor and with much lower CPU and memory than dumac.

The result narrows the next frontier.
Representation-only summary specializations H62, H63, and H64 all cut user-space
resources without materially changing elapsed time.
Further macOS live-scan gains now require reducing directory-open or kernel work itself,
or changing the amount of filesystem that must be visited through journal scoping.
Full-index layout work remains valuable for the tree product’s memory footprint even
when it cannot move this scalar syscall floor.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
