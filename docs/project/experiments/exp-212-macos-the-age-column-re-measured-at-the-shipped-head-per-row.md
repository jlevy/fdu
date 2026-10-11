---
title: "macOS: the age column re-measured at the shipped head, per-row retained cost and a quarter microsecond a machine row"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-212
  title: "macOS: the age column re-measured at the shipped head, per-row retained cost and a quarter microsecond a machine row"
  date: "2026-10-09"
  hypotheses:
    - H192
  subject:
    tree_label: rustup
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: 4304d9d4071fd4478a0510edf80c4b78594f98b84969d90a1d7230eb0dc94d78
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains (root_id 36ce9b22). Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 77355
    tree_directories: 3427
    tree_files: 73928
    tree_symlinks: 0
    tree_apparent_bytes: 3750189949
    tree_allocated_bytes: 3978313728
    tree_max_depth: 17
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
    control: "148ef78e probe: main before the age column (sha256 d2ac70ff, the binary exp-209 to exp-211 used; tied by the run variant notes)"
    candidate: "ae90aef4 probe, clean tree: all of #191 code after reviews B and C (1b3ac793, e25f12e3, directory re-reads, C8, C10, C12, C13; sha256 d3254e26; tied by the run variant notes)"
    control_binary:
      name: control
      sha256: d2ac70ff129f6c510100a0f58a27677015fec20af2f8d29b732f6e29e2182041
      size_bytes: 3363840
      args: []
    candidate_binary:
      name: candidate
      sha256: d3254e26ef25b65b075fb4d070927b800e8103fce6a62b86dba284c13f0f4930
      size_bytes: 3380352
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-212/run.json.gz
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 294241333.0
          candidate_median: 291649062.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.052
          change_pct: -0.757
          ci95_low_pct: -3.803
          ci95_high_pct: 2.087
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 125845166.0
          candidate_median: 123678854.5
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.096
          change_pct: -2.597
          ci95_low_pct: -8.274
          ci95_high_pct: 4.357
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 828526500.0
          candidate_median: 802137000.0
          control_p95_over_median: 1.03
          candidate_p95_over_median: 1.052
          change_pct: -3.724
          ci95_low_pct: -4.652
          ci95_high_pct: 2.553
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 221425500.0
          candidate_median: 216493500.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.028
          change_pct: -0.596
          ci95_low_pct: -1.582
          ci95_high_pct: 0.03
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 611870000.0
          candidate_median: 585840500.0
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.06
          change_pct: -4.809
          ci95_low_pct: -6.512
          ci95_high_pct: 3.502
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 27254784.0
          candidate_median: 27729920.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.078
          change_pct: 1.725
          ci95_low_pct: -0.612
          ci95_high_pct: 5.735
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
          - "peak_rss_bytes straddles its +5% regression limit"
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
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 120243229.5
          candidate_median: 121286083.5
          control_p95_over_median: 1.077
          candidate_p95_over_median: 1.127
          change_pct: 2.329
          ci95_low_pct: -2.994
          ci95_high_pct: 4.642
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 115643104.0
          candidate_median: 116852937.5
          control_p95_over_median: 1.079
          candidate_p95_over_median: 1.129
          change_pct: 2.274
          ci95_low_pct: -2.871
          ci95_high_pct: 4.965
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 617382000.0
          candidate_median: 618517500.0
          control_p95_over_median: 1.151
          candidate_p95_over_median: 1.085
          change_pct: 1.31
          ci95_low_pct: -5.428
          ci95_high_pct: 2.974
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 38870500.0
          candidate_median: 37739500.0
          control_p95_over_median: 1.111
          candidate_p95_over_median: 1.194
          change_pct: -0.119
          ci95_low_pct: -1.835
          ci95_high_pct: 3.461
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 577953500.0
          candidate_median: 578932500.0
          control_p95_over_median: 1.156
          candidate_p95_over_median: 1.095
          change_pct: 1.217
          ci95_low_pct: -5.696
          ci95_high_pct: 3.074
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 12615680.0
          candidate_median: 13189120.0
          control_p95_over_median: 1.058
          candidate_p95_over_median: 1.047
          change_pct: 4.896
          ci95_low_pct: 0.324
          ci95_high_pct: 9.992
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes straddles its +5% regression limit"
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
    - job: index-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 292748875.0
          candidate_median: 287434271.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.022
          change_pct: -1.737
          ci95_low_pct: -4.603
          ci95_high_pct: -0.025
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 130375.5
          candidate_median: 129083.5
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.036
          change_pct: 0.21
          ci95_low_pct: -2.359
          ci95_high_pct: 1.606
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 802835500.0
          candidate_median: 780786000.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.049
          change_pct: -2.623
          ci95_low_pct: -5.425
          ci95_high_pct: -1.292
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 213841500.0
          candidate_median: 213161000.0
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.044
          change_pct: -0.394
          ci95_low_pct: -1.801
          ci95_high_pct: 0.339
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 588519500.0
          candidate_median: 563688000.0
          control_p95_over_median: 1.052
          candidate_p95_over_median: 1.079
          change_pct: -3.808
          ci95_low_pct: -6.268
          ci95_high_pct: -0.946
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 28499968.0
          candidate_median: 28270592.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.059
          change_pct: -2.885
          ci95_low_pct: -5.011
          ci95_high_pct: 2.706
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
    - job: opened-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1097090416.5
          candidate_median: 1088954333.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.021
          change_pct: -0.534
          ci95_low_pct: -2.387
          ci95_high_pct: 1.889
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 133958.0
          candidate_median: 151104.5
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.051
          change_pct: 11.489
          ci95_low_pct: 8.42
          ci95_high_pct: 14.225
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1156511000.0
          candidate_median: 1144946000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.024
          change_pct: -0.716
          ci95_low_pct: -2.552
          ci95_high_pct: 2.038
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 828009000.0
          candidate_median: 826033500.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.009
          change_pct: 0.06
          ci95_low_pct: -1.156
          ci95_high_pct: 1.3
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 328398500.0
          candidate_median: 320488500.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.058
          change_pct: -1.03
          ci95_low_pct: -6.985
          ci95_high_pct: 3.61
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 115761152.0
          candidate_median: 115261440.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.009
          change_pct: -0.121
          ci95_low_pct: -0.939
          ci95_high_pct: 0.291
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
    - job: render-json
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 706544562.5
          candidate_median: 740344708.5
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.017
          change_pct: 4.274
          ci95_low_pct: 3.36
          ci95_high_pct: 6.475
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        component_ns:
          control_median: 128840770.5
          candidate_median: 163173187.5
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.009
          change_pct: 26.603
          ci95_low_pct: 26.467
          ci95_high_pct: 27.11
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1207798500.0
          candidate_median: 1261147500.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.021
          change_pct: 4.311
          ci95_low_pct: 2.44
          ci95_high_pct: 6.276
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 610549500.0
          candidate_median: 650161500.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.013
          change_pct: 6.246
          ci95_low_pct: 5.65
          ci95_high_pct: 6.891
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        system_cpu_ns:
          control_median: 594537500.0
          candidate_median: 608668000.0
          control_p95_over_median: 1.101
          candidate_p95_over_median: 1.05
          change_pct: 2.504
          ci95_low_pct: -1.005
          ci95_high_pct: 6.867
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 129335296.0
          candidate_median: 130523136.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.01
          change_pct: 0.792
          ci95_low_pct: 0.057
          ci95_high_pct: 1.403
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
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
    - job: render-yaml
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 693077646.0
          candidate_median: 739166229.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.032
          change_pct: 5.983
          ci95_low_pct: 4.096
          ci95_high_pct: 7.624
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        component_ns:
          control_median: 118393625.0
          candidate_median: 156388437.5
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.011
          change_pct: 32.374
          ci95_low_pct: 31.154
          ci95_high_pct: 33.006
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1223823500.0
          candidate_median: 1261801500.0
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.076
          change_pct: 2.869
          ci95_low_pct: 0.421
          ci95_high_pct: 5.51
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 604564000.0
          candidate_median: 647933500.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.011
          change_pct: 6.238
          ci95_low_pct: 5.826
          ci95_high_pct: 8.124
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        system_cpu_ns:
          control_median: 618900000.0
          candidate_median: 625310500.0
          control_p95_over_median: 1.125
          candidate_p95_over_median: 1.125
          change_pct: -0.213
          ci95_low_pct: -4.72
          ci95_high_pct: 2.699
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 130498560.0
          candidate_median: 131522560.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.009
          change_pct: 0.933
          ci95_low_pct: 0.666
          ci95_high_pct: 1.362
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
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
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 232459812.0
          candidate_median: 234593479.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.017
          change_pct: -0.011
          ci95_low_pct: -0.65
          ci95_high_pct: 2.19
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 62715208.0
          candidate_median: 62420145.5
          control_p95_over_median: 1.028
          candidate_p95_over_median: 1.005
          change_pct: -0.402
          ci95_low_pct: -2.09
          ci95_high_pct: 0.131
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 229863500.0
          candidate_median: 231365500.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.022
          change_pct: 0.699
          ci95_low_pct: -0.352
          ci95_high_pct: 2.804
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 223906000.0
          candidate_median: 225719500.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.021
          change_pct: 0.736
          ci95_low_pct: -0.105
          ci95_high_pct: 2.214
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 5552500.0
          candidate_median: 5542000.0
          control_p95_over_median: 1.105
          candidate_p95_over_median: 1.239
          change_pct: 0.103
          ci95_low_pct: -7.156
          ci95_high_pct: 13.11
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 2522416.5
          candidate_median: 2243604.5
          control_p95_over_median: 1.547
          candidate_p95_over_median: 1.537
          change_pct: -8.586
          ci95_low_pct: -19.1
          ci95_high_pct: 3.729
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 31621120.0
          candidate_median: 31686656.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.006
          change_pct: 0.13
          ci95_low_pct: -0.078
          ci95_high_pct: 0.623
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
  reference_tools:
    - name: dust
      wall_ns_median: 171107250.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 2908
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "the whole engine change of #191 against 148ef78e (18 source files, 2,646 insertions and 262 deletions), including the review rounds"
  verdict:
    decision: rejected
    primary_job: index-second-report
    primary_metric: component_ns
    change_pct: 0.21
    reason: "not a speed decision: the age column ships regardless, and this re-measures its price at the shipped head with each binary tied to its commit (review C4 to C7 on #191): second tree report over a retained Index +0.21% [-2.36%, +1.61%] and over an opened root +11.49% [+8.42%, +14.22%], 17 us, per-row work; cold-scan-index wall -0.76% [-3.80%, +2.09%] and warm-snapshot-load wall -0.01% non-inferior; default-tree wall +2.33% [-2.99%, +4.64%] not resolved on the loaded host, its peak RSS +4.90% [+0.32%, +9.99%] unattributed; render-json wall +4.27% [+3.36%, +6.47%] and render-yaml +5.98% [+4.10%, +7.62%], about 0.23 to 0.25 us a machine row (fdu-oiuc); opened wall and snapshot-load component signals of exp-210 did not reproduce; Linux unmeasured (H193, fdu-088k)"
    commit: ae90aef4
    kept: candidate
---
## What was predicted

Not a new hypothesis: a re-measure of H192’s cost record at the shipped head of
[#191](https://github.com/jlevy/fdu/pull/191), asked for by review C (C4 to C7).
[exp-210](exp-210-macos-h192-maintained-activity-leaves-the-age-column-per-row.md) and
[exp-211](exp-211-macos-h192-replicated-over-an-opened-root-23-microseconds-a-.md)
measured a binary built from a working tree before its named commit existed, and before
two later changes on measured paths (`1b3ac793`, the folded tree’s restored stats, and
`e25f12e3`, the tree’s age cell).
Since then the reviews added more: observation re-reads a changed entry’s directory
(B1), the maintained activity is an `i64` with a sentinel (C12), the stale-maximum
repair runs only where a maximum was held (C13), each text age cell is formatted once
(C8), and a repaint’s identity digests activity as bytes (C10).

Every machine-format row also gained `modified_at`, an RFC 3339 rendering per row, whose
render jobs had never run (C7); review C estimated a few hundred nanoseconds per row.
The other expectations were H192’s: a retained report pays per-row work, and a cold
walk, a snapshot load, and the default report stay within +3%.

## What was measured

One interleaved probe run on the `rustup` toolchain store (77,355 entries, 3,427
directories, depth 17), 3 warmups and 12 timed trials per variant, exploratory stage, on
an uncontrolled host: the 1-minute load average was 14 at the start and 17 at the end
over 10 cores, the CPU 30% to 32% busy.
No sample was invalid and the tree was unchanged.

Each binary is tied to its source by the run’s variant notes: the control is the probe
built from `148ef78e` (sha256 `d2ac70ff`, the binary exp-209 to exp-211 used), and the
candidate was built from `ae90aef4` with a clean tree (sha256 `d3254e26`), which holds
all of #191’s code after reviews B and C. Later commits on the branch change only
records and documentation.

- `index-second-report` component: 0.130 ms to 0.129 ms, +0.21% [−2.36%, +1.61%].
  Primary. exp-210 measured +5.79% [+2.66%, +7.52%], 7.5 µs, for the first build; C8’s
  single formatting pass is consistent with the difference, but one run on a loaded host
  does not attribute it.
- `opened-second-report` component: 0.134 ms to 0.151 ms, +11.49% [+8.42%, +14.22%], a
  median 17 µs; wall −0.53% [−2.39%, +1.89%], so exp-210’s +1.56% opened-lifecycle wall
  did not reproduce here either.
- `default-tree` wall +2.33% [−2.99%, +4.64%]: not resolved, since the interval crosses
  both zero and the +3% margin.
  Peak RSS +4.90% [+0.32%, +9.99%], 12.6 to 13.2 MB, unattributed: the maintained
  activity adds 8 bytes to each of 3,427 directories, about 27 KB, and the allocation
  counters (`FDU_COUNTERS=1`), not a wall cell, are what would attribute the rest.
- `cold-scan-index` wall −0.76% [−3.80%, +2.09%], non-inferior at +3%; user CPU −0.60%
  [−1.58%, +0.03%].
- `warm-snapshot-load` wall −0.01% [−0.65%, +2.19%] and component −0.40%
  [−2.09%, +0.13%], both non-inferior; exp-210’s +2.21% load component did not
  reproduce.
- `render-json` component 128.8 ms to 163.2 ms, +26.60% [+26.47%, +27.11%], and wall
  +4.27% [+3.36%, +6.47%]; `render-yaml` component 118.4 ms to 156.4 ms, +32.37%
  [+31.15%, +33.01%], and wall +5.98% [+4.10%, +7.62%]. Each streams an unbounded tree
  and a file list, about 151,000 rows, so the new fields cost about 0.23 µs a row in
  JSON and 0.25 µs in YAML, within review C’s estimate.
  This is the one resolved regression here, and it is in the machine formats, not the
  text tree.

## Decision

Not a speed decision, as exp-196, exp-210, and exp-211 are not: the age column ships
regardless, so this records its price at the shipped head, `rejected` as a speed claim
with the candidate kept.
Against H192’s bar it holds: a retained report pays per-row work (17 µs over an opened
root, no resolved change over a retained `Index`), and a cold walk and a snapshot load
are non-inferior at +3%. The default one-shot report’s wall change is not resolved on
this host, and its peak RSS moved +4.9% for a reason this run cannot name.
The machine-format render cost is resolved: +4% to +6% of a full JSON or YAML render’s
wall, about a quarter of a microsecond per row.
Review C’s remedy, writing the digits into a fixed buffer handed to the sink rather than
building a `String` per row, is tracked with this run’s numbers in `fdu-oiuc`. Linux is
unmeasured; H193 (`fdu-088k`) measures the folded tree’s give-back there before 0.5.0.
