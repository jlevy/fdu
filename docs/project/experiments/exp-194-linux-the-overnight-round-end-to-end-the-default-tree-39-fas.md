---
title: "Linux: the overnight round end to end, the default tree 39% faster on linux-v6.12"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-194
  title: "Linux: the overnight round end to end, the default tree 39% faster on linux-v6.12"
  date: "2026-09-29"
  hypotheses: []
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
    run_artifact: docs/project/experiments/evidence/exp-194/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 156471621.0
          candidate_median: 115965481.5
          control_p95_over_median: 1.346
          candidate_p95_over_median: 1.049
          change_pct: -26.249
          ci95_low_pct: -39.364
          ci95_high_pct: -25.217
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 153206498.5
          candidate_median: 112881959.0
          control_p95_over_median: 1.347
          candidate_p95_over_median: 1.052
          change_pct: -26.566
          ci95_low_pct: -39.585
          ci95_high_pct: -25.676
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 579538500.0
          candidate_median: 439278000.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.051
          change_pct: -24.899
          ci95_low_pct: -25.732
          ci95_high_pct: -23.406
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 236478000.0
          candidate_median: 77262000.0
          control_p95_over_median: 1.169
          candidate_p95_over_median: 1.254
          change_pct: -68.702
          ci95_low_pct: -72.82
          ci95_high_pct: -64.15
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 353055000.0
          candidate_median: 360817500.0
          control_p95_over_median: 1.101
          candidate_p95_over_median: 1.121
          change_pct: 2.932
          ci95_low_pct: -0.116
          ci95_high_pct: 7.168
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: inconclusive
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
          control_median: 200301724.5
          candidate_median: 119791905.5
          control_p95_over_median: 1.245
          candidate_p95_over_median: 1.102
          change_pct: -39.003
          ci95_low_pct: -42.993
          ci95_high_pct: -34.928
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 194999690.0
          candidate_median: 117079329.0
          control_p95_over_median: 1.252
          candidate_p95_over_median: 1.101
          change_pct: -38.806
          ci95_low_pct: -42.863
          ci95_high_pct: -34.828
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 586916000.0
          candidate_median: 443527500.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.076
          change_pct: -24.397
          ci95_low_pct: -26.178
          ci95_high_pct: -23.071
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 213876000.0
          candidate_median: 66314000.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.159
          change_pct: -68.666
          ci95_low_pct: -72.472
          ci95_high_pct: -66.105
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 377171000.0
          candidate_median: 378317500.0
          control_p95_over_median: 1.082
          candidate_p95_over_median: 1.081
          change_pct: 0.254
          ci95_low_pct: -4.261
          ci95_high_pct: 4.506
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
          - "voluntary_context_switches exceeds its +50% regression limit"
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
          voluntary_context_switches: rejected
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
    change_pct: -39.003
    reason: "Confirms the round's accepted changes in one paired cell against the Q0 engine; no decision rests on it."
    commit: ebc06c78
    kept: neither
---
## What was predicted

The end-to-end confirmation of the 2026-09-29 overnight round, pre-registered in
[the plan’s Status table](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md#status)
(`78980e4a`) before any sample.
One paired cell compares the whole round at once:
- **Control:** the Q0 engine, the `e5a71c8a` probe, which exp-175 measured.
- **Candidate:** the final head, the `ebc06c78` probe.
  It carries H171, H175, H172 with H176 and F6e, H180, H169 phase 1, and H183.
- **Jobs:** `default-tree` and `aggregate-summary`, each with and without `.gitignore`
  handling, at 20 quiet pairs.
- **Prediction:** the accepted paired medians compounded give −40% to −55% on the
  `linux-v6.12` default tree and −30% to −45% on its default summary.

It is a baseline record: no decision rests on it, and a result outside the ranges is
reported as found. The `node-modules-dense` leg is exp-195.

## What was measured

Quiet, 20 pairs, no invalid samples.

| Job | Q0 engine | Final head | Change |
| --- | ---: | ---: | --- |
| `default-tree` | 200.3 ms | 119.8 ms | **−39.00% [−42.99%, −34.93%]** |
| `aggregate-summary` | 156.5 ms | 116.0 ms | −26.25% [−39.36%, −25.22%] |
| `default-tree --no-controls` | 132.1 ms | 117.9 ms | −9.10% [−12.66%, −7.68%] |
| `aggregate-summary --no-controls` | 117.6 ms | 104.4 ms | −10.21% [−13.69%, −8.18%] |

What `.gitignore` handling costs the default tree fell from 52% of the blind walk (200.3
against 132.1 ms) to 1.6% (119.8 against 117.9 ms).

On `linux-balanced-1m`, a screen at 12 pairs, the default tree fell 9.08%
[−9.80%, −7.86%] (1,173.8 to 1,082.8 ms) and the default summary 19.80%
[−21.93%, −16.93%] (1,081.8 to 873.4 ms).
Peak RSS on the tree fell from 292 to 62 MiB.

The host ran in its slower regime in this cell: the Q0 default tree took 200.3 ms here
against 181.9 ms in exp-175, and the tool standing measured beside it put every tool
near 0.12 s, with 85–88% of each tool’s CPU in the kernel.
Kernel time that no change touched is a larger share of wall in this regime, which
dilutes every relative saving.
Both medians fell just short of their predicted ranges: the tree by one point, and the
summary by four.

**Answers.** The product command line at the final head matched the H183 build byte for
byte in 54 comparisons over the three subjects.
Each accepted change had matched its own control in its own record.

**Supplementary runs.** Beside the primary artifact, gzipped:
- `run-balanced.json.gz`: the `linux-balanced-1m` screen, 12 pairs;
- `run-tools-linux-v6.12.json.gz` and `run-tools-node-modules-dense.json.gz`: the final
  20-pair tool standing against pdu 0.24.0 and diskus 0.9.0.

## Decision

Baseline: this confirms the round, and no decision rests on it.
On `linux-v6.12`, the default tree is 39% faster and the default summary 26% faster than
the Q0 engine in one paired cell.
Almost all that remains of the `.gitignore` cost is gone.
