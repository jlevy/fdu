---
title: Bound adaptive scan diagnostics overhead
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-056
  title: Bound adaptive scan diagnostics overhead
  date: "2026-08-15"
  hypotheses:
    - H86-observability
  subject:
    tree_label: diagnostics-overhead
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
    control: scan without diagnostics
    candidate: bounded scan diagnostics enabled
    control_binary:
      name: normal
      sha256: db02591e5ff6fee2dd203f12498b5e94f0fbdad7ce108849fab7b27446f4df32
      size_bytes: 1495248
      args:
        - "--threads"
        - "6"
    candidate_binary:
      name: diagnostics
      sha256: db02591e5ff6fee2dd203f12498b5e94f0fbdad7ce108849fab7b27446f4df32
      size_bytes: 1495248
      args:
        - "--threads"
        - "6"
        - "--diagnostics"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: discovery
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: benchmarks/results/realtree/run-diagnostics-overhead.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1856422541.5
          candidate_median: 1850159396.0
          change_pct: -0.548
          ci95_low_pct: -1.091
          ci95_high_pct: 0.165
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 1493906937.5
          candidate_median: 1486442270.5
          change_pct: -0.462
          ci95_low_pct: -1.433
          ci95_high_pct: 0.118
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 9389286000.0
          candidate_median: 9330731000.0
          change_pct: -0.408
          ci95_low_pct: -1.69
          ci95_high_pct: 0.148
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 633061500.0
          candidate_median: 638721000.0
          change_pct: 0.124
          ci95_low_pct: -0.648
          ci95_high_pct: 1.263
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 8760753500.0
          candidate_median: 8694999500.0
          change_pct: -0.524
          ci95_low_pct: -1.816
          ci95_high_pct: 0.072
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 93315072.0
          candidate_median: 93552640.0
          change_pct: 0.281
          ci95_low_pct: 0.044
          ci95_high_pct: 0.625
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: discovery
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - major_faults does not establish non-regression
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
          major_faults: inconclusive
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
    notes: Opt-in internal evidence contract; disabled production scans retain their existing path.
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -0.548
    reason: "Diagnostics changed wall time by -0.55% with a 95% interval of [-1.09%, +0.17%], bounding overhead below the +3% non-regression margin."
    commit: 1f7251149f9ec8147a7ebc037a31555ae1351bde
---
# Bound adaptive scan diagnostics overhead

## Hypothesis

H86-observability: collecting a bounded worker-policy and backend trace should add no
more than 3% to scan wall time when explicitly enabled.

## What was tried

The same release probe scanned one frozen 100,001-entry APFS corpus with diagnostics off
and on. Twelve pairs followed three warmups, used an interleaved fixed-N schedule, and
required identical summaries, an unchanged corpus, and complete diagnostics in the
enabled arm.

## What the numbers said

Diagnostics changed median wall time by -0.55%; the paired 95% interval was
[-1.09%, +0.17%]. The interval’s upper bound is well inside the +3% non-regression
margin. No sample failed its oracle and the trace stayed within its event bound.

## Verdict

**ACCEPTED** — Diagnostics changed wall time by -0.55% with a 95% interval of
[-1.09%, +0.17%], bounding overhead below the +3% non-regression margin.

The trace remains opt-in and internal.
This result accepts the evidence mechanism, not a public API or a production controller
change.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
