---
title: Bound FullIndex scan-diagnostics overhead
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-090
  title: Bound FullIndex scan-diagnostics overhead
  date: "2026-09-01"
  hypotheses:
    - H97
  subject:
    tree_label: metabrowser-live
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 58bc2ea1deb0e212c7368177328184d412a1b2da24be8a77a7985a4bf6d4bc64
    tree_provenance: Clean live metabrowser checkout at git revision 2d920d60fe3dfc0e17a4fd2cafa08292e60b3de4; exact filesystem metadata and tree shape are not reconstructible.
    tree_reconstructible: false
    tree_entries: 113794
    tree_directories: 15221
    tree_files: 98525
    tree_symlinks: 48
    tree_apparent_bytes: 2311017461
    tree_allocated_bytes: 2591035392
    tree_max_depth: 21
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
    warmups: 2
    interleaved: true
    control: same immutable default-tree probe with diagnostics disabled
    candidate: same immutable default-tree probe with diagnostics enabled
    control_binary:
      name: "off"
      sha256: 5cc6cd2902aab95186a4c23542e1675c1357db7a667a93581de391d4f1ca90d6
      size_bytes: 2156752
      args: []
    candidate_binary:
      name: "on"
      sha256: 5cc6cd2902aab95186a4c23542e1675c1357db7a667a93581de391d4f1ca90d6
      size_bytes: 2156752
      args:
        - "--diagnostics"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-streaming-parity/results/run-h108-fullindex-diagnostics-overhead.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 377430208.5
          candidate_median: 360675416.5
          control_p95_over_median: 1.369
          candidate_p95_over_median: 1.171
          change_pct: -3.476
          ci95_low_pct: -11.878
          ci95_high_pct: 1.428
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 372463958.5
          candidate_median: 355309937.5
          control_p95_over_median: 1.373
          candidate_p95_over_median: 1.175
          change_pct: -3.633
          ci95_low_pct: -12.052
          ci95_high_pct: 1.337
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2163563000.0
          candidate_median: 2053023000.0
          control_p95_over_median: 1.369
          candidate_p95_over_median: 1.201
          change_pct: -3.164
          ci95_low_pct: -14.634
          ci95_high_pct: 1.927
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 211529500.0
          candidate_median: 216353500.0
          control_p95_over_median: 1.544
          candidate_p95_over_median: 1.458
          change_pct: 0.782
          ci95_low_pct: -1.665
          ci95_high_pct: 6.753
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1923060000.0
          candidate_median: 1845523000.0
          control_p95_over_median: 1.39
          candidate_p95_over_median: 1.219
          change_pct: -3.468
          ci95_low_pct: -17.2
          ci95_high_pct: 1.714
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 89513984.0
          candidate_median: 90185728.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.004
          change_pct: 0.236
          ci95_low_pct: -0.794
          ci95_high_pct: 1.095
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: noninferior
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
    lines_changed: 112
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: One private collection flag crosses the report/open boundary; ordinary callers retain the existing no-diagnostics entry point.
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -3.476
    reason: "The opt-in FullIndex trace is noninferior: -3.48% median with a paired 95% interval of [-11.88%, +1.43%], below the predeclared +3% ceiling, with exact tallies and every resource gate held."
    commit: null
---
# Bound FullIndex scan-diagnostics overhead

## Hypothesis

H97: the bounded worker-policy and backend recorder already stayed within the +3%
non-regression margin on the transient Summary plan.
Retaining the same trace while the scanner builds a FullIndex should remain inside that
margin because the recorder observes the shared producer and does not copy index
entries.

## What was tried

The opt-in report entry point now carries one private collection flag through the
FullIndex open path.
A cold scan selects `scan_into_index_with_diagnostics`; the ordinary entry point still
selects `scan_into_index` and returns no diagnostic value.
The installed command keeps the existing `FDU_SCAN_DIAGNOSTICS=1` transport and emits
the same `fdu-scan-diagnostics-v1` document.

The real-tree probe was extended to retain the FullIndex trace for its `default-tree`
job. The measurement interleaved one immutable binary with `--diagnostics` absent and
present, so code generation, dependencies, and source revision were identical between
the two arms. Both arms retained the existing independent tallies oracle.

## What the numbers said

Across 12 pairs on the 113,794-entry metabrowser subject, diagnostics-on changed wall
time by -3.48%, with a paired 95% interval from -11.88% to +1.43%. The interval does not
establish a speedup, but its upper bound is below the predeclared +3% overhead ceiling.
The harness classified the candidate as noninferior, and every CPU, memory, fault, and
context-switch gate held.

Every measured and warm-up diagnostics-on sample carried a complete versioned trace;
every diagnostics-off sample carried none.
Both arms reported the same files, directories, apparent bytes, allocated bytes, and
newest-file timestamp on every sample, with no invalid samples, tree mutation, or
baseline drift.

## Verdict

**ACCEPTED.** The FullIndex trace is available where field diagnosis needs it and stays
inside its explicit overhead budget.
Cache-only opens have no scan to trace, and warm reconciliation does not use the
cold-scan producer contract; those paths continue to return no scan diagnostic rather
than emitting a misleading partial document.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
