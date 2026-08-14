---
title: Derive an exact rich summary without building an index
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-040
  title: Derive an exact rich summary without building an index
  date: 2026-08-13
  hypotheses:
    - H59
  subject:
    tree_label: live-workspace-exp040
    tree_root_id: 585f55000d4d135311f162954e1cc5fe3e0a729823acc02400e1c308d57a2949
    tree_engine_digest: 4e6152179266350aeef6833667edf3d7852ba8d879b319defadf7be644eeef4b
    tree_entries: 978339
    tree_directories: 113066
    tree_files: 864914
    tree_symlinks: 359
    tree_apparent_bytes: 24501149200
    tree_allocated_bytes: 26841804800
    tree_max_depth: 24
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
    control: cache-off summary after constructing the complete reusable index
    candidate: cache-off summary reduced directly from the scan through a derived execution plan
    control_binary:
      name: fdu-index-summary
      sha256: bc6c69c0ac777e9ea7653ece1931e79a433fca63556dca516e3f76288b5ff910
      size_bytes: 1299824
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
      name: fdu-transient-summary
      sha256: 45aab461bf35be6cf4caaad1ea0595293f7285875dbfec7bf4456dc1ef34c469
      size_bytes: 1299840
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
    run_artifact: benchmarks/results/realtree/run-exp040-transient-summary.json
  results:
    - job: rich-summary-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 4851955458.5
          candidate_median: 4183165104.5
          change_pct: -14.559
          ci95_low_pct: -18.55
          ci95_high_pct: -9.04
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 17735204000.0
          candidate_median: 16048655000.0
          change_pct: -8.646
          ci95_low_pct: -12.433
          ci95_high_pct: -7.31
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 1933615500.0
          candidate_median: 664689500.0
          change_pct: -66.267
          ci95_low_pct: -68.579
          ci95_high_pct: -65.758
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 15771880000.0
          candidate_median: 15367899500.0
          change_pct: -0.805
          ci95_low_pct: -4.145
          ci95_high_pct: 0.121
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 590569472.0
          candidate_median: 27623424.0
          change_pct: -95.278
          ci95_low_pct: -95.509
          ci95_high_pct: -95.115
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        minor_faults:
          control_median: 39186.5
          candidate_median: 1971.5
          change_pct: -94.956
          ci95_low_pct: -95.977
          ci95_high_pct: -94.45
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        involuntary_context_switches:
          control_median: 107190.0
          candidate_median: 92164.0
          change_pct: -13.014
          ci95_low_pct: -24.336
          ci95_high_pct: -3.879
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools:
    - name: dumac
      wall_ns_median: 3970320249.5
      argv:
        - "{binary}"
        - "{root}"
  complexity:
    lines_changed: 273
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - An incorrect requirement proof could select retained state too small for a report
    notes: The planner is internal to the existing CLI composition, has one compact tier, and falls closed to the complete index for cache participation, filters, multiple views, watch mode, or any unproved request
  verdict:
    decision: accepted
    primary_job: rich-summary-report
    primary_metric: wall_ns
    change_pct: -14.559
    reason: "The derived exact-summary plan improves paired wall 14.56% [9.04%, 18.55%], cuts peak RSS 95.28%, and produces one identical stable report hash across every old/new sample with no invalid trial or tree drift"
    commit: null
---
# Derive an Exact Rich Summary Without Building an Index

## Hypothesis

H59: when the cache cannot participate and the complete requested view set is exactly
one unfiltered summary, an execution planner can retain only that exact aggregate row.
It should remove index construction without changing the command, report schema,
provenance, error behavior, scope, or totals.

This is not a `--fast` mode and does not make output depth prune a scan.
The existing composition states the user’s requirements; the library derives the minimum
sufficient internal state and falls back to the full index unless it can prove the
smaller plan exact.

## Method

The candidate and the pre-change indexed-summary binary ran twelve adjacent interleaved
pairs after three warmups on a frozen 978,339-entry APFS fingerprint.
Both ran `--cache off --view summary --format json --color never`. The harness removed
only run-specific generator, root, and timestamp fields before hashing report semantics.
All 24 fdu samples produced the same semantic digest,
`e7e45d030a544253dca48f3a41aa912b0ae6f98f625b985731d34e6dfe858b48`.

The independent tree fingerprint found 44,630 duplicate hard-link entries representing
3,256,389,632 path-counted allocated bytes.
Both fdu variants retained the same path-accounting semantics.
Dumac ran beside the candidate only as a narrower total-only calibration reference; its
hard-link-deduplicated allocated total is not the correctness oracle for this
experiment.

## Results

The exact rich-summary plan improved paired wall time 14.56%, with the entire 95%
bootstrap interval between 9.04% and 18.55% faster.
User CPU fell 66.27%, while system CPU was statistically unchanged; this is the expected
signature of removing index construction without changing the filesystem walk.
Peak RSS fell from 563.2 MiB to 26.3 MiB, a 95.28% reduction, and minor faults fell
94.96%.

After the prototype-only public/Python entry point was removed without changing the CLI
execution path, the exact final CLI binary
(`9ab6a0a6616b8423e85bbb3e0e978741f3f5beac48cd182a217575af9e9be7d6`) ran on two
inactive, mutation-free APFS trees.
On 720,805 entries, it was 2.78% faster [0.39%, 6.85%], used 2.96× less user CPU, and
reduced median RSS from 322.3 MiB to 14.6 MiB (23.0×). On the self-contained
901,963-entry benchmark tree, it was 1.79% faster [−0.54%, 3.97%], used 3.11× less user
CPU, and reduced median RSS from 398.1 MiB to 13.5 MiB (29.6×). Both replications had
zero semantic mismatch, invalid sample, baseline drift, or tree mutation.

The wall effect is therefore topology-sensitive rather than a universal 14.56%. Uniform,
syscall-bound trees can hide nearly all of the eliminated user-space work; the CPU,
allocation-fault, and memory mechanism still reproduces decisively.

## Verdict

**ACCEPTED** — The derived exact-summary plan improves paired wall 14.56%
[9.04%, 18.55%], cuts peak RSS 95.28%, and produces one identical stable report hash
across every old/new sample with no invalid trial or tree drift.

Acceptance rests on the preregistered heterogeneous-tree wall result and decisive RSS
reduction, not on treating that wall percentage as typical across tree shapes.

The next experiments remove work still shared by both summary plans: worker-local
reduction (H62), a report-derived macOS attribute layout (H63), plan-specific worker
depth (H65), and a selected-total-only projection for the DUMAC-matched challenge (H64).
Exp-041/042 later rejected H62 and the H62 plus H63 composition; H65 consequently lost
its proposed reducer prerequisite, but was still screened directly over H59. Exp-043
retained automatic/six after eight workers failed to reproduce, leaving H64 as the clean
matched-workload follow-up.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
