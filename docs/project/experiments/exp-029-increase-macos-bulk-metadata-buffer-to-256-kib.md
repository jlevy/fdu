---
title: Increase macOS bulk metadata buffer to 256 KiB
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-029
  title: Increase macOS bulk metadata buffer to 256 KiB
  date: "2026-08-12"
  hypotheses:
    - H55
  subject:
    tree_label: metabrowser-20260812
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: ce5a7430e152412a519ee9f9776c2fec73e59c58fa553aa3e9c2f8c085d26619
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
    control: exp-026 64 KiB macOS bulk metadata buffer
    candidate: 256 KiB macOS bulk metadata buffer
    control_binary:
      name: control
      sha256: 35198f0525f9501b71bd6764362f35723c925a3689b99c587bfbc457da896019
      size_bytes: 569104
      args: []
    candidate_binary:
      name: candidate
      sha256: 3c5b27d3113b38530845cf78ea317f80cc0c65eafdc341b4c29ab72a5d9e4f1c
      size_bytes: 569104
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp029-larger-bulk-buffer-small-final.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 311045271.0
          candidate_median: 321517041.5
          change_pct: -1.798
          ci95_low_pct: -5.948
          ci95_high_pct: 5.451
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 190788500.0
          candidate_median: 193992437.5
          change_pct: -3.044
          ci95_low_pct: -6.902
          ci95_high_pct: 9.606
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1061677000.0
          candidate_median: 1112058500.0
          change_pct: 1.149
          ci95_low_pct: -4.426
          ci95_high_pct: 8.815
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 229394000.0
          candidate_median: 233439000.0
          change_pct: 3.654
          ci95_low_pct: -5.069
          ci95_high_pct: 7.679
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 832544000.0
          candidate_median: 883737000.0
          change_pct: 3.358
          ci95_low_pct: -7.164
          ci95_high_pct: 10.716
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
          control_median: 35274752.0
          candidate_median: 35864576.0
          change_pct: 2.379
          ci95_low_pct: 0.253
          ci95_high_pct: 3.107
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
          control_median: 448108500.0
          candidate_median: 458014896.0
          change_pct: 1.78
          ci95_low_pct: -0.651
          ci95_high_pct: 7.224
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 180352833.5
          candidate_median: 181084250.0
          change_pct: 3.722
          ci95_low_pct: -2.41
          ci95_high_pct: 6.795
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1715876000.0
          candidate_median: 1717012000.0
          change_pct: -2.174
          ci95_low_pct: -4.053
          ci95_high_pct: 3.346
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 258096500.0
          candidate_median: 264674500.0
          change_pct: 1.508
          ci95_low_pct: -2.578
          ci95_high_pct: 6.696
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 1456314500.0
          candidate_median: 1459449000.0
          change_pct: -3.093
          ci95_low_pct: -5.148
          ci95_high_pct: 2.692
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
          control_median: 35512320.0
          candidate_median: 36560896.0
          change_pct: 2.587
          ci95_low_pct: 0.656
          ci95_high_pct: 5.038
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 510575229.0
          candidate_median: 510113000.0
          change_pct: -0.011
          ci95_low_pct: -0.858
          ci95_high_pct: 1.084
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 309279270.5
          candidate_median: 308254896.0
          change_pct: -0.297
          ci95_low_pct: -0.908
          ci95_high_pct: 1.631
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 506934500.0
          candidate_median: 506800000.0
          change_pct: 0.056
          ci95_low_pct: -0.396
          ci95_high_pct: 0.953
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 226047500.0
          candidate_median: 227520500.0
          change_pct: 0.712
          ci95_low_pct: -0.331
          ci95_high_pct: 1.437
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 281736000.0
          candidate_median: 279950500.0
          change_pct: -0.263
          ci95_low_pct: -0.968
          ci95_high_pct: 1.416
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 4231583.0
          candidate_median: 4306646.5
          change_pct: 8.691
          ci95_low_pct: -11.704
          ci95_high_pct: 26.175
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 32587776.0
          candidate_median: 32759808.0
          change_pct: 0.377
          ci95_low_pct: 0.05
          ci95_high_pct: 1.519
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 1
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - each active reader reserves an additional 192 KiB of syscall buffer capacity
    notes: one constant change; no dependency or unsafe change; the preregistered 720k gate was not triggered
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -1.798
    reason: "Cold-index wall was -1.80% with an interval spanning -5.95% to +5.45%; producer and warm paths were neutral, system CPU did not corroborate a gain, and cold RSS plus faults regressed"
    commit: null
---
# Increase macOS bulk metadata buffer to 256 KiB

## Hypothesis

H55 tested whether the 64 KiB `getattrlistbulk` buffer retained after exp-026 was
forcing repeat syscalls in wide directories.
Increasing it to 256 KiB should have reduced cold scan wall/component time and system
CPU by at least 3%, with warm reconciliation as a possible compositional benefit.
The cost was an additional 192 KiB per reader, or about 1.1 MiB across the normal six
cold workers.

## What was tried

The macOS bulk reader’s fixed syscall buffer changed from 64 KiB to 256 KiB. No parser,
fallback, scheduling, index, or reconciliation behavior changed.
The exact exp-026 binary and the one-line candidate ran twelve interleaved pairs after
three warmups for cold index, producer-only, and full warm revalidation on a freshly
fingerprinted, immutable 60,067-entry APFS subject.

The pre-registered gate required a 3% cold wall or component improvement accompanied by
lower system CPU before spending a 720k-entry confirmation run.
RSS was recorded explicitly because the candidate reserved four times as much syscall
capacity for each reader.

## What the numbers said

Cold-index wall was -1.80% [-5.95%, +5.45%], component -3.04%, and system CPU +3.36%;
all intervals included zero.
Producer wall was +1.78% [-0.65%, +7.22%], component +3.72%, and system CPU -3.09%;
those intervals also included zero.
Warm wall was -0.01% [-0.86%, +1.08%] and its component -0.30%.

The predicted mechanism was absent.
Cold-index RSS and minor faults regressed 2.38% and 1.92%; producer RSS and faults
regressed 2.59% and 2.25%. Warm RSS and faults also rose slightly.
Every sample passed the oracle and the tree remained unchanged.

The subject averages only about eight entries per directory.
A 64 KiB buffer already holds nearly every directory in one call, so quadrupling it
spends resident memory without removing enough bulk calls to measure.

## Verdict

**Rejected and reverted.** No path cleared the 3% gate, system CPU did not corroborate a
cold improvement, and memory counters moved in the predicted adverse direction.
The 720k run was not triggered.
The 64 KiB buffer remains the measured operating point; future syscall work should
reduce directory opens or improve a platform backend rather than enlarge each reader’s
buffer.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
