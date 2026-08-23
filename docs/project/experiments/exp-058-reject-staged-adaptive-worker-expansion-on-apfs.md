---
title: Reject staged adaptive worker expansion on APFS
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-058
  title: Reject staged adaptive worker expansion on APFS
  date: "2026-08-15"
  hypotheses:
    - H99
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
    control: shipped one-shot controller
    candidate: staged ready-work-gated expansion
    control_binary:
      name: shipped
      sha256: 45624e749c84eca21758cd45b3e8d6af45918e33541ceccda8155e850bb9da24
      size_bytes: 1495248
      args: []
    candidate_binary:
      name: staged
      sha256: 45624e749c84eca21758cd45b3e8d6af45918e33541ceccda8155e850bb9da24
      size_bytes: 1495248
      args:
        - "--worker-policy"
        - staged-gated
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
          control_median: 1871768959.0
          candidate_median: 2987532000.0
          change_pct: 60.726
          ci95_low_pct: 49.8
          ci95_high_pct: 67.329
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        component_ns:
          control_median: 1508617333.5
          candidate_median: 2626811459.0
          change_pct: 74.877
          ci95_low_pct: 61.768
          ci95_high_pct: 83.363
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 9469943500.0
          candidate_median: 24246971500.0
          change_pct: 157.891
          ci95_low_pct: 124.997
          ci95_high_pct: 176.828
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        user_cpu_ns:
          control_median: 636678000.0
          candidate_median: 771643000.0
          change_pct: 21.351
          ci95_low_pct: 19.222
          ci95_high_pct: 23.745
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        system_cpu_ns:
          control_median: 8832735500.0
          candidate_median: 23465520500.0
          change_pct: 167.72
          ci95_low_pct: 132.474
          ci95_high_pct: 188.089
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        peak_rss_bytes:
          control_median: 93839360.0
          candidate_median: 99295232.0
          change_pct: 6.007
          ci95_low_pct: 5.419
          ci95_high_pct: 6.966
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
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
          - "peak_rss_bytes exceeds its +5% regression limit"
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
          peak_rss_bytes: rejected
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
    notes: Experiment-only controller; no production behavior retained.
  verdict:
    decision: rejected
    primary_job: adaptive-scan-index
    primary_metric: wall_ns
    change_pct: 60.726
    reason: "Staged expansion was 60.73% slower with a 95% interval of [+49.80%, +67.33%] and exceeded CPU, system-CPU, and scheduler-pressure gates."
    commit: 1d70c628cc4ed262ed2e4d04d992eb977b73c8b1
---
# Reject staged adaptive worker expansion on APFS

## Hypothesis

H99: staged expansion gated by useful ready work should correct the late slow-phase
sensitivity without the contention cost of an immediate jump to 16 workers.

Recorded at the time as `H86-staged-gated`, a local id coined before the registry
assigned H86 to the structural consumer rewrite that is now campaign 2’s centerpiece.
Renumbered to a free id so no number means two things, per the loop’s own rule.
Only the label changed; every measurement below is as recorded.

## What was tried

An experiment-only controller combined repeated windows with ready-frontier and
in-flight observations, then expanded in stages.
It shared the same frozen corpus, interleaved schedule, twelve discovery pairs, exact
oracle, and bounded trace as exp-057.

## What the numbers said

Gating did not rescue the design.
Median wall time was 60.73% worse than the shipped controller, with a paired 95%
interval of [+49.80%, +67.33%]. CPU, system CPU, and involuntary context switches failed
their resource gates.
Its interval against repeated windows spanned no difference, so the additional
controller complexity bought no measured benefit.

## Verdict

**REJECTED** — Staged expansion was 60.73% slower with a 95% interval of
[+49.80%, +67.33%] and exceeded CPU, system-CPU, and scheduler-pressure gates.

The prototype remains probe-only and no production controller code was selected.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
