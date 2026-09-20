---
title: "Linux PGO screen clears 3% on cold-scan-index and warm-revalidate"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-154
  title: "Linux PGO screen clears 3% on cold-scan-index and warm-revalidate"
  date: "2026-09-20"
  hypotheses:
    - H148
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
    control: HEAD fat-LTO / codegen-units=1 release probe
    candidate: same source rebuilt with -Cprofile-use after linux-v6.12 training
    control_binary:
      name: control
      sha256: 35712d10033d0289a1e5686e5890c6ff04ae914b4c12d9a69476d098339359a0
      size_bytes: 3039720
      args: []
    candidate_binary:
      name: candidate
      sha256: 5ebc8fca750d9221e7b3290050ae1bdcd0c83b9d1805f3e1ba0209b7d90bc07b
      size_bytes: 2325808
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-154-h148-pgo.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 504504068.5
          candidate_median: 460536184.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.026
          change_pct: -8.347
          ci95_low_pct: -10.354
          ci95_high_pct: -6.922
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 428622549.5
          candidate_median: 388744045.5
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.031
          change_pct: -8.914
          ci95_low_pct: -11.276
          ci95_high_pct: -7.046
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 625597500.0
          candidate_median: 581448000.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.01
          change_pct: -6.834
          ci95_low_pct: -8.285
          ci95_high_pct: -5.454
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 520896000.0
          candidate_median: 470319000.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.031
          change_pct: -8.497
          ci95_low_pct: -10.744
          ci95_high_pct: -6.524
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 102660500.0
          candidate_median: 109512500.0
          control_p95_over_median: 1.292
          candidate_p95_over_median: 1.072
          change_pct: 7.408
          ci95_low_pct: -11.487
          ci95_high_pct: 18.14
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 36929536.0
          candidate_median: 35895296.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.005
          change_pct: -2.784
          ci95_low_pct: -2.983
          ci95_high_pct: -2.608
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: superior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
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
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 480470994.5
          candidate_median: 441839895.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.031
          change_pct: -8.15
          ci95_low_pct: -8.641
          ci95_high_pct: -7.067
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 40754147.5
          candidate_median: 40914390.5
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.076
          change_pct: -0.434
          ci95_low_pct: -2.353
          ci95_high_pct: 1.74
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 590851500.0
          candidate_median: 550613500.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.048
          change_pct: -6.397
          ci95_low_pct: -7.145
          ci95_high_pct: -5.564
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 472382000.0
          candidate_median: 445208500.0
          control_p95_over_median: 1.039
          candidate_p95_over_median: 1.038
          change_pct: -7.327
          ci95_low_pct: -8.843
          ci95_high_pct: -2.311
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 118129000.0
          candidate_median: 115854000.0
          control_p95_over_median: 1.152
          candidate_p95_over_median: 1.075
          change_pct: -7.268
          ci95_low_pct: -19.404
          ci95_high_pct: 4.775
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 41564160.0
          candidate_median: 40716288.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.001
          change_pct: -2.064
          ci95_low_pct: -2.272
          ci95_high_pct: -1.943
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: superior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
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
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: no engine source change; PGO rebuild only; no dependency; no unsafe; profdata not checked in; unmeasured on macOS
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -8.347
    reason: "quiet linux-v6.12 cold-scan-index -8.35% [-10.35%, -6.92%] and warm-revalidate -8.15% [-8.64%, -7.07%]; RSS no worse; revalidate component flat so that wall win is spawn; Cargo.toml unchanged (profdata is host-specific)"
    commit: b46edf65
---
## What was predicted

H148: profile-guided optimization of the current fat-LTO / one-codegen-unit release
probe cuts Linux `cold-scan-index` and `warm-revalidate` wall at least 3% on
reconstructible `linux-v6.12`, with both intervals entirely below zero and peak RSS no
worse.

Screen only. Do not change `[profile.release]` unless both jobs clear.
Do not check a profdata file into the repository.
Not H93 (that id was reused).
Not a `PORTABLE` constant.
Not an engine source change.

## What was measured

Quiet 12-pair on reconstructible `linux-v6.12` (92,474 entries).
Control is the current release probe (`35712d10…`, 3,039,720 bytes).
Candidate is the same source rebuilt with `-Cprofile-use` after three training rounds of
`scan-index`, `snapshot-save`, `revalidate`, `summary`, and `summary --no-controls`
(`5ebc8fca…`, 2,325,808 bytes).

`cold-scan-index`: wall −8.35% [−10.35%, −6.92%]; component −8.91% [−11.28%, −7.05%];
user CPU −8.50%; peak RSS 35.2 → 34.2 MiB (−2.78%).

`warm-revalidate`: wall −8.15% [−8.64%, −7.07%]; component −0.43% [−2.35%, +1.74%]
(noninferior, includes zero).
Reconciliation is still ~41 ms.
The wall win on this job is process spawn and binary size, not the reconcile leftover.

0 invalid. Load/core 0.179–0.204 held.
Tallies identical (92,474 entries / 86,643 files / 1.64 GiB). Adaptive qualification
superior (exploratory).

## Decision

Accepted as a Linux screen.
Both named jobs cleared the 3% wall bar with intervals below zero and RSS no worse.

`[profile.release]` is unchanged.
A `profile-use` build needs a host-specific profdata file, and checking that file in is
a non-goal. Adoption into shipped release builds is a pipeline decision, not a one-line
Cargo.toml change.

Do not reuse H93. Do not treat this as a Darwin number.
Do not ship `PORTABLE`.
