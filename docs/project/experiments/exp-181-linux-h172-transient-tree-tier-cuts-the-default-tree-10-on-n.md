---
title: "Linux H172 transient tree tier cuts the default tree 10% on node-modules-dense"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-181
  title: "Linux H172 transient tree tier cuts the default tree 10% on node-modules-dense"
  date: "2026-09-29"
  hypotheses:
    - H172
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
    control: "a1a4a568 probe: H171 + H175 (accepted engine)"
    candidate: "956659de probe: H172 transient tree tier"
    control_binary:
      name: control
      sha256: ef07e553fa04cc83ac07549ed575303b89ea99d9ed27b88e25e429be12527509
      size_bytes: 3818784
      args: []
    candidate_binary:
      name: h172
      sha256: 882a639f58a20c22081fca23fc02359ce21809c90d91396a8b350910de555b51
      size_bytes: 3846168
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-181/run.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 184754680.5
          candidate_median: 182088289.0
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.11
          change_pct: -1.895
          ci95_low_pct: -5.185
          ci95_high_pct: 2.245
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 77887539.5
          candidate_median: 75880959.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.145
          change_pct: -1.9
          ci95_low_pct: -6.019
          ci95_high_pct: 2.378
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 370697000.0
          candidate_median: 365303500.0
          control_p95_over_median: 1.08
          candidate_p95_over_median: 1.102
          change_pct: -1.972
          ci95_low_pct: -3.41
          ci95_high_pct: 3.061
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 189374500.0
          candidate_median: 184212000.0
          control_p95_over_median: 1.221
          candidate_p95_over_median: 1.127
          change_pct: -3.804
          ci95_low_pct: -13.146
          ci95_high_pct: 3.084
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 177627500.0
          candidate_median: 183609500.0
          control_p95_over_median: 1.134
          candidate_p95_over_median: 1.237
          change_pct: 0.075
          ci95_low_pct: -3.124
          ci95_high_pct: 4.628
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 30072832.0
          candidate_median: 29958144.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.049
          change_pct: -0.748
          ci95_low_pct: -1.081
          ci95_high_pct: 1.271
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
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 85297479.0
          candidate_median: 75428222.0
          control_p95_over_median: 1.09
          candidate_p95_over_median: 1.108
          change_pct: -10.304
          ci95_low_pct: -15.087
          ci95_high_pct: -7.199
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 79630132.5
          candidate_median: 72137545.0
          control_p95_over_median: 1.082
          candidate_p95_over_median: 1.11
          change_pct: -8.287
          ci95_low_pct: -13.265
          ci95_high_pct: -4.658
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 265573000.0
          candidate_median: 253465500.0
          control_p95_over_median: 1.123
          candidate_p95_over_median: 1.081
          change_pct: -4.202
          ci95_low_pct: -8.029
          ci95_high_pct: -1.722
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 85272500.0
          candidate_median: 64314000.0
          control_p95_over_median: 1.251
          candidate_p95_over_median: 1.389
          change_pct: -24.507
          ci95_low_pct: -31.563
          ci95_high_pct: -17.504
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 184235000.0
          candidate_median: 183344500.0
          control_p95_over_median: 1.156
          candidate_p95_over_median: 1.165
          change_pct: 4.989
          ci95_low_pct: -6.761
          ci95_high_pct: 9.449
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
    lines_changed: 1555
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -10.304
    reason: "quiet 20-pair node-modules-dense default-tree -10.30% [-15.09%, -7.20%]; cold-scan-index placebo -1.90% [-5.18%, +2.25%] includes zero"
    commit: 956659de
    kept: candidate
---
## What was predicted

The `node-modules-dense` co-primary of H172 (with H176 and F6e), pre-registered with
exp-180: `default-tree` wall −3% with the interval below zero on this real,
directory-dense tree with no `.gitignore`; placebo `cold-scan-index`.

## What was measured

Quiet, 20 pairs, no invalid samples.
Control: the accepted H171+H175 engine (`a1a4a568` probe).
Candidate: `956659de`.

| Job | Control | H172 | Change |
| --- | ---: | ---: | --- |
| `default-tree` | 85.3 ms | 75.4 ms | **−10.30% [−15.09%, −7.20%]** |
| `cold-scan-index` (placebo) | 184.8 ms | 182.1 ms | −1.90% [−5.18%, +2.25%] |

Answers were identical to the base, as recorded in exp-180.

## Decision

Accepted with exp-180. At Q0 fdu’s default command trailed pdu’s default by 11% on this
subject (exp-176); the standing against the peers is re-measured at the end of the
night.
