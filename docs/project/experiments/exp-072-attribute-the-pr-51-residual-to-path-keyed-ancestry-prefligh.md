---
title: "Attribute the PR #51 residual to path-keyed ancestry preflight"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-072
  title: "Attribute the PR #51 residual to path-keyed ancestry preflight"
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
    control: "PR #51 without detached publication"
    candidate: "diagnostic PR #51 without detached publication or ancestry overlay"
    control_binary:
      name: no-publish
      sha256: 1b6a6558833360e8d83894d104388abd8d0ac23d1296afd8389622e6fc746529
      size_bytes: 1826000
      args: []
    candidate_binary:
      name: no-publish-no-ancestry
      sha256: 26d89317508977d1fb598d8db173a7da4f4b34eb10756e724de71bff15a413d4
      size_bytes: 1826000
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-pr51-review-runs/run-attribution-ladder.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 706822062.5
          candidate_median: 384383270.5
          control_p95_over_median: 1.07
          candidate_p95_over_median: 1.157
          change_pct: -45.473
          ci95_low_pct: -48.29
          ci95_high_pct: -39.429
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 6
        component_ns:
          control_median: 480484896.0
          candidate_median: 160526604.5
          control_p95_over_median: 1.072
          candidate_p95_over_median: 1.204
          change_pct: -66.653
          ci95_low_pct: -68.854
          ci95_high_pct: -61.364
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 6
        cpu_ns:
          control_median: 1283354000.0
          candidate_median: 949871000.0
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.041
          change_pct: -25.295
          ci95_low_pct: -29.537
          ci95_high_pct: -21.129
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 6
        user_cpu_ns:
          control_median: 691634500.0
          candidate_median: 374146500.0
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.075
          change_pct: -45.941
          ci95_low_pct: -47.012
          ci95_high_pct: -42.913
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 6
        system_cpu_ns:
          control_median: 579912000.0
          candidate_median: 579062000.0
          control_p95_over_median: 1.2
          candidate_p95_over_median: 1.062
          change_pct: 0.909
          ci95_low_pct: -14.919
          ci95_high_pct: 11.078
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 6
        peak_rss_bytes:
          control_median: 102383616.0
          candidate_median: 87195648.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.021
          change_pct: -14.214
          ci95_low_pct: -16.098
          ci95_high_pct: -12.709
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - The candidate omits ancestry validation and is evidence about cost only; applying it directly would weaken atomic correctness.
    notes: Disposable attribution variants only; the plan requires one private resolved-parent proof and no second reducer.
  verdict:
    decision: blocked
    primary_job: cold-scan-index
    primary_metric: component_ns
    change_pct: -66.653
    reason: "Removing the path-keyed ancestry overlay closes most of the remaining engine gap, but the diagnostic candidate cannot ship until scanner preparation supplies an equivalent atomic proof."
    commit: e8f1bed
---
## Question

Which part of the residual commit pipeline accounts for the engine time after PR #51?

## Method

Disposable release probes removed one consequence at a time from the same PR #51 head:
the compatibility projection, detached publication, path-keyed ancestry preflight,
prepared-path copying, and remaining live bookkeeping.
The harness interleaved all variants on the same unchanged real tree and checked the
same engine digest after every sample.

The no-ancestry candidate intentionally removes a correctness check without replacing
its proof. It is diagnostic code and cannot ship.

## Result

Removing the compatibility projection alone did not materially change component time.
Skipping detached publication reduced it, and removing the path-keyed ancestry overlay
closed most of the remaining gap.
Moving scanner-owned paths and removing unused live bookkeeping reduced the residual
further. Separate allocation counters assigned most of the remaining allocation gap to
prepare, effect, and compatibility path ownership.

## Decision

Treat ancestry preflight as the leading CPU mechanism.
Implementation remains blocked on a private scanner batch that proves canonical paths
and resolved parents before mutation.
The public arbitrary-batch path keeps atomic validation.
