---
title: Linux default /usr aggregate --threads 8 regresses; do not lower unlock
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-149
  title: Linux default /usr aggregate --threads 8 regresses; do not lower unlock
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
    control: HEAD automatic workers
    candidate: same probe --threads 8
    control_binary:
      name: control
      sha256: 2bc59b685deda30378ab83d0a9782ea8711451611bd03918f37183c602b978b7
      size_bytes: 3030680
      args: []
    candidate_binary:
      name: candidate
      sha256: 2bc59b685deda30378ab83d0a9782ea8711451611bd03918f37183c602b978b7
      size_bytes: 3030680
      args:
        - "--threads"
        - "8"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-149-h84-usr-default-threads-8-quiet.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 464479934.0
          candidate_median: 484288142.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.068
          change_pct: 7.123
          ci95_low_pct: 3.721
          ci95_high_pct: 11.301
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        component_ns:
          control_median: 462043587.0
          candidate_median: 481705934.5
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.068
          change_pct: 7.144
          ci95_low_pct: 3.707
          ci95_high_pct: 11.313
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 740649500.0
          candidate_median: 733473000.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.046
          change_pct: 1.343
          ci95_low_pct: -2.068
          ci95_high_pct: 3.183
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 489024500.0
          candidate_median: 480299500.0
          control_p95_over_median: 1.034
          candidate_p95_over_median: 1.113
          change_pct: 0.019
          ci95_low_pct: -1.79
          ci95_high_pct: 6.268
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 253552000.0
          candidate_median: 251403500.0
          control_p95_over_median: 1.088
          candidate_p95_over_median: 1.054
          change_pct: -0.541
          ci95_low_pct: -3.765
          ci95_high_pct: 2.223
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 71561216.0
          candidate_median: 72153088.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.005
          change_pct: 0.39
          ci95_low_pct: -0.02
          ci95_high_pct: 0.951
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
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: inconclusive
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
    notes: screen only; no engine change
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: 7.123
    reason: "default gitignore-on /usr aggregate --threads 8 is +7.12% [+3.72%, +11.30%] quiet regression; do not lower unlock or ship PORTABLE from this host"
    commit: 06b12212
    kept: neither
---
## What was predicted

If the H84 `--no-controls` sign were a reason to lower the unlock threshold, the default
gitignore-on `aggregate-summary` on nominated `/usr` would also clear 3% at
`--threads 8`. Predicted: it does not, and may regress, matching `linux-v6.12` (+1.75%).

Named before measuring:

- Same automatic-vs-`--threads 8` screen as exp-146, on `/usr`, default controls on.
- Quiet first. Do not lower the 25% bar.
- A regression here is a reason not to ship a lower unlock threshold.

## What was measured

Quiet held: initial load/core 0.082, final 0.120, 0 of 30 samples invalidated.
3 warmups, 12 timed pairs, interleaved.
Same HEAD probe (`2bc59b68…`). No RAM disk.

Subject: the same frozen `/usr` prefix as exp-148. Not reconstructible.

| Arm | Wall median | Component |
| --- | ---: | ---: |
| control (automatic 4) | 464.5 ms | 462.0 ms |
| candidate (`--threads 8`) | 484.3 ms | 481.7 ms |

Wall +7.12% [+3.72%, +11.30%] regression.
The interval is entirely above +3%. One control sample was 228.7 ms (half the others);
the other eleven controls sit in 445–498 ms and the candidate arm is still slower on
those pairs.

Default `/usr` summary is index-bound (~462 ms), the same class as `linux-v6.12` default
summary and `cold-scan-index`.

## What the determination said

Forcing 8 workers on the default gitignore-on path regresses a nominated deciding
subject.
Combined with exp-146’s +1.75% on `linux-v6.12`, this is why the `--no-controls`
sign must not become a lower unlock threshold or a `PORTABLE` `measured` constant from
this host.

No engine patch. `fdu-tk1b` stays open.
