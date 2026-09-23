---
title: "Linux H72 d_type skip misses 3% on source-tree v6.12"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-152
  title: "Linux H72 d_type skip misses 3% on source-tree v6.12"
  date: "2026-09-20"
  hypotheses:
    - H72
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
    control: H147 --no-controls aggregate (stat every listed child)
    candidate: "same probe --no-controls, skip directory and symlink statx via file_type"
    control_binary:
      name: control
      sha256: 213e4765e60eb910b575f556a1eb06a198209e897c317ce0fceb16a73e54a13a
      size_bytes: 3039176
      args:
        - "--no-controls"
    candidate_binary:
      name: candidate
      sha256: 8b16b478a1074c2edd1b5d22a60ce67bf5058472ed2d39ab434397901a7cc9f4
      size_bytes: 3039720
      args:
        - "--no-controls"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-152-h72-v612.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 36464080.5
          candidate_median: 35237923.0
          control_p95_over_median: 1.111
          candidate_p95_over_median: 1.135
          change_pct: -1.627
          ci95_low_pct: -3.333
          ci95_high_pct: -0.72
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 35647869.0
          candidate_median: 34463047.0
          control_p95_over_median: 1.111
          candidate_p95_over_median: 1.134
          change_pct: -1.521
          ci95_low_pct: -3.257
          ci95_high_pct: -0.603
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 137728000.0
          candidate_median: 134144500.0
          control_p95_over_median: 1.116
          candidate_p95_over_median: 1.133
          change_pct: -1.169
          ci95_low_pct: -2.453
          ci95_high_pct: -0.962
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 39105500.0
          candidate_median: 42555500.0
          control_p95_over_median: 1.31
          candidate_p95_over_median: 1.151
          change_pct: 12.874
          ci95_low_pct: -7.269
          ci95_high_pct: 33.174
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 98051500.0
          candidate_median: 96666000.0
          control_p95_over_median: 1.17
          candidate_p95_over_median: 1.084
          change_pct: -9.686
          ci95_low_pct: -11.809
          ci95_high_pct: 2.651
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 25657344.0
          candidate_median: 25657344.0
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
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "minor_faults straddles its +10% regression limit"
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
          minor_faults: inconclusive
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 123
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "listing file_type skip on RetainedState::Summary only; no dependency; no unsafe; one_filesystem still stats directories"
  verdict:
    decision: rejected
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -1.627
    reason: "quiet linux-v6.12 --no-controls aggregate -1.63% [-3.33%, -0.72%]; under 3%; stats 92474 to 86644; RSS flat; directory-heavy keep is exp-153"
    commit: f841662c
    kept: neither
---
## What was predicted

H72: the transient summary needs no directory or symlink attributes, so listing
`file_type` (`d_type` on Linux) can skip their `statx` calls.

Named before measuring: produced stats down by the directory-plus-symlink share; warm
`--no-controls` `aggregate-summary` wall down at least 3% on a directory-heavy tree.
Previous measure was −1.4% on a 6.4%-directory tree, below the gate.

`linux-v6.12` is that same class (6.2% directories, 62 symlinks).
It cannot accept H72 as a source-tree cut.
`one_filesystem` still stats directories.
Both arms `--no-controls` (H107). Peak RSS must not get worse.

## What was measured

Quiet 12-pair `--no-controls` `aggregate-summary` on reconstructible `linux-v6.12`
(92,474 entries). Wall −1.63% [−3.33%, −0.72%]. Component −1.52%. Peak RSS flat (24.5
MiB). 0 invalid. Load/core 0.117 held.

Counters: control 92,474 stats; candidate 86,644 (root plus every file).
Skip equals the 5,768 listed directories plus 62 symlinks.
Tallies match (86,643 files, 5,768 dirs, same bytes).

The symlink-heavy nominated `/usr` transfer (22% directories plus symlinks, −9.01%) is
exp-153.

## Decision

Rejected against the 3% bar on a reconstructible source tree.
Same class as the earlier −1.4% measure.
Do not lower the bar.
The directory-heavy claim is exp-153. Engine `f841662c` is kept or dropped with that
cell, not this one.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
