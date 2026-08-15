---
title: Skip unread journal capture on the bootstrap apply path
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-058
  title: Skip unread journal capture on the bootstrap apply path
  date: "2026-08-15"
  hypotheses:
    - H90
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
    control: "post-exp-057 head: bootstrap batches journalled then cleared"
    candidate: "apply_with(journal: false) on the baseline path; arbitration identical, history capture skipped"
    control_binary:
      name: control
      sha256: 79e1a2d13ade986c5808eecf31c22e1cd4c03b71777dc1dd3bc97be3c5f0481a
      size_bytes: 1878920
      args: []
    candidate_binary:
      name: candidate
      sha256: 908a3c2903a6fec3d4cab386b3bcb8d7775b6361389dbd1b23f719d3ac50ea21
      size_bytes: 1879720
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: /tmp/fdu-realtree/results/run-exp058-journal-skip-rerun.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1725212613.5
          candidate_median: 1655319435.5
          change_pct: -5.058
          ci95_low_pct: -6.034
          ci95_high_pct: -3.543
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 681612706.5
          candidate_median: 644087948.0
          change_pct: -6.545
          ci95_low_pct: -8.428
          ci95_high_pct: -4.486
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 2682235000.0
          candidate_median: 2607588000.0
          change_pct: -1.956
          ci95_low_pct: -3.718
          ci95_high_pct: -1.14
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 1823178000.0
          candidate_median: 1730851000.0
          change_pct: -6.167
          ci95_low_pct: -7.309
          ci95_high_pct: -4.323
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 821520000.0
          candidate_median: 882167500.0
          change_pct: 4.786
          ci95_low_pct: 1.561
          ci95_high_pct: 10.408
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 276131840.0
          candidate_median: 277106688.0
          change_pct: -0.336
          ci95_low_pct: -3.585
          ci95_high_pct: 3.0
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1658992627.5
          candidate_median: 1677432726.5
          change_pct: 1.592
          ci95_low_pct: -1.23
          ci95_high_pct: 2.346
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 286393791.0
          candidate_median: 284112885.0
          change_pct: -0.829
          ci95_low_pct: -2.145
          ci95_high_pct: 0.221
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 2452638500.0
          candidate_median: 2458818000.0
          change_pct: -0.082
          ci95_low_pct: -0.663
          ci95_high_pct: 1.114
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 1627211500.0
          candidate_median: 1644729500.0
          change_pct: 2.063
          ci95_low_pct: -0.368
          ci95_high_pct: 3.682
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 825024500.0
          candidate_median: 812664500.0
          change_pct: -2.471
          ci95_low_pct: -6.94
          ci95_high_pct: 0.853
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 199886848.0
          candidate_median: 199612416.0
          change_pct: -0.118
          ci95_low_pct: -0.159
          ci95_high_pct: -0.076
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 40
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -5.058
    reason: "Cold wall -5.06% [-6.03%, -3.54%] on the confirming re-run after -4.62% [-8.45%, -1.02%] first time; the first run's RSS and warm-path alarms did not reproduce, and the reproducing context-switch increase costs no wall or CPU"
    commit: 8286c7e
---
`apply_baseline` routed bootstrap batches through the live journalling apply: every
changed op was cloned into the effective list, the delta was cloned again into the
journal, and `establish_baseline` then cleared that journal - per batch, for the entire
cold scan. One flag threads through `apply_with`: arbitration, validation, guards, and
stats are bit-identical in both modes, and only the unread history capture goes.
The clock-exhaustion probe keeps the journalling path, since its would-commit check
reads the minted delta.

exp-003 refuted this hypothesis’s ancestor on macOS at 60k in 2026-08-11’s build, via a
duplicated arbitration loop that was rightly rejected on complexity.
Three campaigns later the consumer is the measured bottleneck on Linux, the change is a
parameter rather than a twin loop, and the residue measured: cold-scan-index wall -4.62%
[-8.45%, -1.02%] in the first 12-trial run and -5.06% [-6.03%, -3.54%] in the confirming
re-run, with user CPU down ~6% in both.

The re-run was taken because the first run’s secondary signals looked alarming and
mostly were not: the +4.44% peak-RSS regression did not reproduce (-0.34%
[-3.58%, +3.00%]), and warm-revalidate’s +1.54% [+0.03%, +2.75%] - on a path this change
does not touch - relaxed to +1.59% [-1.23%, +2.35%], the exp-002 drift lesson again.
What does reproduce is roughly double the involuntary context switches (+120.58%
[+78.32%, +147.61%] from a base of a few hundred): the consumer finishes batches faster
and the interleaving shifts.
Recorded as a cost; wall and CPU say it is not a tax.
