---
title: Decode complete UTF-8 chunks in place
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-050
  title: Decode complete UTF-8 chunks in place
  date: "2026-08-13"
  hypotheses:
    - H82
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
    trials: 32
    warmups: 2
    interleaved: true
    control: copy every chunk through a temporary vector
    candidate: decode directly unless a UTF-8 carry is pending
    control_binary:
      name: baseline
      sha256: 5a41e15cbc4199ce9ad790baf6fd1ad2426559cec4f3f71754488f9dba56d7a1
      size_bytes: 1115168
      args:
        - "--threads"
        - "1"
    candidate_binary:
      name: candidate
      sha256: fa15fe670cf5ef3d075539dc482a4cd4b6b6f0245d05b902fee8c766244310c2
      size_bytes: 1115168
      args:
        - "--threads"
        - "1"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: null
  results:
    - job: markdown-prose
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 251493687.5
          candidate_median: 218914145.5
          change_pct: -12.042
          ci95_low_pct: -16.457
          ci95_high_pct: -8.38
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 32
        component_ns:
          control_median: 235354729.0
          candidate_median: 203491833.0
          change_pct: -13.668
          ci95_low_pct: -19.216
          ci95_high_pct: -8.816
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 32
        cpu_ns:
          control_median: 1293842500.0
          candidate_median: 1195189000.0
          change_pct: -6.786
          ci95_low_pct: -8.668
          ci95_high_pct: -5.209
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 32
        user_cpu_ns:
          control_median: 851512500.0
          candidate_median: 745498000.0
          change_pct: -12.244
          ci95_low_pct: -13.19
          ci95_high_pct: -11.661
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 32
        system_cpu_ns:
          control_median: 436226000.0
          candidate_median: 442964000.0
          change_pct: 4.773
          ci95_low_pct: -1.581
          ci95_high_pct: 7.579
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 32
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
          control_median: 16048128.0
          candidate_median: 14573568.0
          change_pct: -9.116
          ci95_low_pct: -11.213
          ci95_high_pct: -4.828
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 32
  reference_tools: []
  complexity:
    lines_changed: 7
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "One direct fast path plus the existing bounded carry path; no new dependency, unsafe code, or failure mode"
  verdict:
    decision: accepted
    primary_job: markdown-prose
    primary_metric: wall_ns
    change_pct: -12.042
    reason: "The 32-pair Markdown run improved wall 12.04% with a 95% interval of [-16.46%, -8.38%], cut user CPU and peak RSS, preserved digests and goldens, and left self-host and cache-hit latency neutral"
    commit: 2fef9bf
---
# Decode complete UTF-8 chunks in place

## Hypothesis

H82 predicted that the basic analyzer’s unconditional temporary vector was copying every
read chunk before UTF-8 decoding.
Decoding a complete chunk in place should remove one allocation and byte copy while
retaining the existing carry path for a multibyte character split across reads.

## What was tried

The candidate adds one no-carry fast path and extracts the existing UTF-8 admission into
a helper shared with the carry path.
It adds no dependency, unsafe code, unbounded read, or semantic branch.
Exhaustive chunk-boundary unit tests and all 87 CLI goldens remained unchanged, and
every performance sample matched the immutable-tree and metric digests.

## What the numbers said

The 32-pair primary Markdown run used one worker because an unrelated benchmark was
saturating the remaining host cores.
Wall improved 12.04%, with the 95% interval wholly below zero at [−16.46%, −8.38%]. The
measured analysis component improved 13.67%, user CPU improved 12.24%, and peak RSS
improved 9.12%, all with intervals below zero.

A default-worker diagnostic under the same contention could not establish wall time, but
independently measured user CPU improved 12.71% and peak RSS improved 8.29%. Plain text
and the small self-host tree were wall-neutral; both preserved their digests and reduced
peak RSS, while self-host user CPU improved 17.95%. Document cache hits were neutral, as
expected because they do not decode content.

## Verdict

**ACCEPTED** — The primary Markdown run improved wall 12.04% with a 95% interval of
[−16.46%, −8.38%], cut component time, user CPU, and peak RSS, preserved all semantic
oracles, and left self-host and cache-hit latency neutral.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
