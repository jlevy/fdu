---
title: Point lookup for public mutation preflight
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-102
  title: Point lookup for public mutation preflight
  date: "2026-09-07"
  hypotheses: []
  subject:
    tree_label: metabrowser-final-parity
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 7ec24464a1b59f63a517e985e888f7d78af5b721f5d0398163ad55c0d1b5b0c5
    tree_provenance: "Live github.com/jlevy/metabrowser working tree with local changes and ignored outputs; exact filesystem shape and metadata are not reconstructible. The measured public mutations use the deterministic probe-generated 100001-operation fixture, not a filesystem scan."
    tree_reconstructible: false
    tree_entries: 97587
    tree_directories: 5679
    tree_files: 91897
    tree_symlinks: 11
    tree_apparent_bytes: 949087546
    tree_allocated_bytes: 1179123712
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
    trials: 12
    warmups: 3
    interleaved: true
    control: 64c6e61 exact public preflight with an owned ordered overlay
    candidate: ad52469 standard hash-map overlay with unchanged ownership and contracts
    control_binary:
      name: control
      sha256: d3c34cca3ce3b63a42ca72bfdd9f0d8e259bf84c5e59424ad3c0b15927eafc41
      size_bytes: 2288976
      args: []
    candidate_binary:
      name: candidate
      sha256: 9df36f8d32a07ba51b0f4224f345459cb27c02e71f47a428b95810435a4daa6f
      size_bytes: 2288960
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-102/run.json
  results:
    - job: delta-apply-batched
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 560069458.5
          candidate_median: 339018062.5
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.02
          change_pct: -39.754
          ci95_low_pct: -39.909
          ci95_high_pct: -39.048
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 328768229.0
          candidate_median: 106353229.5
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.016
          change_pct: -67.707
          ci95_low_pct: -67.759
          ci95_high_pct: -67.587
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 556554500.0
          candidate_median: 335744500.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.02
          change_pct: -39.817
          ci95_low_pct: -40.136
          ci95_high_pct: -39.313
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 544272000.0
          candidate_median: 324311500.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.024
          change_pct: -40.478
          ci95_low_pct: -40.789
          ci95_high_pct: -40.001
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 12319500.0
          candidate_median: 11834000.0
          control_p95_over_median: 1.157
          candidate_p95_over_median: 1.055
          change_pct: -2.956
          ci95_low_pct: -11.295
          ci95_high_pct: 0.141
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        blocked_ns:
          control_median: 3400062.0
          candidate_median: 3190145.5
          control_p95_over_median: 1.183
          candidate_p95_over_median: 1.226
          change_pct: -10.22
          ci95_low_pct: -15.555
          ci95_high_pct: 12.921
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 99442688.0
          candidate_median: 98385920.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.03
          change_pct: -0.447
          ci95_low_pct: -1.344
          ci95_high_pct: 1.111
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
    - job: delta-apply-large
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 652120271.0
          candidate_median: 325046729.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.022
          change_pct: -49.785
          ci95_low_pct: -50.306
          ci95_high_pct: -49.344
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 419943791.5
          candidate_median: 91865458.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.027
          change_pct: -77.844
          ci95_low_pct: -78.114
          ci95_high_pct: -77.734
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 647368500.0
          candidate_median: 322199000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.02
          change_pct: -49.999
          ci95_low_pct: -50.494
          ci95_high_pct: -49.545
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 631325500.0
          candidate_median: 308353500.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.017
          change_pct: -50.949
          ci95_low_pct: -51.497
          ci95_high_pct: -50.679
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 15511000.0
          candidate_median: 14018000.0
          control_p95_over_median: 1.113
          candidate_p95_over_median: 1.105
          change_pct: -8.419
          ci95_low_pct: -14.858
          ci95_high_pct: -2.094
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        blocked_ns:
          control_median: 3453791.5
          candidate_median: 2959125.0
          control_p95_over_median: 1.566
          candidate_p95_over_median: 1.197
          change_pct: -13.356
          ci95_low_pct: -23.571
          ci95_high_pct: -3.219
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 133521408.0
          candidate_median: 130220032.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.016
          change_pct: -2.454
          ci95_low_pct: -4.08
          ci95_high_pct: -0.516
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
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
  reference_tools:
    - name: dust
      wall_ns_median: 193939833.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 129
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "One private container substitution, two explanatory lines, and 125 lines of contract tests; no new dependency, unsafe code, failure mode or caller restriction."
  verdict:
    decision: accepted
    primary_job: delta-apply-large
    primary_metric: wall_ns
    change_pct: -49.785
    reason: "Retain for quiet-host confirmation: fixed twelve-pair exploratory large and repeated public mutations improve wall 49.78% and 39.75%, with both wall/component intervals below zero, exact oracles, and resource ratios within 1.05. Batched allocated bytes rise 4.95%, a recorded tradeoff. This uncontrolled screen does not close final one-shot or opened parity."
    commit: ad52469d7d16fee3135a515fd07a43c5bab8ba11
---
# Point Lookup for Public Mutation Preflight

## Hypothesis and Change

The private `StructuralOverlay` uses its path keys for exact lookup, insertion, and
subtree retention. It exposes no iteration order.
A counter-disabled call tree identified ordered-map lookup and insertion as the main
cost of public ancestry validation.
The predeclared experiment, tracked in `fdu-0q6w`, replaces that temporary `BTreeMap`
with a standard `HashMap`.

The production change is one container substitution and its explanation.
Public operation order, exact commits, ancestry rejection, and control projection are
unchanged. There are no new dependencies, unsafe blocks, bounds, or caller requirements.

Before changing storage, three new tests checked mixed structural batches against the
independent model, atomic rejection after removing and recreating an ancestor, and
pruning both retained and transient controls after kind changes.
An injected omitted-pruning mutant failed the new atomicity test; the mutant was then
removed.
The candidate passed those tests, the full `make check` gate, and cross-platform
lint.

## Experiment

The release control is `64c6e61`; the candidate is `ad52469`. Both enable `gitignore`,
and the provenance manifest verifies each immutable binary against its own clean source
revision.
The fixed schedule contains twelve interleaved measured pairs and three warmups
for each job. This is a warm-steady, uncontrolled, exploratory run on Apple M1 Pro/APFS.
No sample, oracle, baseline, or subject-drift check failed.

These jobs apply a deterministic synthetic 100,001-operation observation: once as a
large batch and once in batches of 4,096. The live Metabrowser tree supplies the harness
root and fingerprint guard; the measured mutation does not scan that filesystem.
The results describe public mutation, not filesystem-walk throughput.

| Job | Paired wall change | Paired component change | Allocation-event ratio | Allocated-byte ratio |
| --- | --- | --- | --- | --- |
| Large batch | -49.78% | -77.84% | 0.9822 | 1.0239 |
| Repeated batches | -39.75% | -67.71% | 0.9844 | 1.0495 |

Both wall and component intervals are entirely below zero, as recorded above.
Peak RSS is within the 1.05 ceiling, and reallocation counts are unchanged.
The allocated-byte increase is real: repeated batches are close to the predeclared +5%
ceiling. This change buys cheaper lookup, not lower cumulative allocation volume.
Separate scoped counter runs have identical complete summaries, including final-state
and commit digests, after removing only the implementation-specific counters.

The counter-disabled call trees put ancestry validation at 1,993 of 2,391 inclusive
`Index::apply` samples in the engine-unchanged `1a39be9` profile, versus 345 of 1,192 in
`ad52469`: about 83% versus 29%. These are attribution samples, not elapsed-time ratios.
The post-change profile’s largest whole-process leaf is now `summarize_commits`, a probe
diagnostic outside the component interval.
It must not be charged to the engine.

The [evidence directory](evidence/exp-102/) retains the raw paired run, provenance,
scoped counter outputs, and profile summaries.
The preliminary controls-on default-tree profile is not used as evidence for the
non-watch CLI.

## Decision

Accept the private container change for quiet-host confirmation.
Both predeclared public-mutation screens pass with exact oracles and within the resource
ceilings. The implementation does not add an ordering contract or another engine path.

This exploratory result does not close historical one-shot parity, opened-root
noninferiority, or final-binary readiness.
The original quiet-host gates remain open in `fdu-lj4h`; no busy-host measurement is
substituted for them.
The generic adaptive qualification also remains inconclusive because the all-zero
voluntary-context-switch metric has no percentage interval; this experiment changes no
adaptive scheduling policy.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
