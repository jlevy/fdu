---
title: "Linux H175 derived control chains take another 3% off the default tree"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-179
  title: "Linux H175 derived control chains take another 3% off the default tree"
  date: "2026-09-29"
  hypotheses:
    - H175
  subject:
    tree_label: linux-v6.12
    tree_root_id: 4ffe9d749fe638cd6c746668f7ae66d88e50225fa6903eabd8c4b9405b1aa245
    tree_engine_digest: 55a09ea65f48d38d88d7e2eacde95fa81d0a9970baabfeeba6f9e9418f478704
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293209
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v49
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: "7c69e88a probe: H171"
    candidate: "a1a4a568 probe: H171 + H175 derived chains"
    control_binary:
      name: h171
      sha256: 24482043e6d2a6c057ff479a613347c963d0f9d9627b55a57d12f2849fb1e523
      size_bytes: 3815504
      args: []
    candidate_binary:
      name: h175
      sha256: ef07e553fa04cc83ac07549ed575303b89ea99d9ed27b88e25e429be12527509
      size_bytes: 3818784
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-179/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 115529640.0
          candidate_median: 116395064.0
          control_p95_over_median: 1.093
          candidate_p95_over_median: 1.188
          change_pct: 1.827
          ci95_low_pct: -0.643
          ci95_high_pct: 4.394
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 112500329.0
          candidate_median: 113258665.0
          control_p95_over_median: 1.09
          candidate_p95_over_median: 1.139
          change_pct: 1.899
          ci95_low_pct: -0.618
          ci95_high_pct: 4.223
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 425727000.0
          candidate_median: 428508000.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.097
          change_pct: 1.108
          ci95_low_pct: -1.001
          ci95_high_pct: 3.189
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 130379000.0
          candidate_median: 138750500.0
          control_p95_over_median: 1.179
          candidate_p95_over_median: 1.213
          change_pct: 4.642
          ci95_low_pct: -2.087
          ci95_high_pct: 19.905
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 292838500.0
          candidate_median: 293096500.0
          control_p95_over_median: 1.154
          candidate_p95_over_median: 1.105
          change_pct: -1.08
          ci95_low_pct: -4.397
          ci95_high_pct: 1.79
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
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
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 130170716.5
          candidate_median: 125954081.0
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.055
          change_pct: -3.312
          ci95_low_pct: -7.428
          ci95_high_pct: -0.84
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 124920369.5
          candidate_median: 120180167.5
          control_p95_over_median: 1.07
          candidate_p95_over_median: 1.055
          change_pct: -4.053
          ci95_low_pct: -8.171
          ci95_high_pct: -0.991
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 423526500.0
          candidate_median: 414229000.0
          control_p95_over_median: 1.047
          candidate_p95_over_median: 1.06
          change_pct: -2.086
          ci95_low_pct: -5.212
          ci95_high_pct: 1.664
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 119730500.0
          candidate_median: 109666000.0
          control_p95_over_median: 1.247
          candidate_p95_over_median: 1.178
          change_pct: -9.991
          ci95_low_pct: -20.994
          ci95_high_pct: -1.307
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 307009500.0
          candidate_median: 304901500.0
          control_p95_over_median: 1.059
          candidate_p95_over_median: 1.127
          change_pct: 0.912
          ci95_low_pct: -4.093
          ci95_high_pct: 3.55
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 32401408.0
          candidate_median: 31014912.0
          control_p95_over_median: 1.039
          candidate_p95_over_median: 1.059
          change_pct: -2.151
          ci95_low_pct: -4.436
          ci95_high_pct: -0.124
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: superior
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
    lines_changed: 141
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -3.312
    reason: "quiet 20-pair linux-v6.12 default-tree -3.31% [-7.43%, -0.84%] stacked on H171, as predicted; aggregate-summary +1.83% [-0.64%, +4.39%] unchanged as predicted; --no-controls placebo includes zero; answers identical"
    commit: a1a4a568
    kept: candidate
---
## What was predicted

H175, from the overnight plan’s review (amendment 3): the detached builder keeps each
directory’s control chain beside its id, and a listing’s chain is its parent’s, plus its
own source when the listing carries an admitted control.
`ControlTable::chain_for`, a `BTreeMap<PathBuf>` probe per ancestor with component-wise
ordering, then runs only for control-bearing directories.
It was pre-registered before any timed sample (`f55d501c`): measured stacked on H171 in
the same cell at 20 pairs; prediction −2% to −4% on `default-tree`, controls on,
`linux-v6.12`; placebo both arms `--no-controls`. The summary route keeps its per-parent
cache, so `aggregate-summary` is expected unchanged.

## What was measured

The quiet five-arm cell of exp-178 (20 pairs, no invalid samples), comparing the H175
build (`a1a4a568`) with the H171 build (`7c69e88a`):

| Job | H171 | H171 + H175 | Change |
| --- | ---: | ---: | --- |
| `default-tree` | 130.2 ms | 126.0 ms | **−3.31% [−7.43%, −0.84%]** |
| `aggregate-summary` | 115.5 ms | 116.4 ms | +1.83% [−0.64%, +4.39%] |

- Placebo, both arms `--no-controls` against the base: `default-tree` +0.02%
  [−0.90%, +2.49%], includes zero.
- Answers: the product command line’s output over the three subjects was byte-identical
  to the base, as was the git differential over `linux-v6.12` and the built overlay.
  A seeded test compares derived chains with `chain_for` over random trees whose
  controls are applied, shared, refused or absent.
  In debug builds, every listing’s derived chain is asserted equal to `chain_for`.

## Decision

Accepted, at the margin: −3.31% with the interval below zero at 20 pairs, as predicted,
and the placebo in the same cell includes zero.
The summary route is unchanged, as predicted.
About 130 lines.
