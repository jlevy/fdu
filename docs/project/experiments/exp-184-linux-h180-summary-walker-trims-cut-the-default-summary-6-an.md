---
title: "Linux H180 summary walker trims cut the default summary 6% and the blind summary 13% on linux-v6.12"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-184
  title: "Linux H180 summary walker trims cut the default summary 6% and the blind summary 13% on linux-v6.12"
  date: "2026-09-29"
  hypotheses:
    - H180
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
    control: "956659de probe: H172 head"
    candidate: "70c2725c probe: H180 summary walker trims"
    control_binary:
      name: control
      sha256: 882a639f58a20c22081fca23fc02359ce21809c90d91396a8b350910de555b51
      size_bytes: 3846168
      args: []
    candidate_binary:
      name: h180
      sha256: 9430e7379bb91acefc51c26a4ee3b1491a976beeb3a4dd9a0320e6223b3cc670
      size_bytes: 3851832
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-184/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 90462371.5
          candidate_median: 83108095.0
          control_p95_over_median: 1.095
          candidate_p95_over_median: 1.102
          change_pct: -5.635
          ci95_low_pct: -15.149
          ci95_high_pct: -2.754
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 87141779.5
          candidate_median: 79257789.0
          control_p95_over_median: 1.093
          candidate_p95_over_median: 1.101
          change_pct: -6.831
          ci95_low_pct: -15.757
          ci95_high_pct: -2.708
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 309763000.0
          candidate_median: 281746500.0
          control_p95_over_median: 1.129
          candidate_p95_over_median: 1.124
          change_pct: -8.628
          ci95_low_pct: -13.117
          ci95_high_pct: -6.266
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 136119000.0
          candidate_median: 112636500.0
          control_p95_over_median: 1.184
          candidate_p95_over_median: 1.155
          change_pct: -16.786
          ci95_low_pct: -24.837
          ci95_high_pct: -10.369
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 179752500.0
          candidate_median: 174967000.0
          control_p95_over_median: 1.141
          candidate_p95_over_median: 1.056
          change_pct: 0.813
          ci95_low_pct: -9.903
          ci95_high_pct: 6.951
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
          control_median: 78571028.5
          candidate_median: 80750524.0
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.143
          change_pct: 3.294
          ci95_low_pct: -2.628
          ci95_high_pct: 10.521
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 75652127.5
          candidate_median: 77135045.0
          control_p95_over_median: 1.088
          candidate_p95_over_median: 1.14
          change_pct: 2.975
          ci95_low_pct: -3.012
          ci95_high_pct: 11.021
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 272907500.0
          candidate_median: 275927000.0
          control_p95_over_median: 1.095
          candidate_p95_over_median: 1.114
          change_pct: 1.362
          ci95_low_pct: -2.228
          ci95_high_pct: 7.009
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 90368500.0
          candidate_median: 95236500.0
          control_p95_over_median: 1.381
          candidate_p95_over_median: 1.262
          change_pct: 5.601
          ci95_low_pct: -11.66
          ci95_high_pct: 16.844
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 185650500.0
          candidate_median: 192556000.0
          control_p95_over_median: 1.066
          candidate_p95_over_median: 1.097
          change_pct: -0.911
          ci95_low_pct: -7.877
          ci95_high_pct: 9.047
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
  reference_tools: []
  complexity:
    lines_changed: 125
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -5.635
    reason: "quiet 20-pair linux-v6.12 aggregate-summary -5.63% [-15.15%, -2.75%], --no-controls -12.76% [-15.71%, -10.09%]; default-tree placebos include zero"
    commit: 70c2725c
    kept: candidate
---
## What was predicted

The `linux-v6.12` cell of H180, pre-registered with exp-183: `aggregate-summary` and
`aggregate-summary --no-controls` at 20 pairs, `default-tree` as the placebo.

## What was measured

Quiet, 20 pairs, no invalid samples.
The first attempt was refused at the harness’s start gate (CPU busy 26.4%, an
implementation agent’s unguarded `strace`) and rerun whole.
Control: the H172 head (`956659de` probe).
Candidate: `70c2725c`.

| Job | Control | H180 | Change |
| --- | ---: | ---: | --- |
| `aggregate-summary` | 90.5 ms | 83.1 ms | **−5.63% [−15.15%, −2.75%]** |
| `aggregate-summary --no-controls` | 74.7 ms | 64.9 ms | **−12.76% [−15.71%, −10.09%]** |
| `default-tree` (placebo) | 78.6 ms | 80.8 ms | +3.29% [−2.63%, +10.52%] |
| `default-tree --no-controls` (placebo) | 72.8 ms | 70.9 ms | −0.39% [−5.30%, +5.44%] |

## Decision

Accepted.
The default summary is 5.6% faster on the source tree, and with ignore handling
off the summary route is 12.8% faster, where the per-entry path clone is a larger share
of the walker’s work.
Both placebos include zero.
