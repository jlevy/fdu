---
title: Linux H84 --no-controls --threads 8 sign transfers to nominated /usr
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-148
  title: Linux H84 --no-controls --threads 8 sign transfers to nominated /usr
  date: "2026-09-20"
  hypotheses:
    - H84
  subject:
    tree_label: usr-prefix
    tree_root_id: 894d731f69bb61966f9ca61d0762a37bc542e0bbe4ba7c931fc75129c2229961
    tree_engine_digest: 44de521b2e23288d278e331e2dbc4a3beaeaa41d9e90e8777b13e60874df9998
    tree_provenance: "This cloud image's /usr prefix (ext4). Shape depends on the image package set, so it is not a recipe another machine can follow. Frozen for this session: no package installs after nomination."
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
    control: HEAD automatic workers --no-controls
    candidate: same probe --no-controls --threads 8
    control_binary:
      name: control
      sha256: 2bc59b685deda30378ab83d0a9782ea8711451611bd03918f37183c602b978b7
      size_bytes: 3030680
      args:
        - "--no-controls"
    candidate_binary:
      name: candidate
      sha256: 2bc59b685deda30378ab83d0a9782ea8711451611bd03918f37183c602b978b7
      size_bytes: 3030680
      args:
        - "--no-controls"
        - "--threads"
        - "8"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-148-h84-usr-summary-nocontrols-threads-8-quiet.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 93484583.5
          candidate_median: 82644132.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.069
          change_pct: -10.064
          ci95_low_pct: -14.721
          ci95_high_pct: -7.95
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 92677047.5
          candidate_median: 81716499.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.072
          change_pct: -10.174
          ci95_low_pct: -14.979
          ci95_high_pct: -8.162
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 341497500.0
          candidate_median: 313333500.0
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.031
          change_pct: -7.746
          ci95_low_pct: -10.05
          ci95_high_pct: -6.035
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 128293000.0
          candidate_median: 100240500.0
          control_p95_over_median: 1.225
          candidate_p95_over_median: 1.099
          change_pct: -25.27
          ci95_low_pct: -30.492
          ci95_high_pct: -17.148
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 209222500.0
          candidate_median: 212415000.0
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.154
          change_pct: 2.156
          ci95_low_pct: -2.909
          ci95_high_pct: 8.602
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 29593600.0
          candidate_median: 29593600.0
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: screen only; no engine change
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -10.064
    reason: "confirmatory --no-controls sign on nominated /usr: -10.06% quiet; still not a shipped PORTABLE constant; minor_faults inferior"
    commit: a58f9e30
---
## What was predicted

H84’s `--no-controls` aggregate sign on `linux-v6.12` (−5.42% quiet at `--threads 8`)
should transfer to the other nominated deciding subject on this host, `/usr`
(usr-prefix, 208,411 entries).
Same automatic-vs-8 screen.
Still not a shipped `PORTABLE` constant.

Named before measuring:

- Job: 12-pair `aggregate-summary` with `--no-controls` on both arms; candidate adds
  `--threads 8`.
- Quiet first. Do not lower the 25% bar.
- `/usr` is nominated and deciding, not reconstructible.
  A 3% quiet pair here strengthens the sign; it does not size a constant.

## What was measured

Quiet held: initial load/core 0.077, final 0.171, 0 of 30 samples invalidated.
3 warmups, 12 timed pairs, interleaved.
Same HEAD probe (`2bc59b68…`). `FDU_COUNTERS` unset.
No RAM disk.

Subject: this image’s `/usr` prefix, frozen for the session (no package installs).
208,411 entries / 162,440 files / 14,325 directories / 31,646 symlinks.
Not reconstructible.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control (automatic 4) | 93.5 ms | 92.7 ms | same |
| candidate (`--threads 8`) | 82.6 ms | 81.7 ms | same |

Wall −10.06% [−14.72%, −7.95%]. Component −10.17%. Adaptive qualification **inferior**
on `minor_faults` (+50%), same class as the `linux-v6.12` and 450k `--no-controls`
signs.

## What the determination said

The `--no-controls` warm sign transfers to the second nominated deciding subject.
It is still a sign, not a shipped constant: more workers fault more, the default
gitignore-on path regresses at 8, and this host is a 4-core VM.

No engine patch. Do not change `PORTABLE` to `measured`. `fdu-tk1b` stays open.
