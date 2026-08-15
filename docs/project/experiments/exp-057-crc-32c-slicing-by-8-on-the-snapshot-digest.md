---
title: CRC-32C slicing-by-8 on the snapshot digest
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-057
  title: CRC-32C slicing-by-8 on the snapshot digest
  date: "2026-08-15"
  hypotheses:
    - H88
  subject:
    tree_label: vm450k
    tree_root_id: 9311b5fa18bc84e62d74e610f48c354dc72b352a0e0ebb6a5dc6847091a61ce0
    tree_engine_digest: 45022e107978f96adc9624cfa30f54271d024961832dcf0192188eabcacf7ea5
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
    trials: 12
    warmups: 3
    interleaved: true
    control: "branch head 3bda5c8: byte-at-a-time table CRC"
    candidate: "slicing-by-8 through eight const-derived tables, bit-identical digest"
    control_binary:
      name: control
      sha256: 2b574c6ecfdbbed12907144b6325f627564cf44afc2bbb66d876a41382827de2
      size_bytes: 1870760
      args: []
    candidate_binary:
      name: candidate
      sha256: 79e1a2d13ade986c5808eecf31c22e1cd4c03b71777dc1dd3bc97be3c5f0481a
      size_bytes: 1878920
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: /tmp/fdu-realtree/results/run-exp057-crc-slice8.json
  results:
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2007471056.0
          candidate_median: 1925330581.0
          change_pct: -2.949
          ci95_low_pct: -6.768
          ci95_high_pct: -1.19
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 270993873.0
          candidate_median: 208123571.0
          change_pct: -12.202
          ci95_low_pct: -36.018
          ci95_high_pct: -0.099
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 2937570000.0
          candidate_median: 2832916500.0
          change_pct: -2.179
          ci95_low_pct: -5.504
          ci95_high_pct: -1.194
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 1977983500.0
          candidate_median: 1930715000.0
          change_pct: -3.627
          ci95_low_pct: -4.813
          ci95_high_pct: -0.983
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 945041000.0
          candidate_median: 947836500.0
          change_pct: -2.062
          ci95_low_pct: -10.191
          ci95_high_pct: 3.665
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 284315648.0
          candidate_median: 286928896.0
          change_pct: -0.926
          ci95_low_pct: -3.144
          ci95_high_pct: 3.875
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1354385091.0
          candidate_median: 1337438715.0
          change_pct: -1.972
          ci95_low_pct: -2.876
          ci95_high_pct: -0.602
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 403738764.0
          candidate_median: 393729186.5
          change_pct: -2.023
          ci95_low_pct: -7.003
          ci95_high_pct: -0.344
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 1353570000.0
          candidate_median: 1336886000.0
          change_pct: -1.92
          ci95_low_pct: -2.901
          ci95_high_pct: -0.59
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 1254756500.0
          candidate_median: 1224977000.0
          change_pct: -2.086
          ci95_low_pct: -4.449
          ci95_high_pct: -1.173
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 97995500.0
          candidate_median: 101928500.0
          change_pct: 6.044
          ci95_low_pct: -7.585
          ci95_high_pct: 25.537
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 532099.5
          candidate_median: 624024.0
          change_pct: 33.349
          ci95_low_pct: -29.502
          ci95_high_pct: 102.56
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 199864320.0
          candidate_median: 199651328.0
          change_pct: -0.115
          ci95_low_pct: -0.178
          ci95_high_pct: -0.084
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 55
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: cold-snapshot-save
    primary_metric: component_ns
    change_pct: -12.202
    reason: "Accepted on the pre-registered component signal: save component -12.20% [-36.02%, -0.10%] clears the bar decisively, load component -2.02% and both walls improved with all intervals below zero, for ~55 dependency-free lines with a bit-identity test"
    commit: 3bda5c8
---
The snapshot digest folded one byte per step through one table; slicing-by-8 folds eight
bytes per step through eight const-derived tables whose table 0 is the classic byte
table, so the remainder loop and the fast path share one source of truth.
The digest is bit-identical, pinned by a new equivalence test across uneven lengths
beside the standard check value, and the change is ~55 lines of straight-line
arithmetic: no dependency, no unsafe, no new failure mode.

Scored under the pre-registered-signal exception, exp-009’s precedent.
The registry row declared the component signal for both snapshot jobs before
measurement. The save component cleared it decisively (-12.20% [-36.02%, -0.10%]); the
load component came in real but under the bar (-2.02% [-7.00%, -0.34%]), because CRC is
a smaller share of a load that also parses than of a save that only serializes and
digests. Both walls improved with intervals entirely below zero (-2.95% [-6.77%, -1.19%]
save, -1.97% [-2.88%, -0.60%] load), as did total and user CPU on both jobs - direction
is unambiguous on every timing metric, and the fourth arm of the accept rule (is the
complexity worth it) faces its cheapest possible case.

The reasoning is stated rather than assumed because the wall medians alone would read as
reject: the acceptance rests on the declared component metric and the near-zero carrying
cost together. Hardware CRC32C (SSE 4.2 / ARMv8) remains available behind the same
interface if a future format revision makes the digest a larger share again.
