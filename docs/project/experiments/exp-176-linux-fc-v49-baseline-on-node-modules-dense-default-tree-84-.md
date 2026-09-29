---
title: "Linux fc-v49 baseline on node-modules-dense: default tree 84 ms, 11% behind pdu"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-176
  title: "Linux fc-v49 baseline on node-modules-dense: default tree 84 ms, 11% behind pdu"
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
    trials: 12
    warmups: 3
    interleaved: true
    control: "e5a71c8a probe (0.2.1 engine + docs), copy A"
    candidate: "e5a71c8a probe, copy B (A/A)"
    control_binary:
      name: control
      sha256: 3ed8fdbfa1af82713b2b828ec0398cfaed3f3eabd14f55b6b738914849bd0d25
      size_bytes: 3758784
      args: []
    candidate_binary:
      name: candidate
      sha256: 3ed8fdbfa1af82713b2b828ec0398cfaed3f3eabd14f55b6b738914849bd0d25
      size_bytes: 3758784
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-176/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 76635522.0
          candidate_median: 74975597.0
          control_p95_over_median: 1.058
          candidate_p95_over_median: 1.087
          change_pct: -0.27
          ci95_low_pct: -5.971
          ci95_high_pct: 2.249
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 73947535.5
          candidate_median: 72251203.5
          control_p95_over_median: 1.049
          candidate_p95_over_median: 1.089
          change_pct: -0.375
          ci95_low_pct: -6.175
          ci95_high_pct: 2.58
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 279440500.0
          candidate_median: 270609500.0
          control_p95_over_median: 1.05
          candidate_p95_over_median: 1.092
          change_pct: 0.061
          ci95_low_pct: -6.561
          ci95_high_pct: 2.652
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 77278000.0
          candidate_median: 65672000.0
          control_p95_over_median: 1.193
          candidate_p95_over_median: 1.428
          change_pct: -13.783
          ci95_low_pct: -39.695
          ci95_high_pct: 25.755
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 203393000.0
          candidate_median: 205123500.0
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.227
          change_pct: 4.837
          ci95_low_pct: -10.876
          ci95_high_pct: 14.841
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
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
          control_median: 84033493.0
          candidate_median: 84507680.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.058
          change_pct: 3.862
          ci95_low_pct: -4.276
          ci95_high_pct: 7.861
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 79045932.0
          candidate_median: 79625297.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.064
          change_pct: 3.781
          ci95_low_pct: -3.922
          ci95_high_pct: 8.146
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 274160500.0
          candidate_median: 272283500.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.069
          change_pct: 2.618
          ci95_low_pct: -4.894
          ci95_high_pct: 6.772
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 85809000.0
          candidate_median: 78379000.0
          control_p95_over_median: 1.142
          candidate_p95_over_median: 1.251
          change_pct: -10.372
          ci95_low_pct: -20.535
          ci95_high_pct: 25.339
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 190863500.0
          candidate_median: 195548000.0
          control_p95_over_median: 1.066
          candidate_p95_over_median: 1.084
          change_pct: 4.494
          ci95_low_pct: -3.08
          ci95_high_pct: 8.863
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 30982144.0
          candidate_median: 30656512.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.017
          change_pct: -1.58
          ci95_low_pct: -2.901
          ci95_high_pct: 1.018
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 3.862
    reason: "quiet four-arm A/A of e5a71c8a on node-modules-dense: default-tree 84.0 ms, aggregate-summary 76.6 ms; A/A +3.86% [-4.28%, +7.86%] and -0.27% [-5.97%, +2.25%] include zero"
    commit: null
    kept: control
---
## What was predicted

The overnight loop’s Q0 baseline on `node-modules-dense`, the real directory-dense
subject H159 was accepted on (exp-190), rebuilt on this host from its committed
manifests. As in exp-175, four arms of `e5a71c8a`, with `control` and `candidate` copies
of the same probe: every same-binary comparison should include zero.
The subject has no `.gitignore`, so the `--no-controls` arms should equal the default
arms.

## What was measured

Quiet regime, 3 warmups, 12 pairs, no invalid samples.
The rebuilt tree has 79,957 entries against exp-190’s 79,953, with the same 9,439
directories.

| Job | control | candidate (same binary) | `--no-controls` |
| --- | ---: | ---: | ---: |
| `default-tree` | 84.0 ms | 84.5 ms | 85.5 ms |
| `aggregate-summary` | 76.6 ms | 75.0 ms | 71.5 ms |

- A/A: `default-tree` +3.86% [−4.28%, +7.86%]; `aggregate-summary` −0.27%
  [−5.97%, +2.25%]. Both include zero.
- Controls on against off: `default-tree` +0.65% [−2.61%, +5.79%], as predicted with no
  rules to match.

Quiet tool cell, `fdu-default-tree` contract, 12 pairs, no invalid samples or
mismatches:

| Tool | Wall | Change against the adjacent fdu run |
| --- | ---: | --- |
| `fdu --color never PATH` | 0.086 s | baseline |
| pdu 0.24.0, default | 0.077 s | −11% [−21%, −4%] |
| pdu 0.24.0, `--max-depth 2` | 0.070 s | −20% |
| diskus 0.9.0 | 0.075 s | −12% |

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff reviewable: `run-q0-tools-nmd.json.gz`, the Q0 tool standing on `node-modules-dense`.

## Decision

Baseline. On a dense tree with no ignore rules fdu trails pdu’s default by 11%. That gap
is the walk and the index build, not classification: the default tree costs about 8 ms
more than the summary here.
