---
title: Tune a shared macOS directory-opener pool
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-046
  title: Tune a shared macOS directory-opener pool
  date: "2026-08-13"
  hypotheses:
    - H70
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
    candidate: six scan and parser workers drawing opened directories from one shared two-thread opener pool
    control_binary:
      name: control
      sha256: dc0bb7ccbb29ff32b270e91abd9baca980fe572cc456bc04b65c0f37ff37bf60
      size_bytes: 1299856
      args:
        - --cache
        - "off"
        - --view
        - summary
        - --format
        - json
        - --color
        - never
    candidate_binary:
      name: open-shared-6p2
      sha256: 110243ffae9b5c6d897eab070b10f3b6f1fba5231c54bd65cda165ed81a920b3
      size_bytes: 1332944
      args:
        - --cache
        - "off"
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
    - job: rich-summary-shared-openers
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3337896833.0
          candidate_median: 3220922875.0
          change_pct: -3.98
          ci95_low_pct: -9.871
          ci95_high_pct: -0.71
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 5
        cpu_ns:
          control_median: 11757665000.0
          candidate_median: 10203650000.0
          change_pct: -15.98
          ci95_low_pct: -19.695
          ci95_high_pct: -10.377
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 5
        user_cpu_ns:
          control_median: 527397000.0
          candidate_median: 636754000.0
          change_pct: 23.307
          ci95_low_pct: 17.314
          ci95_high_pct: 25.316
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 5
        system_cpu_ns:
          control_median: 11230268000.0
          candidate_median: 9551484000.0
          change_pct: -17.619
          ci95_low_pct: -21.388
          ci95_high_pct: -11.945
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 5
        peak_rss_bytes:
          control_median: 14041088.0
          candidate_median: 14270464.0
          change_pct: 2.214
          ci95_low_pct: -0.577
          ci95_high_pct: 3.823
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 5
  reference_tools:
    - name: dumac
      wall_ns_median: 3050421458.5
      argv:
        - "{binary}"
        - "{root}"
  complexity:
    lines_changed: 121
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - A shared opener pool adds request ordering and shutdown paths
      - Every descriptor handoff adds scheduler and channel traffic
      - Opener and adaptive scan-worker policies can exceed one intended concurrency budget
    notes: The primary five-pair screen used two shared openers; separate two, three, and four-opener screens plus a direct twelve-pair four-opener comparison with dumac were diagnostic only, and all prototype code remains outside the production branch.
  verdict:
    decision: in-progress
    primary_job: rich-summary-shared-openers
    primary_metric: wall_ns
    change_pct: -3.98
    reason: "Two shared openers cleared the short screen at -3.98% [-9.87%, -0.70%], but doubled involuntary context switches and later count/dumac runs suffered extreme host outliers; quiet 12-pair and independent-topology confirmation remain required"
    commit: null
---
# Tune a Shared macOS Directory-Opener Pool

## Hypothesis

H70 asks whether one small opener pool can keep synchronous directory opens in flight
without deepening the complete scan-worker pool.
This changes the overlap boundary rather than the metadata contract: the accepted six
workers still parse bulk records and reduce the exact five-field summary.

## Primary Screen

Five adjacent pairs after two warmups compared immutable binaries on the frozen
901,963-entry APFS tree.
All samples had one semantic digest, matched every independent tally, and observed no
tree mutation or baseline drift.
Two shared openers improved paired wall 3.98% [0.70%, 9.87%] and aggregate CPU 15.98%.
The mechanism traded system time for transport work: system CPU fell 17.62%, user CPU
rose 23.31%, and involuntary context switches rose 111.80%.

Profiles explain that trade.
The two opener threads spent about 77% of sampled tops in `open`, while the six scanners
still spent about 37% waiting for descriptor responses.
The pool exposed genuine overlap, but descriptor delivery became part of the boundary.

## Count Sweep and Dumac Calibration

A separate five-pair sweep tested two, three, and four shared openers.
Two was unclear at −1.13% [−6.00%, +5.02%]; three had a −5.13% point estimate but
crossed zero [−6.92%, +1.13%]. Four reported −12.90% [−67.29%, −5.81%], but two control
samples were extreme 8.36- and 9.31-second outliers while the candidate remained near 3
seconds. That interval measures host interference, not a trustworthy four-opener effect.
CPU and involuntary context switches also rose 3.42% and 217.15% in that arm.

A direct twelve-pair four-opener comparison with dumac produced 3.156- and 3.050-second
medians, respectively, with a paired dumac change of −0.09% [−7.83%, +37.54%]. Both
tools suffered large outliers, so wall time was unresolved.
Dumac used 40.68% more aggregate CPU and 223.23% more peak RSS than the experimental FDU
binary; equivalently, FDU used 28.92% less CPU and 69.06% less memory than dumac.

## Current Verdict

**IN PROGRESS** — the shared pool is more promising than pairwise opener ownership, but
the primary screen is short and the follow-up host state is unusable for acceptance.
No opener-pipeline code is in the production branch.
Resume with twelve quiet adjacent pairs, one preselected count, and an independent large
topology. Retain it only if the normal 3% wall gate clears without disproportionate
context-switch cost and the opener plus scan-worker total remains one explicit budget.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
