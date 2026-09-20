---
title: "Linux H72 d_type skip clears 3% on symlink-heavy /usr"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-153
  title: "Linux H72 d_type skip clears 3% on symlink-heavy /usr"
  date: "2026-09-20"
  hypotheses:
    - H72
  subject:
    tree_label: usr-prefix
    tree_root_id: 894d731f69bb61966f9ca61d0762a37bc542e0bbe4ba7c931fc75129c2229961
    tree_engine_digest: b7e4da0044760ab9046c739e52e4d3d47f1a64657c327a2fc874dd2ba3d0fef4
    tree_provenance: "This cloud image's /usr prefix (ext4). Shape depends on the image package set. Fresh baseline for this cell because the nominated digest had drifted (198,150 to 208,411 entries; digest b7e4da00 versus 44de521b). Not a recipe another machine can follow."
    tree_reconstructible: false
    tree_entries: 208411
    tree_directories: 14325
    tree_files: 162440
    tree_symlinks: 31646
    tree_apparent_bytes: 6019421920
    tree_allocated_bytes: 6423949312
    tree_max_depth: 17
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
    run_artifact: /tmp/fdu-realtree/results/run-exp-152-h72-usr.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 86302076.0
          candidate_median: 78582763.5
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.045
          change_pct: -9.01
          ci95_low_pct: -12.52
          ci95_high_pct: -6.298
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 85459327.0
          candidate_median: 77755569.5
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.046
          change_pct: -9.072
          ci95_low_pct: -12.629
          ci95_high_pct: -6.373
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 323976500.0
          candidate_median: 294383000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.032
          change_pct: -9.733
          ci95_low_pct: -11.243
          ci95_high_pct: -7.014
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 104475500.0
          candidate_median: 101575500.0
          control_p95_over_median: 1.109
          candidate_p95_over_median: 1.12
          change_pct: -0.773
          ci95_low_pct: -11.183
          ci95_high_pct: 6.516
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 222608000.0
          candidate_median: 192011000.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.123
          change_pct: -14.097
          ci95_low_pct: -17.542
          ci95_high_pct: -10.191
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 29458432.0
          candidate_median: 29458432.0
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
    notes: same patch as exp-152; private summary-path skip; no dependency; no unsafe; unmeasured on macOS; /usr not reconstructible
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -9.01
    reason: "quiet nominated /usr --no-controls aggregate -9.01% [-12.52%, -6.30%]; 22% skippable dirs+symlinks; RSS flat; v6.12 companion -1.63% noninferior (exp-152)"
    commit: f841662c
---
## What was predicted

H72: on a directory-heavy tree the transient summary can skip `statx` for directories
and symlinks. `linux-v6.12` is not that tree (exp-152, −1.63%). Nominated `/usr` is
14,325 directories and 31,646 symlinks of 208,411 entries (22% skippable).

Named before measuring: stats down by that share; quiet `--no-controls`
`aggregate-summary` wall down at least 3% with the interval below zero; peak RSS no
worse; tallies identical.
Fresh baseline for this cell because the image fingerprint had drifted since nomination.
Not reconstructible.
Do not treat this as a `PORTABLE` constant.

## What was measured

Quiet 12-pair `--no-controls` `aggregate-summary` on nominated `/usr` (208,411 entries).
Wall −9.01% [−12.52%, −6.30%]. Component −9.07%. System CPU −14.10%. Peak RSS flat (28.1
MiB). 0 invalid. Load/core 0.077–0.171 held.
Tree verified unchanged across the run against a fresh baseline
(`tree-usr-prefix-h72.json`, digest `b7e4da00…`).

Counters: control 208,411 stats; candidate 162,441 (root plus every file).
Skip equals the listed directories plus symlinks.
Tallies match (162,440 files, 14,324 dirs, same bytes).

The reconstructible source-tree companion is exp-152 (−1.63%, noninferior).

## Decision

Accepted as H72’s directory-heavy keep.
Engine kept (`f841662c`). Interval entirely below −6%. RSS flat.
Source trees stay noninferior.
Do not retry H71. Do not skip directory stats when `one_filesystem` is on.
Unmeasured on macOS (bulk path already has attrs).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
