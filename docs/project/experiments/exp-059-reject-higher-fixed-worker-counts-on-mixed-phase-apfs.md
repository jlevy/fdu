---
title: Reject higher fixed worker counts on mixed-phase APFS
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-059
  title: Reject higher fixed worker counts on mixed-phase APFS
  date: "2026-08-15"
  hypotheses:
    - H96
  subject:
    tree_label: adaptive-fast-slow-100k
    tree_root_id: 314ea87cd4c0f7e4669757bc583cc1551ab6b944a86c267db74da7de704a6584
    tree_engine_digest: bd3452ca670e3f3a0e8382fca0f51b2b1d41321aa64c0ff67dafd71440481644
    tree_entries: 100001
    tree_directories: 60314
    tree_files: 39687
    tree_symlinks: 0
    tree_apparent_bytes: 35888409
    tree_allocated_bytes: 135069696
    tree_max_depth: 49
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
    control: fixed six workers
    candidate: fixed eight workers
    control_binary:
      name: fixed6
      sha256: 45624e749c84eca21758cd45b3e8d6af45918e33541ceccda8155e850bb9da24
      size_bytes: 1495248
      args:
        - "--threads"
        - "6"
    candidate_binary:
      name: fixed8
      sha256: 45624e749c84eca21758cd45b3e8d6af45918e33541ceccda8155e850bb9da24
      size_bytes: 1495248
      args:
        - "--threads"
        - "8"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: discovery
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: benchmarks/results/realtree/run-discovery-fast-slow-v6-uncontrolled.json
  results:
    - job: adaptive-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1878282521.0
          candidate_median: 2532063771.5
          change_pct: 35.551
          ci95_low_pct: 33.339
          ci95_high_pct: 37.978
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        component_ns:
          control_median: 1515009333.0
          candidate_median: 2170179604.0
          change_pct: 43.86
          ci95_low_pct: 41.451
          ci95_high_pct: 47.31
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 9502257000.0
          candidate_median: 17636294500.0
          change_pct: 86.441
          ci95_low_pct: 82.891
          ci95_high_pct: 92.438
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        user_cpu_ns:
          control_median: 634950000.0
          candidate_median: 819751500.0
          change_pct: 28.297
          ci95_low_pct: 23.559
          ci95_high_pct: 31.201
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        system_cpu_ns:
          control_median: 8860585000.0
          candidate_median: 16806027000.0
          change_pct: 90.695
          ci95_low_pct: 87.042
          ci95_high_pct: 96.985
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        peak_rss_bytes:
          control_median: 93782016.0
          candidate_median: 95526912.0
          change_pct: 1.863
          ci95_low_pct: 1.463
          ci95_high_pct: 2.158
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: discovery
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "cpu_ns exceeds its +50% regression limit"
          - "system_cpu_ns exceeds its +75% regression limit"
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: rejected
          involuntary_context_switches: rejected
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: rejected
          voluntary_context_switches: within-limit
        policy_stable: true
        policy_rule: zero-structurally-harmful-and-one-outcome-sensitivity-signature-v2
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: Fixed counts were diagnostic controls; the shipped automatic policy is unchanged.
  verdict:
    decision: rejected
    primary_job: adaptive-scan-index
    primary_metric: wall_ns
    change_pct: 35.551
    reason: "Eight workers were 35.55% slower than six with a 95% interval of [+33.34%, +37.98%]; ten and sixteen regressed further."
    commit: 1d70c628cc4ed262ed2e4d04d992eb977b73c8b1
---
# Reject higher fixed worker counts on mixed-phase APFS

## Hypothesis

H96: if the slow suffix benefits from deeper concurrency, a fixed count above six should
identify a useful hardware bound before controller design.

Recorded at the time as `H87-fixed-worker-knee`, a local id coined before the registry
assigned H87 to the `spawn_save` deep clone (exp-063). Renumbered to the next free id so
no number means two things, per the loop’s own rule.
Only the label changed; every measurement below is as recorded.

## What was tried

The release probe ran fixed 6, 8, 10, and 16 worker controls in the same twelve-pair
interleaved discovery run as exp-057 and exp-058. Eight represents the performance-core
count, ten the available logical CPUs, and sixteen the shipped automatic ceiling on this
M1 Pro.

## What the numbers said

Eight workers were 35.55% slower than six, with a paired 95% interval of
[+33.34%, +37.98%]. Ten was 16.95% slower than eight, and sixteen was another 5.90%
slower than ten. Fixed six and the shipped policy were practically level: +0.76%
[-0.06%, +1.18%]. The service signal called the suffix slow, but more parallelism
amplified kernel and scheduler contention rather than hiding latency.

## Verdict

**REJECTED** — Eight workers were 35.55% slower than six with a 95% interval of
[+33.34%, +37.98%]; ten and sixteen regressed further.

Hardware topology supplies safety bounds here, not a selector signal.
The production worker caps and automatic policy remain unchanged.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
