---
title: Single-pass checksum and parse on snapshot load
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-009
  title: Single-pass checksum and parse on snapshot load
  date: 2026-08-11
  hypotheses:
    - H32
  subject:
    tree_label: metabrowser-clone
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: bf574331eca680372f7060d4f9ab3b3b175afd265ac27bda6b6dc67ed9c80798
    tree_entries: 59654
    tree_directories: 7341
    tree_files: 52291
    tree_symlinks: 22
    tree_apparent_bytes: 1082046346
    tree_allocated_bytes: 1225879552
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
    trials: 14
    warmups: 3
    interleaved: true
    control: "exp-008 build: CRC pass over the whole image, seek to zero, second pass to parse"
    candidate: CRC folds over bytes as the parser consumes them; the verdict still gates the returned index
    control_binary:
      name: h14h18
      sha256: fd188164cb635a257654f7cbb5d72d6faeec70fd53b9661f4da523db8c0ff448
      size_bytes: 519328
      args: []
    candidate_binary:
      name: h14h18h32
      sha256: eb25a8f293b17f8de9f481c2ca17790f7fd086196375e97d74eb9f4848ef8d2d
      size_bytes: 519328
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp007-009-portable-stack.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 462634895.5
          candidate_median: 453369395.5
          change_pct: -7.073
          ci95_low_pct: -10.505
          ci95_high_pct: 27.918
          significant: false
          pairs: 14
        component_ns:
          control_median: 282551875.0
          candidate_median: 277134417.0
          change_pct: 6.231
          ci95_low_pct: -6.184
          ci95_high_pct: 29.794
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 1078041500.0
          candidate_median: 1062271000.0
          change_pct: -5.585
          ci95_low_pct: -7.963
          ci95_high_pct: -0.937
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 248176000.0
          candidate_median: 249426500.0
          change_pct: -0.325
          ci95_low_pct: -2.43
          ci95_high_pct: 1.16
          significant: false
          pairs: 14
        system_cpu_ns:
          control_median: 826702500.0
          candidate_median: 812248500.0
          change_pct: -7.204
          ci95_low_pct: -10.7
          ci95_high_pct: -0.968
          significant: true
          pairs: 14
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 35086336.0
          candidate_median: 34775040.0
          change_pct: 0.456
          ci95_low_pct: -2.278
          ci95_high_pct: 4.327
          significant: false
          pairs: 14
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 536973042.0
          candidate_median: 517971708.5
          change_pct: -2.726
          ci95_low_pct: -5.857
          ci95_high_pct: 0.576
          significant: false
          pairs: 14
        component_ns:
          control_median: 219996750.0
          candidate_median: 211333458.5
          change_pct: -0.238
          ci95_low_pct: -6.824
          ci95_high_pct: 3.697
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 2119421500.0
          candidate_median: 2078589500.0
          change_pct: 0.581
          ci95_low_pct: -5.899
          ci95_high_pct: 2.98
          significant: false
          pairs: 14
        user_cpu_ns:
          control_median: 317423000.0
          candidate_median: 317536000.0
          change_pct: -0.203
          ci95_low_pct: -1.927
          ci95_high_pct: 1.006
          significant: false
          pairs: 14
        system_cpu_ns:
          control_median: 1801933000.0
          candidate_median: 1757793000.0
          change_pct: 0.786
          ci95_low_pct: -6.903
          ci95_high_pct: 3.663
          significant: false
          pairs: 14
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          pairs: 0
        peak_rss_bytes:
          control_median: 34324480.0
          candidate_median: 34889728.0
          change_pct: 1.086
          ci95_low_pct: -0.473
          ci95_high_pct: 2.2
          significant: false
          pairs: 14
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 952780791.5
          candidate_median: 921808813.0
          change_pct: -3.523
          ci95_low_pct: -13.672
          ci95_high_pct: 4.941
          significant: false
          pairs: 14
        component_ns:
          control_median: 643461791.5
          candidate_median: 599599625.0
          change_pct: -3.18
          ci95_low_pct: -11.343
          ci95_high_pct: 2.901
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 771031000.0
          candidate_median: 747834500.0
          change_pct: -3.035
          ci95_low_pct: -5.391
          ci95_high_pct: -1.404
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 274157000.0
          candidate_median: 266748500.0
          change_pct: -2.588
          ci95_low_pct: -3.986
          ci95_high_pct: -1.637
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 495445000.0
          candidate_median: 479551000.0
          change_pct: -3.48
          ci95_low_pct: -5.516
          ci95_high_pct: -1.194
          significant: true
          pairs: 14
        blocked_ns:
          control_median: 184705791.5
          candidate_median: 171900187.5
          change_pct: -32.281
          ci95_low_pct: -44.338
          ci95_high_pct: 31.838
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 32563200.0
          candidate_median: 32030720.0
          change_pct: -0.18
          ci95_low_pct: -1.775
          ci95_high_pct: 1.208
          significant: false
          pairs: 14
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 278913250.0
          candidate_median: 235387146.0
          change_pct: -4.182
          ci95_low_pct: -18.556
          ci95_high_pct: 1.072
          significant: false
          pairs: 14
        component_ns:
          control_median: 123834167.0
          candidate_median: 110835521.0
          change_pct: -5.377
          ci95_low_pct: -12.035
          ci95_high_pct: 8.031
          significant: false
          pairs: 14
        cpu_ns:
          control_median: 227014500.0
          candidate_median: 219499500.0
          change_pct: -3.793
          ci95_low_pct: -4.257
          ci95_high_pct: -2.371
          significant: true
          pairs: 14
        user_cpu_ns:
          control_median: 214189000.0
          candidate_median: 207207500.0
          change_pct: -3.13
          ci95_low_pct: -3.62
          ci95_high_pct: -2.594
          significant: true
          pairs: 14
        system_cpu_ns:
          control_median: 13955500.0
          candidate_median: 12171500.0
          change_pct: -12.035
          ci95_low_pct: -17.081
          ci95_high_pct: -3.018
          significant: true
          pairs: 14
        blocked_ns:
          control_median: 47098750.0
          candidate_median: 17620271.0
          change_pct: -20.84
          ci95_low_pct: -68.887
          ci95_high_pct: 20.939
          significant: false
          pairs: 14
        peak_rss_bytes:
          control_median: 31236096.0
          candidate_median: 31031296.0
          change_pct: -0.393
          ci95_low_pct: -1.434
          ci95_high_pct: 0.558
          significant: false
          pairs: 14
  reference_tools:
    - name: dust
      wall_ns_median: 220640562.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 60
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Fail-closed unchanged: the index is returned only after the digest over the complete payload matches; structural corruption is still caught by the parser's own checks. What moves is one full read of the image."
  verdict:
    decision: in-progress
    primary_job: warm-snapshot-load
    primary_metric: wall_ns
    change_pct: -4.182
    reason: "Underpowered, not refuted: -4.18% median with interval [-18.56%, +1.07%] under load average 17; the change is held as an uncommitted patch pending the focused re-measurement on a quiet machine"
    commit: null
---
# Single-pass checksum and parse on snapshot load

## Hypothesis

H32: _state what you expected to be slow, why,
and which metric would move._

## What was tried

_The smallest change that tests the hypothesis._

## What the numbers said

_Read the tables in the frontmatter. Say what surprised you._

## Verdict

**IN-PROGRESS** — Underpowered, not refuted: -4.18% median with interval [-18.56%, +1.07%] under load average 17; the change is held as an uncommitted patch pending the focused re-measurement on a quiet machine
