---
title: Revisit the macOS bulk buffer on the live 1M workspace
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-039
  title: Revisit the macOS bulk buffer on the live 1M workspace
  date: "2026-08-13"
  hypotheses:
    - H55
  subject:
    tree_label: live-workspace-exp039
    tree_root_id: 585f55000d4d135311f162954e1cc5fe3e0a729823acc02400e1c308d57a2949
    tree_engine_digest: 71f21a5e1d0c377ec3d266243ef5590382a5fadb12c21983cbb4951be500d29a
    tree_entries: 1009679
    tree_directories: 113951
    tree_files: 895356
    tree_symlinks: 372
    tree_apparent_bytes: 28208810548
    tree_allocated_bytes: 30613979136
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
    trials: 6
    warmups: 2
    interleaved: true
    control: 64 KiB getattrlistbulk buffer
    candidate: 256 KiB getattrlistbulk buffer
    control_binary:
      name: buffer64
      sha256: 3da6d0a6c284c6d89958204232a4647e5a9dced0c5316a4060439a2e23f2ff33
      size_bytes: 602192
      args: []
    candidate_binary:
      name: buffer256
      sha256: 26899d199802937c30a655afd0499c7dd16eaa411da78c25862b05fbb33a7cf8
      size_bytes: 602192
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp039-buffer-revisit-screen.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 7821985604.5
          candidate_median: 8112605250.0
          change_pct: 2.217
          ci95_low_pct: -1.619
          ci95_high_pct: 8.191
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        component_ns:
          control_median: 5001057062.5
          candidate_median: 5189071208.5
          change_pct: 3.339
          ci95_low_pct: -1.179
          ci95_high_pct: 8.574
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        cpu_ns:
          control_median: 28563871000.0
          candidate_median: 28307348500.0
          change_pct: 0.083
          ci95_low_pct: -11.567
          ci95_high_pct: 5.476
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        user_cpu_ns:
          control_median: 4729699000.0
          candidate_median: 4740809000.0
          change_pct: -0.083
          ci95_low_pct: -1.313
          ci95_high_pct: 1.191
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        system_cpu_ns:
          control_median: 23781779000.0
          candidate_median: 23545782500.0
          change_pct: 0.036
          ci95_low_pct: -13.245
          ci95_high_pct: 6.405
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
        peak_rss_bytes:
          control_median: 665952256.0
          candidate_median: 704184320.0
          change_pct: 5.066
          ci95_low_pct: -2.245
          ci95_high_pct: 15.492
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 6
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: One constant changed for the screen and was reverted
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 2.217
    reason: "The 256 KiB screen changes indexed wall by +2.22% with an interval crossing zero and no preregistered mechanism win; it is reverted"
    commit: null
---
# Revisit the macOS bulk buffer on the live 1M workspace

## Hypothesis

H55: a 256 KiB `getattrlistbulk` buffer may amortize kernel crossings better than the
accepted 64 KiB buffer on a one-million-entry tree with very wide generated directories.
It should improve indexed wall time by at least 3% without raising RSS or page faults.

## Method

Only the macOS bulk-buffer constant changed.
The 64 KiB control and 256 KiB candidate ran a six-pair, two-warmup exploratory screen
on a 1,009,679-entry fingerprint.
This was a mechanism screen rather than a final acceptance run; a weak result would be
reverted without spending twelve pairs on a constant already tested at smaller scale.

## Results

The larger buffer changed paired indexed wall by +2.22%, with a wide interval from
-1.62% to +8.19%. The measured component regressed 3.34%; RSS rose 5.07% and minor
faults rose 5.36%, both statistically unclear.
No metric showed the preregistered substantial win, so the larger allocation does not
earn a full gate.

## Verdict

**REJECTED** — The 256 KiB screen changes indexed wall by +2.22% with an interval
crossing zero and no preregistered mechanism win; it is reverted

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
