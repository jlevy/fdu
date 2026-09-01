---
title: "PR #51 halves its base regression but does not restore main parity"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-071
  title: "PR #51 halves its base regression but does not restore main parity"
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
    control: "PR #51 base at 954a959"
    candidate: "PR #51 head at e8f1bed"
    control_binary:
      name: pr-base
      sha256: 1bebcb1c0d2fc36e6d96c128facad54bab2a533cba4e907b708b5276509298a5
      size_bytes: 1809488
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
          control_median: 1698578395.5
          candidate_median: 847697812.5
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.16
          change_pct: -49.545
          ci95_low_pct: -51.436
          ci95_high_pct: -43.363
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 6
        component_ns:
          control_median: 1477819666.5
          candidate_median: 627993187.5
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.186
          change_pct: -56.889
          ci95_low_pct: -59.098
          ci95_high_pct: -50.735
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 6
        cpu_ns:
          control_median: 2269800000.0
          candidate_median: 1419174000.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.016
          change_pct: -37.047
          ci95_low_pct: -38.178
          ci95_high_pct: -36.819
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 6
        user_cpu_ns:
          control_median: 1676795000.0
          candidate_median: 841823000.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.055
          change_pct: -49.61
          ci95_low_pct: -50.627
          ci95_high_pct: -47.277
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 6
        system_cpu_ns:
          control_median: 593006000.0
          candidate_median: 576405500.0
          control_p95_over_median: 1.028
          candidate_p95_over_median: 1.04
          change_pct: -3.043
          ci95_low_pct: -9.661
          ci95_high_pct: 1.28
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
        peak_rss_bytes:
          control_median: 107962368.0
          candidate_median: 106348544.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.011
          change_pct: -1.296
          ci95_low_pct: -2.285
          ci95_high_pct: 0.379
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 6
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - voluntary_context_switches is missing a paired percent interval
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
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 304
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - The uncontrolled host regime prevents a final parity claim; repeat on a quiet host for acceptance.
    notes: Partial pipeline simplification; no new dependency or unsafe code.
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -49.545
    reason: "The exact-result candidate removes about half of the base cost, but remains above the pre-rewrite control; accept the mechanisms without closing the parity blocker."
    commit: e8f1bed
---
## Question

Does PR #51 remove enough of the opened-root rewrite’s one-shot regression to satisfy
the existing parity requirement?

## Method

The run interleaved release probes from the pre-rewrite control, the PR #51 declared
base, and the PR #51 head on one unchanged real tree.
Every sample produced the same engine digest.
The harness classified the host as uncontrolled, so the run supports attribution and a
merge-blocking regression diagnosis but not a final parity claim.

## Result

PR #51 removes about half of its base’s wall and engine-component cost.
The head remains well above the pre-rewrite control, so the accepted partial fix does
not close `fdu-pro1`. Counter runs held directory enumeration, metadata reads, accepted
upserts, roll-up merges, and the final digest constant while allocation work diverged.

## Decision

Accept the PR #51 mechanisms as partial improvements.
Keep one-shot parity blocked and profile the residual before selecting the next change.
