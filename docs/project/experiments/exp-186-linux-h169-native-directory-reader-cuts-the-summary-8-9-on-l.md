---
title: "Linux H169 native directory reader cuts the summary 8-9% on linux-v6.12; the tree does not clear"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-186
  title: "Linux H169 native directory reader cuts the summary 8-9% on linux-v6.12; the tree does not clear"
  date: "2026-09-29"
  hypotheses:
    - H169
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
    control: "70c2725c probe: H180 head"
    candidate: "20933081 probe: H169 Linux-native reader"
    control_binary:
      name: control-blind
      sha256: 9430e7379bb91acefc51c26a4ee3b1491a976beeb3a4dd9a0320e6223b3cc670
      size_bytes: 3851832
      args:
        - "--no-controls"
    candidate_binary:
      name: h169-blind
      sha256: 1b69d391a91fdb3296d51c97a396d2365731941ab631ef333f03ff0a016882b8
      size_bytes: 3861144
      args:
        - "--no-controls"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-186/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 63843342.0
          candidate_median: 59998718.5
          control_p95_over_median: 1.274
          candidate_p95_over_median: 1.116
          change_pct: -7.899
          ci95_low_pct: -9.553
          ci95_high_pct: -4.741
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 61006225.0
          candidate_median: 57288554.5
          control_p95_over_median: 1.279
          candidate_p95_over_median: 1.118
          change_pct: -7.438
          ci95_low_pct: -10.187
          ci95_high_pct: -5.179
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 236469500.0
          candidate_median: 216114000.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.137
          change_pct: -7.801
          ci95_low_pct: -10.358
          ci95_high_pct: -6.309
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 60687000.0
          candidate_median: 43922500.0
          control_p95_over_median: 1.137
          candidate_p95_over_median: 1.246
          change_pct: -24.643
          ci95_low_pct: -39.449
          ci95_high_pct: -11.783
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 175123000.0
          candidate_median: 179781000.0
          control_p95_over_median: 1.189
          candidate_p95_over_median: 1.132
          change_pct: -0.069
          ci95_low_pct: -7.007
          ci95_high_pct: 6.767
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
          - "minor_faults exceeds its +10% regression limit"
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
          minor_faults: rejected
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
          control_median: 71095860.0
          candidate_median: 68452385.5
          control_p95_over_median: 1.114
          candidate_p95_over_median: 1.097
          change_pct: -1.791
          ci95_low_pct: -6.832
          ci95_high_pct: 2.656
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 67769388.0
          candidate_median: 65124819.0
          control_p95_over_median: 1.107
          candidate_p95_over_median: 1.108
          change_pct: -1.267
          ci95_low_pct: -6.771
          ci95_high_pct: 2.656
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 250935000.0
          candidate_median: 244028000.0
          control_p95_over_median: 1.079
          candidate_p95_over_median: 1.099
          change_pct: -0.691
          ci95_low_pct: -3.824
          ci95_high_pct: 3.128
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 62876500.0
          candidate_median: 54011000.0
          control_p95_over_median: 1.47
          candidate_p95_over_median: 1.478
          change_pct: -10.879
          ci95_low_pct: -35.35
          ci95_high_pct: -3.97
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 186218500.0
          candidate_median: 192162500.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.109
          change_pct: 2.783
          ci95_low_pct: -1.092
          ci95_high_pct: 12.949
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
          - "minor_faults exceeds its +10% regression limit"
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
          minor_faults: rejected
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 1621
    new_dependencies: []
    new_unsafe_blocks: 4
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -7.899
    reason: "quiet 20-pair linux-v6.12 aggregate-summary --no-controls -7.90% [-9.55%, -4.74%], narrowly short of the predicted -8% to -12% but clearing the rule; default summary -9.39%; default-tree with controls on -2.06% [-7.38%, +1.83%] does not clear (co-secondary), nor does the --no-controls pair recorded here, -1.79%; serial-portable placebo -0.29% includes zero"
    commit: "20933081"
    kept: candidate
---
## What was predicted

The `linux-v6.12` cell of H169 phase 1, pre-registered with exp-185: deciding
`aggregate-summary --no-controls` at 20 pairs, predicted −8% to −12%; co-secondary
`default-tree`, predicted −6% to −8%; placebo
`aggregate-summary --no-controls --threads 1`.

## What was measured

Quiet, 20 pairs, no invalid samples.
Control: `70c2725c` probe (the H180 engine).
Candidate: `20933081`.

This record’s frontmatter results are the `--no-controls` pair the deciding job used
(`h169-blind_vs_control-blind`): the `aggregate-summary` result is the deciding job, and
the `default-tree` result is `default-tree --no-controls`, −1.79%. The controls-on rows
below come from the run’s controls-on arms (`h169_vs_control` in `run.json`).

| Job | Control | H169 | Change |
| --- | ---: | ---: | --- |
| `aggregate-summary --no-controls` (deciding) | 63.8 ms | 60.0 ms | **−7.90% [−9.55%, −4.74%]** |
| `aggregate-summary` | 83.4 ms | 75.8 ms | −9.39% [−13.55%, −3.71%] |
| `default-tree` (co-secondary) | 79.4 ms | 76.8 ms | −2.06% [−7.38%, +1.83%] |
| `default-tree --no-controls` | 71.1 ms | 68.5 ms | −1.79% [−6.83%, +2.66%] |
| `aggregate-summary --no-controls --threads 1` (placebo) | 193.7 ms | 191.9 ms | −0.29% [−3.76%, +0.43%] |

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-placebo-serial.json.gz`, the `--threads 1` serial placebo.

## Decision

Accepted on the deciding job, with the placebo at zero.
The deciding job’s −7.90% clears the rule and falls a tenth of a point short of its
predicted −8% to −12%. The default summary gains 9.4%. The default tree’s −2.1% with
controls on, and −1.8% without, do not clear the rule on this subject, below the
co-secondary prediction.
The tree route’s remaining cost here is dominated by the kernel’s `statx` and the
consumer, which this reader does not touch.
