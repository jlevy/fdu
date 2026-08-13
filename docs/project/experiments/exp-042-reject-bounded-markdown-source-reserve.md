---
title: Reject bounded Markdown source reserve
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-042
  title: Reject bounded Markdown source reserve
  date: 2026-08-13
  hypotheses:
    - H67
  subject:
    tree_label: generated-markdown-2000
    tree_root_id: 58aeab71141cc4924989599a2dbf53bcae48f3cec814d1cc2a171f22f9d1ab85
    tree_engine_digest: 8163a878594a9f43b44aed7d73bbde6c725c24aba789b70538a22e1b2d0539be
    tree_entries: 2001
    tree_directories: 1
    tree_files: 2000
    tree_symlinks: 0
    tree_apparent_bytes: 60071000
    tree_allocated_bytes: 65536000
    tree_max_depth: 1
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
    control: zero-capacity retained Markdown buffer
    candidate: reserve up to the known bounded file size
    control_binary:
      name: baseline
      sha256: 5a41e15cbc4199ce9ad790baf6fd1ad2426559cec4f3f71754488f9dba56d7a1
      size_bytes: 1115168
      args: []
    candidate_binary:
      name: candidate
      sha256: 8f098d830b621e7e1890f576d2e571b62e8f0b6870be8a59572c60463755b3db
      size_bytes: 1115168
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: null
  results:
    - job: document-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 23128353.5
          candidate_median: 23189396.0
          change_pct: 0.988
          ci95_low_pct: -0.862
          ci95_high_pct: 5.155
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 15820416.5
          candidate_median: 16314375.5
          change_pct: 3.913
          ci95_low_pct: -1.382
          ci95_high_pct: 5.865
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 20703000.0
          candidate_median: 20994500.0
          change_pct: 1.224
          ci95_low_pct: -0.475
          ci95_high_pct: 4.371
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 17870500.0
          candidate_median: 18251500.0
          change_pct: 0.751
          ci95_low_pct: -0.206
          ci95_high_pct: 2.664
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 2828500.0
          candidate_median: 2883000.0
          change_pct: 5.929
          ci95_low_pct: -8.16
          ci95_high_pct: 21.375
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 2217354.5
          candidate_median: 2411375.0
          change_pct: 7.05
          ci95_low_pct: -2.264
          ci95_high_pct: 15.625
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 7299072.0
          candidate_median: 7372800.0
          change_pct: 1.124
          ci95_low_pct: 0.335
          ci95_high_pct: 1.691
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: markdown-prose
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 382016146.0
          candidate_median: 365608854.0
          change_pct: -3.548
          ci95_low_pct: -14.495
          ci95_high_pct: 7.449
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 334212208.5
          candidate_median: 332312166.5
          change_pct: 0.866
          ci95_low_pct: -9.239
          ci95_high_pct: 10.444
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1321549000.0
          candidate_median: 1275532000.0
          change_pct: -3.246
          ci95_low_pct: -10.195
          ci95_high_pct: 3.135
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 870655500.0
          candidate_median: 864372000.0
          change_pct: -0.767
          ci95_low_pct: -2.539
          ci95_high_pct: 1.093
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 458875000.0
          candidate_median: 415794500.0
          change_pct: -6.191
          ci95_low_pct: -26.634
          ci95_high_pct: 7.866
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
          control_median: 16146432.0
          candidate_median: 16408576.0
          change_pct: -0.053
          ci95_low_pct: -3.001
          ci95_high_pct: 4.943
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 5
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: One bounded capacity hint; production change reverted
  verdict:
    decision: rejected
    primary_job: markdown-prose
    primary_metric: wall_ns
    change_pct: -3.548
    reason: "Markdown wall moved -3.55%, but the 95% paired interval [-14.49%, +7.45%] crossed zero; cache hits were neutral, so the capacity hint is reverted"
    commit: null
---
# Reject bounded Markdown source reserve

## Hypothesis

H67 predicted that reserving the known, bounded Markdown file size would remove repeated
buffer growth. The preregistered signal was at least a 3% improvement in both wall and
component time on the immutable 2,000-file Markdown corpus, without a semantic-digest,
cache-hit, or memory regression.

## What was tried

The candidate replaced an empty retained-source vector with a capacity hint capped at 1
MiB. It added no dependency, unsafe code, or unbounded allocation.
Both variants ran 12 interleaved pairs after three warmups; the probe and independent
tree oracle agreed on every sample.

## What the numbers said

Markdown wall moved from 382.0 ms to 365.6 ms, a paired median change of −3.55%, but the
95% interval of [−14.49%, +7.45%] included both a useful win and a regression.
The component changed only −0.79% with a similarly inconclusive interval.
Cache-hit wall moved +0.99% and peak RSS was neutral.
Most corpus files already fit in the first 64 KiB read, so the proposed capacity hint
removed little actual growth.

## Verdict

**REJECTED** — Markdown wall moved −3.55%, but the 95% paired interval [−14.49%, +7.45%]
crossed zero. Cache-hit behavior was neutral, so the capacity hint was reverted.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
