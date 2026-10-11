---
title: "macOS: an early look at one root through the several-roots reader, unresolved on an overloaded host"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-215
  title: "macOS: an early look at one root through the several-roots reader, unresolved on an overloaded host"
  date: "2026-10-09"
  hypotheses:
    - H195
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
    control: "b2968074 probe (sha256 c4e01629, exp-213 candidate; engine as 23d11b19; the run variant notes are empty, so the binding is stated in the record body)"
    candidate: "d0621141 probe: #192 before review (sha256 e40a5173; binding stated in the record body)"
    control_binary:
      name: control
      sha256: c4e0162987460cadcf82dc566df7c3b10119597d3715cdc7cac9d6befd13f53f
      size_bytes: 3380352
      args: []
    candidate_binary:
      name: candidate
      sha256: e40a5173f2bffd6df54207985f49b2d298674fcc0e091e678519329e64abe437
      size_bytes: 3496096
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-215/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 223963458.0
          candidate_median: 206123479.5
          control_p95_over_median: 1.222
          candidate_p95_over_median: 1.314
          change_pct: -1.834
          ci95_low_pct: -22.653
          ci95_high_pct: 16.905
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 189165729.0
          candidate_median: 193709646.0
          control_p95_over_median: 1.229
          candidate_p95_over_median: 1.245
          change_pct: 4.489
          ci95_low_pct: -13.443
          ci95_high_pct: 14.324
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 553897500.0
          candidate_median: 539133000.0
          control_p95_over_median: 1.066
          candidate_p95_over_median: 1.162
          change_pct: 1.469
          ci95_low_pct: -10.491
          ci95_high_pct: 14.235
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 27749000.0
          candidate_median: 27634500.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.08
          change_pct: 0.402
          ci95_low_pct: -0.045
          ci95_high_pct: 4.159
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 526044000.0
          candidate_median: 510369500.0
          control_p95_over_median: 1.069
          candidate_p95_over_median: 1.174
          change_pct: 1.505
          ci95_low_pct: -10.761
          ci95_high_pct: 15.056
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 16965632.0
          candidate_median: 16490496.0
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.185
          change_pct: -2.997
          ci95_low_pct: -10.822
          ci95_high_pct: 6.479
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
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 498495895.5
          candidate_median: 485949666.5
          control_p95_over_median: 1.136
          candidate_p95_over_median: 1.103
          change_pct: 2.816
          ci95_low_pct: -1.655
          ci95_high_pct: 5.393
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 280737208.5
          candidate_median: 279087979.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.073
          change_pct: -2.536
          ci95_low_pct: -8.659
          ci95_high_pct: 11.715
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1222779500.0
          candidate_median: 1310519500.0
          control_p95_over_median: 1.097
          candidate_p95_over_median: 1.06
          change_pct: 2.521
          ci95_low_pct: -1.769
          ci95_high_pct: 8.735
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 224364500.0
          candidate_median: 223200000.0
          control_p95_over_median: 1.058
          candidate_p95_over_median: 1.099
          change_pct: -0.646
          ci95_low_pct: -3.54
          ci95_high_pct: 2.644
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 992729000.0
          candidate_median: 1075051000.0
          control_p95_over_median: 1.119
          candidate_p95_over_median: 1.091
          change_pct: 3.253
          ci95_low_pct: -3.517
          ci95_high_pct: 9.656
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 27426816.0
          candidate_median: 27435008.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.047
          change_pct: 0.537
          ci95_low_pct: -2.314
          ci95_high_pct: 4.598
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
          - "voluntary_context_switches straddles its +50% regression limit"
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 203025625.5
          candidate_median: 204193396.0
          control_p95_over_median: 1.134
          candidate_p95_over_median: 1.469
          change_pct: 4.914
          ci95_low_pct: -14.375
          ci95_high_pct: 31.569
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 178443395.5
          candidate_median: 180884229.0
          control_p95_over_median: 1.217
          candidate_p95_over_median: 1.594
          change_pct: 6.105
          ci95_low_pct: -16.423
          ci95_high_pct: 36.884
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 446211000.0
          candidate_median: 484452000.0
          control_p95_over_median: 1.141
          candidate_p95_over_median: 1.09
          change_pct: 8.122
          ci95_low_pct: 3.923
          ci95_high_pct: 16.351
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        user_cpu_ns:
          control_median: 38465000.0
          candidate_median: 39678500.0
          control_p95_over_median: 1.071
          candidate_p95_over_median: 1.048
          change_pct: 2.311
          ci95_low_pct: -0.872
          ci95_high_pct: 6.079
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 406829500.0
          candidate_median: 443773500.0
          control_p95_over_median: 1.149
          candidate_p95_over_median: 1.101
          change_pct: 9.083
          ci95_low_pct: 3.899
          ci95_high_pct: 18.06
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        peak_rss_bytes:
          control_median: 13361152.0
          candidate_median: 12689408.0
          control_p95_over_median: 1.109
          candidate_p95_over_median: 1.063
          change_pct: -6.582
          ci95_low_pct: -10.623
          ci95_high_pct: 2.867
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
          - "voluntary_context_switches straddles its +50% regression limit"
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
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: index-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 355509500.0
          candidate_median: 364954458.5
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.154
          change_pct: 2.6
          ci95_low_pct: -1.184
          ci95_high_pct: 3.798
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 133958.5
          candidate_median: 131229.0
          control_p95_over_median: 1.098
          candidate_p95_over_median: 1.029
          change_pct: -3.705
          ci95_low_pct: -8.097
          ci95_high_pct: 2.094
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 880915000.0
          candidate_median: 865567000.0
          control_p95_over_median: 1.141
          candidate_p95_over_median: 1.078
          change_pct: -1.044
          ci95_low_pct: -4.279
          ci95_high_pct: 3.601
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 220267500.0
          candidate_median: 219537500.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.054
          change_pct: 0.232
          ci95_low_pct: -2.515
          ci95_high_pct: 3.111
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 659752000.0
          candidate_median: 646937500.0
          control_p95_over_median: 1.19
          candidate_p95_over_median: 1.101
          change_pct: -1.595
          ci95_low_pct: -4.713
          ci95_high_pct: 2.808
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 28295168.0
          candidate_median: 27500544.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.029
          change_pct: -2.135
          ci95_low_pct: -5.13
          ci95_high_pct: 0.66
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
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: opened-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 5783896458.5
          candidate_median: 5552616729.0
          control_p95_over_median: 1.242
          candidate_p95_over_median: 1.162
          change_pct: -6.959
          ci95_low_pct: -17.511
          ci95_high_pct: 6.103
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 154146.0
          candidate_median: 261187.0
          control_p95_over_median: 2.178
          candidate_p95_over_median: 27.089
          change_pct: 26.445
          ci95_low_pct: -19.477
          ci95_high_pct: 192.358
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1477727500.0
          candidate_median: 1477336500.0
          control_p95_over_median: 1.212
          candidate_p95_over_median: 1.097
          change_pct: -0.637
          ci95_low_pct: -16.044
          ci95_high_pct: 3.44
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 996117500.0
          candidate_median: 984862500.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.036
          change_pct: -0.696
          ci95_low_pct: -1.609
          ci95_high_pct: 1.006
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 490465500.0
          candidate_median: 489404500.0
          control_p95_over_median: 1.636
          candidate_p95_over_median: 1.306
          change_pct: -1.676
          ci95_low_pct: -35.782
          ci95_high_pct: 8.146
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 116613120.0
          candidate_median: 116572160.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.013
          change_pct: 0.071
          ci95_low_pct: -0.98
          ci95_high_pct: 0.759
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
          - "voluntary_context_switches straddles its +50% regression limit"
          - major_faults does not establish non-regression
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
          major_faults: inconclusive
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
          control_median: 1244392750.0
          candidate_median: 1440726979.5
          control_p95_over_median: 2.441
          candidate_p95_over_median: 1.439
          change_pct: 7.783
          ci95_low_pct: -11.356
          ci95_high_pct: 19.036
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 319763333.5
          candidate_median: 334946187.5
          control_p95_over_median: 2.412
          candidate_p95_over_median: 1.551
          change_pct: -1.825
          ci95_low_pct: -24.566
          ci95_high_pct: 29.901
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1285027000.0
          candidate_median: 1224890000.0
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.243
          change_pct: 7.439
          ci95_low_pct: -5.591
          ci95_high_pct: 14.087
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 657702000.0
          candidate_median: 665727000.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.015
          change_pct: 1.23
          ci95_low_pct: -1.285
          ci95_high_pct: 2.946
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 627326000.0
          candidate_median: 576518500.0
          control_p95_over_median: 1.201
          candidate_p95_over_median: 1.479
          change_pct: 14.839
          ci95_low_pct: -9.898
          ci95_high_pct: 30.097
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 131874816.0
          candidate_median: 131923968.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.014
          change_pct: 0.266
          ci95_low_pct: -1.421
          ci95_high_pct: 1.6
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
          - "voluntary_context_switches straddles its +50% regression limit"
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
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 271849270.5
          candidate_median: 334523333.0
          control_p95_over_median: 1.605
          candidate_p95_over_median: 1.899
          change_pct: 2.6
          ci95_low_pct: -6.911
          ci95_high_pct: 38.364
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 71807917.0
          candidate_median: 77432395.5
          control_p95_over_median: 1.303
          candidate_p95_over_median: 1.884
          change_pct: 7.99
          ci95_low_pct: -2.444
          ci95_high_pct: 44.699
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 239507500.0
          candidate_median: 238185000.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.064
          change_pct: -0.13
          ci95_low_pct: -1.616
          ci95_high_pct: 1.432
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 231198500.0
          candidate_median: 230039500.0
          control_p95_over_median: 1.039
          candidate_p95_over_median: 1.045
          change_pct: -0.311
          ci95_low_pct: -1.386
          ci95_high_pct: 1.001
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 8554500.0
          candidate_median: 7557000.0
          control_p95_over_median: 1.248
          candidate_p95_over_median: 1.586
          change_pct: -7.951
          ci95_low_pct: -13.174
          ci95_high_pct: 13.083
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 29746478.5
          candidate_median: 93591020.5
          control_p95_over_median: 6.137
          candidate_p95_over_median: 4.058
          change_pct: 32.063
          ci95_low_pct: -25.073
          ci95_high_pct: 249.149
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 31784960.0
          candidate_median: 31670272.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.006
          change_pct: -0.077
          ci95_low_pct: -0.671
          ci95_high_pct: 0.104
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
          - "voluntary_context_switches straddles its +50% regression limit"
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
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 216364333.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 4.914
    reason: "inconclusive, and recorded so it is not read as evidence either way: run before H195 was registered, on a host at load 25 to 43 over 10 cores; every wall interval includes zero and is wider than the +3% margin, default-tree +4.91% [-14.38%, +31.57%]; superseded by exp-214 at the reviewed head"
    commit: d0621141
    kept: candidate
---
## What was predicted

Nothing was registered: this exploratory run preceded H195, which was registered
afterwards for exp-214. It was a first look at whether #192’s several-roots refactor
slowed one root, at the PR’s first complete head.

## What was measured

One interleaved run on the `rustup` toolchain store (77,355 entries), 3 warmups and 12
timed trials per variant, exploratory stage, on an uncontrolled host that was far busier
than the rule allows: the 1-minute load average was 27.4 at the start and 24.9 at the
end, the 15-minute 24.8 and 43.1, over 10 cores, the CPU 100% and 99% busy.
No sample was invalid and the tree was unchanged.

The run’s variant notes are empty, so the binding is stated here.
The control is the probe built from `b2968074` (sha256 `c4e01629`, exp-213’s candidate),
whose engine is the one `23d11b19` ships, since the commits between them change records
and documents. The candidate is the probe built from `d0621141`, #192’s head before
review (sha256 `e40a5173`).

Every wall interval includes zero, and every one is wider than the +3% margin:
`default-tree` +4.91% [−14.38%, +31.57%], `aggregate-summary` −1.83% [−22.65%, +16.91%],
`index-second-report` +2.60% [−1.18%, +3.80%], `opened-second-report` −6.96%
[−17.51%, +6.10%], `render-json` +7.78% [−11.36%, +19.04%], `cold-scan-index` +2.82%
[−1.66%, +5.39%], and `warm-snapshot-load` +2.60% [−6.91%, +38.36%].

## Decision

Inconclusive, and recorded so that it is not mistaken for evidence either way: the host
was too loaded to resolve any job against the margin.
Superseded by exp-214, which tests H195 at #192’s head after review against a control
built from `23d11b19`. Recorded `rejected` as a speed claim, with the candidate kept.
