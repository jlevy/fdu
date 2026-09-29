---
title: "Linux: the overnight round end to end, the default tree 10% faster on node-modules-dense"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-195
  title: "Linux: the overnight round end to end, the default tree 10% faster on node-modules-dense"
  date: "2026-09-29"
  hypotheses: []
  subject:
    tree_label: node-modules-dense
    tree_root_id: db6d49b68990d6b3a741e6289f360187d4317abfdda8be3f70915f799eb7acd4
    tree_engine_digest: 11c4122bec53693180e2e240899b75cdccc707ca9759b9acfadb80a0b19bafac
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.2, npm 10.9.7); rebuilt 2026-09-29 on this host: 79,957 entries (exp-190 recorded 79,953), 9,439 directories, no .gitignore."
    tree_reconstructible: true
    tree_entries: 79957
    tree_directories: 9439
    tree_files: 70411
    tree_symlinks: 107
    tree_apparent_bytes: 538992241
    tree_allocated_bytes: 757870592
    tree_max_depth: 12
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
    control: e5a71c8a probe (the Q0 engine)
    candidate: ebc06c78 probe (the final head of the round)
    control_binary:
      name: q0
      sha256: 3ed8fdbfa1af82713b2b828ec0398cfaed3f3eabd14f55b6b738914849bd0d25
      size_bytes: 3758784
      args: []
    candidate_binary:
      name: final
      sha256: 938370e0cbdc5885888514aa8ec2caa854b151cddab9c83c11712b0251525e82
      size_bytes: 3862104
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-195/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 109415624.0
          candidate_median: 101084782.0
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.098
          change_pct: -7.016
          ci95_low_pct: -11.255
          ci95_high_pct: -3.831
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 106500127.5
          candidate_median: 98451324.0
          control_p95_over_median: 1.121
          candidate_p95_over_median: 1.095
          change_pct: -7.016
          ci95_low_pct: -11.31
          ci95_high_pct: -3.665
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 411093500.0
          candidate_median: 377412500.0
          control_p95_over_median: 1.103
          candidate_p95_over_median: 1.099
          change_pct: -6.887
          ci95_low_pct: -11.168
          ci95_high_pct: -3.424
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 66571000.0
          candidate_median: 50242500.0
          control_p95_over_median: 1.357
          candidate_p95_over_median: 1.219
          change_pct: -31.489
          ci95_low_pct: -41.686
          ci95_high_pct: -20.279
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 349742000.0
          candidate_median: 330046000.0
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.159
          change_pct: -4.145
          ci95_low_pct: -7.111
          ci95_high_pct: -1.169
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "minor_faults straddles its +10% regression limit"
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
          minor_faults: inconclusive
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
          control_median: 126933756.5
          candidate_median: 114228512.5
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.099
          change_pct: -9.75
          ci95_low_pct: -11.84
          ci95_high_pct: -7.328
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 122690348.5
          candidate_median: 111406357.0
          control_p95_over_median: 1.063
          candidate_p95_over_median: 1.101
          change_pct: -8.439
          ci95_low_pct: -10.532
          ci95_high_pct: -6.237
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 432517500.0
          candidate_median: 405500500.0
          control_p95_over_median: 1.075
          candidate_p95_over_median: 1.034
          change_pct: -6.356
          ci95_low_pct: -8.806
          ci95_high_pct: -3.479
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 84017500.0
          candidate_median: 59174000.0
          control_p95_over_median: 1.35
          candidate_p95_over_median: 1.146
          change_pct: -35.801
          ci95_low_pct: -39.432
          ci95_high_pct: -18.482
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 351318500.0
          candidate_median: 344130500.0
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.062
          change_pct: -1.63
          ci95_low_pct: -3.506
          ci95_high_pct: 3.833
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "voluntary_context_switches straddles its +50% regression limit"
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: rejected
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -9.75
    reason: "Confirms the round's accepted changes on the dense tree in one paired cell against the Q0 engine; no decision rests on it."
    commit: ebc06c78
    kept: neither
---
## What was predicted

The `node-modules-dense` leg of the end-to-end confirmation.
Its linux-v6.12 leg is
[exp-194](exp-194-linux-the-overnight-round-end-to-end-the-default-tree-39-fas.md).
Both were pre-registered in
[the plan’s Status table](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md#status)
(`78980e4a`) before any sample.
- **Control:** the Q0 engine, the `e5a71c8a` probe.
- **Candidate:** the final head, the `ebc06c78` probe.
- **Jobs:** `default-tree` and `aggregate-summary`, each with and without `.gitignore`
  handling, at 20 quiet pairs.
- **Prediction:** the accepted paired medians compounded give −10% to −20% on the
  default tree and −12% to −22% on the default summary.
  The tree gains come from H172 and H169, the summary gains from H180 and H169. The
  subject has no `.gitignore`, so H171, H175 and H183 do not apply.

It is a baseline record: no decision rests on it, and a result outside the ranges is
reported as found.

## What was measured

Quiet, 20 pairs, no invalid samples.

| Job | Q0 engine | Final head | Change |
| --- | ---: | ---: | --- |
| `default-tree` | 126.9 ms | 114.2 ms | **−9.75% [−11.84%, −7.33%]** |
| `aggregate-summary` | 109.4 ms | 101.1 ms | −7.02% [−11.26%, −3.83%] |
| `default-tree --no-controls` | 124.7 ms | 115.9 ms | −6.91% [−12.96%, −4.33%] |
| `aggregate-summary --no-controls` | 110.1 ms | 101.1 ms | −8.21% [−10.74%, −5.00%] |

The cell ran in the host’s slower regime, as exp-194 did.
The Q0 default tree took 126.9 ms here against 84.0 ms in exp-176. Kernel time that no
change touched is then a larger share of wall, and both medians fell short of their
predicted ranges: the tree by a quarter point, and the summary by five points.

**Answers.** The product command line at the final head matched the H183 build byte for
byte in 54 comparisons over the three subjects.

## Decision

Baseline: this confirms the round on the dense tree, and no decision rests on it.
On `node-modules-dense`, the default tree is 10% faster and the default summary 7%
faster than the Q0 engine in one paired cell.
