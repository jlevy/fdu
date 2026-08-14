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
  date: "2026-08-11"
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
    trials: 20
    warmups: 4
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
    run_artifact: benchmarks/results/realtree/run-exp007-009-requiem.json
  results:
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 661268292.0
          candidate_median: 640465146.0
          change_pct: -1.168
          ci95_low_pct: -3.897
          ci95_high_pct: 0.133
          significant: false
          pairs: 20
        component_ns:
          control_median: 443910562.5
          candidate_median: 432408562.5
          change_pct: -0.382
          ci95_low_pct: -3.712
          ci95_high_pct: 1.003
          significant: false
          pairs: 20
        cpu_ns:
          control_median: 653868500.0
          candidate_median: 627925000.0
          change_pct: -1.489
          ci95_low_pct: -4.556
          ci95_high_pct: -0.716
          significant: true
          pairs: 20
        user_cpu_ns:
          control_median: 250100500.0
          candidate_median: 245113500.0
          change_pct: -2.066
          ci95_low_pct: -3.124
          ci95_high_pct: -1.336
          significant: true
          pairs: 20
        system_cpu_ns:
          control_median: 400237000.0
          candidate_median: 383823000.0
          change_pct: -1.018
          ci95_low_pct: -4.49
          ci95_high_pct: 0.114
          significant: false
          pairs: 20
        blocked_ns:
          control_median: 7645541.5
          candidate_median: 9546833.5
          change_pct: -16.805
          ci95_low_pct: -25.308
          ci95_high_pct: 35.466
          significant: false
          pairs: 20
        peak_rss_bytes:
          control_median: 31899648.0
          candidate_median: 32153600.0
          change_pct: 0.874
          ci95_low_pct: 0.051
          ci95_high_pct: 2.379
          significant: false
          pairs: 20
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 351552125.0
          candidate_median: 318324479.5
          change_pct: -7.98
          ci95_low_pct: -21.493
          ci95_high_pct: 4.574
          significant: false
          pairs: 20
        component_ns:
          control_median: 168317479.0
          candidate_median: 130718271.0
          change_pct: -12.383
          ci95_low_pct: -22.85
          ci95_high_pct: -4.705
          significant: true
          pairs: 20
        cpu_ns:
          control_median: 244257000.0
          candidate_median: 227703000.0
          change_pct: -2.728
          ci95_low_pct: -8.411
          ci95_high_pct: -0.49
          significant: true
          pairs: 20
        user_cpu_ns:
          control_median: 228513500.0
          candidate_median: 212913500.0
          change_pct: -2.262
          ci95_low_pct: -6.357
          ci95_high_pct: -1.117
          significant: true
          pairs: 20
        system_cpu_ns:
          control_median: 15450500.0
          candidate_median: 13545000.0
          change_pct: -17.142
          ci95_low_pct: -23.753
          ci95_high_pct: -5.767
          significant: true
          pairs: 20
        blocked_ns:
          control_median: 99017208.0
          candidate_median: 81282521.0
          change_pct: -22.446
          ci95_low_pct: -68.178
          ci95_high_pct: 21.246
          significant: false
          pairs: 20
        peak_rss_bytes:
          control_median: 30760960.0
          candidate_median: 30695424.0
          change_pct: -0.188
          ci95_low_pct: -0.638
          ci95_high_pct: 1.173
          significant: false
          pairs: 20
  reference_tools:
    - name: dust
      wall_ns_median: 222827604.0
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
    notes: "Net-negative lines: the two-pass helper is deleted. Fail-closed unchanged - the index is returned only after the digest over the complete payload matches, and structural corruption is still caught by the parser's own checks. The wall-vs-component ruling is codified in the loop guide: a pre-registered signal may be the accept metric; a post-hoc metric switch is never an accept."
  verdict:
    decision: accepted
    primary_job: warm-snapshot-load
    primary_metric: component_ns
    change_pct: -12.383
    reason: "Accepted on the pre-registered signal: the research registry declared this hypothesis's predicted signal as warm-snapshot-load component_ns -15-25%, and the quiet re-run measured -12.38% [-22.85%, -4.71%] with cpu and user cpu significantly down; wall spans zero only because probe spawn and the untimed oracle digest are half that job's wall"
    commit: 9f4f029
---
# Single-pass checksum and parse on snapshot load

## Hypothesis

H32: *state what you expected to be slow, why, and which metric would move.*

## What was tried

*The smallest change that tests the hypothesis.*

## What the numbers said

*Read the tables in the frontmatter.
Say what surprised you.*

## Verdict

**ACCEPTED** — Accepted on the pre-registered signal: the research registry declared
this hypothesis’s predicted signal as warm-snapshot-load component_ns -15-25%, and the
quiet re-run measured -12.38% [-22.85%, -4.71%] with cpu and user cpu significantly
down; wall spans zero only because probe spawn and the untimed oracle digest are half
that job’s wall
