---
title: "Linux transient batch recycle clears 3% after H85 misses 20%"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-151
  title: "Linux transient batch recycle clears 3% after H85 misses 20%"
  date: "2026-09-20"
  hypotheses:
    - H147
  subject:
    tree_label: linux-v6.12
    tree_root_id: d7c0dad8d82c8bb394f459b1dd77c9e8af60d0482cca92199519668546a5147e
    tree_engine_digest: a298a9c2c8f8ee910d22b87093a739993cdf153104590f4638eeee740b239bd0
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone .git directory as git left it."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293224
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: Intel(R) Xeon(R) Processor
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16791945216
    host_system: Linux 6.12.94+
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: "HEAD --no-controls aggregate, consumer drops Observation batches"
    candidate: "same probe --no-controls, drained batches returned to producing worker"
    control_binary:
      name: control
      sha256: 84060ec54c559a261960c5e7659020da0f045ef9733e54a0daf72b592e9d7e82
      size_bytes: 3030680
      args:
        - "--no-controls"
    candidate_binary:
      name: candidate
      sha256: 213e4765e60eb910b575f556a1eb06a198209e897c317ce0fceb16a73e54a13a
      size_bytes: 3039176
      args:
        - "--no-controls"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-150-h85-recycle-v612.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 36025695.0
          candidate_median: 34475126.5
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.015
          change_pct: -4.981
          ci95_low_pct: -5.92
          ci95_high_pct: -4.33
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 35246676.0
          candidate_median: 33633741.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.016
          change_pct: -5.125
          ci95_low_pct: -5.954
          ci95_high_pct: -4.514
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 133010500.0
          candidate_median: 129044000.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.023
          change_pct: -3.269
          ci95_low_pct: -4.504
          ci95_high_pct: -1.942
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 40903500.0
          candidate_median: 45572000.0
          control_p95_over_median: 1.563
          candidate_p95_over_median: 1.243
          change_pct: 18.104
          ci95_low_pct: -27.395
          ci95_high_pct: 49.433
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 91421500.0
          candidate_median: 85063500.0
          control_p95_over_median: 1.184
          candidate_p95_over_median: 1.128
          change_pct: -8.762
          ci95_low_pct: -21.71
          ci95_high_pct: 11.437
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 25632768.0
          candidate_median: 25632768.0
          control_p95_over_median: 1.0
          candidate_p95_over_median: 1.0
          change_pct: 0.0
          ci95_low_pct: 0.0
          ci95_high_pct: 0.0
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "minor_faults exceeds its +10% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: rejected
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 186
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: same patch as exp-150; private recycle; no dependency; no unsafe; unmeasured on macOS
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -4.981
    reason: "quiet linux-v6.12 --no-controls aggregate -4.98% [-5.92%, -4.33%]; RSS flat; default gitignore-on placebo +0.91% includes zero; H85 20% missed so this is the 3% keep"
    commit: 5c6e6394
---
## What was predicted

H147: after H85 missed its 20% mimalloc bar, the same recycle still clears the iteration
3% rule on a reconstructible deciding subject, without moving the gitignore-on index
path or peak RSS.

Named before recording: quiet `--no-controls` `aggregate-summary` on `linux-v6.12`, 3%
wall, interval entirely below zero, tally oracle, RSS no worse.
Placebo on the default index path includes zero.

Not H85. Not a `PORTABLE` thread constant.
Not an H86 restart. Unmeasured on macOS.

## What was measured

Quiet 12-pair `--no-controls` `aggregate-summary` on reconstructible `linux-v6.12`
(92,474 entries). Wall −4.98% [−5.92%, −4.33%]. Component −5.12%. Peak RSS flat (24.4
MiB). 0 invalid. Load/core 0.127 held.
Adaptive qualification inferior on `minor_faults` (+45%); wall and RSS are the accept
metrics.

Placebo, quiet, default gitignore-on `aggregate-summary` on the same tree: +0.91%
[−1.18%, +2.68%].

H85’s 20% screen on this cell and on incomplete quiet `linux-450k` (−11.31%, n=7) is
exp-150. That id stays rejected.

## Decision

Accepted as the 3% keep of the H85 recycle on Linux.
Engine kept (`5c6e6394`). ~186 lines, no dependency, no `unsafe`. Do not retry H85’s 20%
bar. Do not claim this on Darwin.
Do not treat `--no-controls` as the shipped default.

The public `scan` path now starts every batch after the first at `batch_size` capacity
(`send_full` → `next_vec`). That allocation shape is inherited, not measured: this cell
drove the fold path.
The worker’s final flush leaves an empty vec rather than a replacement that would never
be used.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
