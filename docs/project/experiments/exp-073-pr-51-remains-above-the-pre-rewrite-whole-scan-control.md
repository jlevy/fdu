---
title: "PR #51 remains above the pre-rewrite whole-scan control"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-073
  title: "PR #51 remains above the pre-rewrite whole-scan control"
  date: "2026-09-01"
  hypotheses: []
  subject:
    tree_label: rustup-toolchains
    tree_root_id: 63a6945ccbafe92ddea2de69e10ac10530c3cbd1ffe8e4fe52ba12236280227b
    tree_engine_digest: 594842ccb17c6c0dbe8004ee54ef6afef2383da545b6c55a2da1d7edb2a87dfa
    tree_provenance: Local rustup toolchain installation observed in place; package-manager state is not reconstructible.
    tree_reconstructible: false
    tree_entries: 119368
    tree_directories: 3775
    tree_files: 115593
    tree_symlinks: 0
    tree_apparent_bytes: 3630247080
    tree_allocated_bytes: 3983802368
    tree_max_depth: 16
    tree_mutated_during_run: false
    host_cpu: Apple M1 Pro
    host_arch: arm64
    host_cores: 10
    host_performance_cores: 8
    host_efficiency_cores: 2
    host_memory_bytes: 34359738368
    host_system: Darwin 25.5.0
    filesystem: apfs
    host_virtualization: bare-metal
    os_cache: warm-steady
  method:
    trials: 6
    warmups: 1
    interleaved: true
    control: pre-rewrite main at b75bf85
    candidate: "PR #51 head at e8f1bed"
    control_binary:
      name: main
      sha256: 2fb01956a7a2b59576fefdf4a44d88cf9fff9203d7aca49d06870b6a757a7133
      size_bytes: 1561440
      args: []
    candidate_binary:
      name: pr51
      sha256: a9845b4996c2927470a8f957f1d5bc137bbc98d42c6acf93aa8a8f323048fa51
      size_bytes: 1826000
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-pr51-review-runs/run-pr-base-head-main.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 349866375.0
          candidate_median: 847697812.5
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.16
          change_pct: 144.455
          ci95_low_pct: 129.733
          ci95_high_pct: 165.69
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 6
        component_ns:
          control_median: 132360021.0
          candidate_median: 627993187.5
          control_p95_over_median: 1.186
          candidate_p95_over_median: 1.186
          change_pct: 386.479
          ci95_low_pct: 324.221
          ci95_high_pct: 414.438
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 6
        cpu_ns:
          control_median: 911816000.0
          candidate_median: 1419174000.0
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.016
          change_pct: 55.497
          ci95_low_pct: 49.363
          ci95_high_pct: 59.054
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 6
        user_cpu_ns:
          control_median: 327581000.0
          candidate_median: 841823000.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.055
          change_pct: 156.476
          ci95_low_pct: 150.948
          ci95_high_pct: 165.653
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 6
        system_cpu_ns:
          control_median: 580564500.0
          candidate_median: 576405500.0
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.04
          change_pct: -1.799
          ci95_low_pct: -11.778
          ci95_high_pct: 3.825
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 6
        peak_rss_bytes:
          control_median: 64143360.0
          candidate_median: 106348544.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.011
          change_pct: 66.054
          ci95_low_pct: 57.983
          ci95_high_pct: 67.434
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 6
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "cpu_ns straddles its +50% regression limit"
          - "peak_rss_bytes exceeds its +5% regression limit"
          - "minor_faults exceeds its +10% regression limit"
          - voluntary_context_switches is missing a paired percent interval
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: inconclusive
          involuntary_context_switches: inconclusive
          major_faults: within-limit
          minor_faults: rejected
          peak_rss_bytes: rejected
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - The uncontrolled host regime prevents a final parity claim; repeat the finished design on a quiet host.
    notes: Measurement only; implementation is owned by the linked parity plan.
  verdict:
    decision: blocked
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 144.455
    reason: "The exact-result PR head remains about 2.4 times the control wall time and 4.7 times its engine component, so the one-shot parity blocker remains open."
    commit: e8f1bed
---
## Question

Does the PR #51 head restore detached whole-scan performance to the pre-rewrite control?

## Method

The run interleaved release probes from the pre-rewrite control, the PR #51 declared
base, and the PR #51 head on one unchanged real tree.
This record selects the main and PR-head variants.
Every sample produced the same engine digest.

The harness classified the host as uncontrolled.
The size and direction of the gap justify keeping the existing regression bead open,
while a final parity verdict still requires the quiet-host protocol.

## Result

PR #51 remains about 2.4 times the control wall time and 4.7 times the control
engine-component time in this run.
The candidate also remains above the control in CPU and retained resources.

## Decision

Block merge on the existing one-shot parity requirement.
Use the separate attribution experiment to choose the first correctness-preserving
redesign.
