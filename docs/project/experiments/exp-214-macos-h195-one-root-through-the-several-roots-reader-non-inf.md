---
title: "macOS: H195 one root through the several-roots reader, non-inferior on five jobs and unresolved on three under load"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-214
  title: "macOS: H195 one root through the several-roots reader, non-inferior on five jobs and unresolved on three under load"
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
    control: "23d11b19 probe: the age column head and the base of #192, built in a clean worktree (sha256 2c738e89)"
    candidate: "3f195a2b probe: #192 after reviews A, B, and C, clean tree (sha256 2495b561)"
    control_binary:
      name: control
      sha256: 2c738e89e4c4dd7773db2985d8466682c45e87310bd46babd5a8094c4fa60da1
      size_bytes: 3380352
      args: []
    candidate_binary:
      name: candidate
      sha256: 2495b5612a3fb8d61145d2eb455a91d7af2b4647d17e8eb507e7735dda638bb4
      size_bytes: 3644944
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-214/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 184736333.5
          candidate_median: 189121438.0
          control_p95_over_median: 1.076
          candidate_p95_over_median: 1.215
          change_pct: 1.844
          ci95_low_pct: -4.536
          ci95_high_pct: 25.239
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 177311728.5
          candidate_median: 183050749.5
          control_p95_over_median: 1.091
          candidate_p95_over_median: 1.222
          change_pct: 1.511
          ci95_low_pct: -3.782
          ci95_high_pct: 25.511
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 726082500.0
          candidate_median: 729620000.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.275
          change_pct: 2.498
          ci95_low_pct: -5.234
          ci95_high_pct: 18.758
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 29574500.0
          candidate_median: 31080000.0
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.037
          change_pct: 4.439
          ci95_low_pct: 0.682
          ci95_high_pct: 7.949
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 695734000.0
          candidate_median: 699123500.0
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.285
          change_pct: 2.451
          ci95_low_pct: -5.401
          ci95_high_pct: 19.201
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 11378688.0
          candidate_median: 12402688.0
          control_p95_over_median: 1.184
          candidate_p95_over_median: 1.347
          change_pct: 7.219
          ci95_low_pct: -0.993
          ci95_high_pct: 33.698
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
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 349256438.0
          candidate_median: 341252396.0
          control_p95_over_median: 1.495
          candidate_p95_over_median: 1.22
          change_pct: -2.25
          ci95_low_pct: -11.464
          ci95_high_pct: 2.731
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 158333792.0
          candidate_median: 162799812.5
          control_p95_over_median: 1.544
          candidate_p95_over_median: 1.345
          change_pct: -1.807
          ci95_low_pct: -10.033
          ci95_high_pct: 9.022
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 812676000.0
          candidate_median: 805993500.0
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.133
          change_pct: -0.983
          ci95_low_pct: -9.998
          ci95_high_pct: 3.713
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 224204500.0
          candidate_median: 221876500.0
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.028
          change_pct: -1.324
          ci95_low_pct: -2.825
          ci95_high_pct: -0.618
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 574310000.0
          candidate_median: 582769000.0
          control_p95_over_median: 1.144
          candidate_p95_over_median: 1.196
          change_pct: -0.908
          ci95_low_pct: -12.573
          ci95_high_pct: 5.502
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 28008448.0
          candidate_median: 27746304.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.021
          change_pct: 1.471
          ci95_low_pct: -2.734
          ci95_high_pct: 3.436
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
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
    - job: content-query
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 18253543583.0
          candidate_median: 16918012270.5
          control_p95_over_median: 1.354
          candidate_p95_over_median: 1.163
          change_pct: -7.859
          ci95_low_pct: -20.625
          ci95_high_pct: -3.057
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 12869070583.5
          candidate_median: 11130276958.5
          control_p95_over_median: 1.211
          candidate_p95_over_median: 1.252
          change_pct: -6.82
          ci95_low_pct: -15.152
          ci95_high_pct: -1.221
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 23959181000.0
          candidate_median: 24205823000.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.067
          change_pct: 0.758
          ci95_low_pct: -3.854
          ci95_high_pct: 9.189
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 15248580000.0
          candidate_median: 15010187500.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.022
          change_pct: -2.164
          ci95_low_pct: -2.324
          ci95_high_pct: -0.702
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 8456708000.0
          candidate_median: 8916161500.0
          control_p95_over_median: 1.185
          candidate_p95_over_median: 1.193
          change_pct: 9.912
          ci95_low_pct: -7.147
          ci95_high_pct: 33.769
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 982040576.0
          candidate_median: 984752128.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.005
          change_pct: 0.196
          ci95_low_pct: -0.598
          ci95_high_pct: 1.816
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 207385437.5
          candidate_median: 190929604.0
          control_p95_over_median: 1.118
          candidate_p95_over_median: 1.226
          change_pct: -7.178
          ci95_low_pct: -11.594
          ci95_high_pct: -0.117
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 196092499.5
          candidate_median: 182393062.5
          control_p95_over_median: 1.08
          candidate_p95_over_median: 1.244
          change_pct: -6.891
          ci95_low_pct: -12.752
          ci95_high_pct: 5.162
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 611228500.0
          candidate_median: 637062500.0
          control_p95_over_median: 1.127
          candidate_p95_over_median: 1.097
          change_pct: 2.486
          ci95_low_pct: -1.296
          ci95_high_pct: 11.16
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 42119000.0
          candidate_median: 42857000.0
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.049
          change_pct: 1.313
          ci95_low_pct: -0.968
          ci95_high_pct: 5.046
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 570158000.0
          candidate_median: 594606500.0
          control_p95_over_median: 1.129
          candidate_p95_over_median: 1.099
          change_pct: 2.589
          ci95_low_pct: -1.509
          ci95_high_pct: 11.619
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 12345344.0
          candidate_median: 12124160.0
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.093
          change_pct: -1.759
          ci95_low_pct: -9.224
          ci95_high_pct: 3.199
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
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
    - job: index-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 358332395.5
          candidate_median: 358828750.0
          control_p95_over_median: 1.03
          candidate_p95_over_median: 1.068
          change_pct: 0.105
          ci95_low_pct: -0.931
          ci95_high_pct: 2.274
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 132854.5
          candidate_median: 135291.5
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.018
          change_pct: -0.138
          ci95_low_pct: -2.353
          ci95_high_pct: 3.256
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 907214500.0
          candidate_median: 910981500.0
          control_p95_over_median: 1.082
          candidate_p95_over_median: 1.087
          change_pct: 0.582
          ci95_low_pct: -5.154
          ci95_high_pct: 3.172
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 222173000.0
          candidate_median: 221615000.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.026
          change_pct: 0.779
          ci95_low_pct: -0.797
          ci95_high_pct: 1.603
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 686975000.0
          candidate_median: 690068500.0
          control_p95_over_median: 1.102
          candidate_p95_over_median: 1.109
          change_pct: 0.25
          ci95_low_pct: -6.501
          ci95_high_pct: 3.825
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 27893760.0
          candidate_median: 27484160.0
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.026
          change_pct: -2.385
          ci95_low_pct: -3.913
          ci95_high_pct: -0.301
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
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
    - job: opened-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1517621979.5
          candidate_median: 1399900792.0
          control_p95_over_median: 1.248
          candidate_p95_over_median: 1.239
          change_pct: -2.08
          ci95_low_pct: -16.486
          ci95_high_pct: 11.298
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 158791.5
          candidate_median: 157291.5
          control_p95_over_median: 1.091
          candidate_p95_over_median: 1.034
          change_pct: -0.267
          ci95_low_pct: -3.502
          ci95_high_pct: 1.21
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1415631000.0
          candidate_median: 1346415000.0
          control_p95_over_median: 1.104
          candidate_p95_over_median: 1.093
          change_pct: -1.176
          ci95_low_pct: -5.87
          ci95_high_pct: 1.403
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 894680000.0
          candidate_median: 889557000.0
          control_p95_over_median: 1.063
          candidate_p95_over_median: 1.072
          change_pct: -0.475
          ci95_low_pct: -2.334
          ci95_high_pct: 0.359
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 494103500.0
          candidate_median: 469544000.0
          control_p95_over_median: 1.299
          candidate_p95_over_median: 1.132
          change_pct: -4.025
          ci95_low_pct: -13.097
          ci95_high_pct: 2.958
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 116097024.0
          candidate_median: 114466816.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.009
          change_pct: -1.133
          ci95_low_pct: -1.951
          ci95_high_pct: -0.807
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
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
          control_median: 1588911104.0
          candidate_median: 1469927604.0
          control_p95_over_median: 1.269
          candidate_p95_over_median: 1.173
          change_pct: -8.385
          ci95_low_pct: -14.067
          ci95_high_pct: 0.761
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 288554021.0
          candidate_median: 261980083.5
          control_p95_over_median: 1.251
          candidate_p95_over_median: 1.279
          change_pct: -9.941
          ci95_low_pct: -20.777
          ci95_high_pct: 5.061
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1728826500.0
          candidate_median: 1704685500.0
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.199
          change_pct: 1.869
          ci95_low_pct: -4.687
          ci95_high_pct: 8.422
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 710953500.0
          candidate_median: 705494000.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.035
          change_pct: -1.238
          ci95_low_pct: -2.636
          ci95_high_pct: 0.227
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 997271500.0
          candidate_median: 998735000.0
          control_p95_over_median: 1.224
          candidate_p95_over_median: 1.325
          change_pct: 5.807
          ci95_low_pct: -6.436
          ci95_high_pct: 15.87
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 130498560.0
          candidate_median: 130129920.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.009
          change_pct: -0.324
          ci95_low_pct: -1.112
          ci95_high_pct: -0.075
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: render-yaml
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1469604874.5
          candidate_median: 1663605792.0
          control_p95_over_median: 1.328
          candidate_p95_over_median: 1.459
          change_pct: 7.674
          ci95_low_pct: -10.311
          ci95_high_pct: 24.992
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 234067187.5
          candidate_median: 265371645.5
          control_p95_over_median: 2.248
          candidate_p95_over_median: 2.468
          change_pct: 6.884
          ci95_low_pct: -10.487
          ci95_high_pct: 36.847
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1910079500.0
          candidate_median: 1848104000.0
          control_p95_over_median: 1.105
          candidate_p95_over_median: 1.096
          change_pct: -3.068
          ci95_low_pct: -8.823
          ci95_high_pct: 1.182
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 720130500.0
          candidate_median: 736295000.0
          control_p95_over_median: 1.185
          candidate_p95_over_median: 1.107
          change_pct: 0.145
          ci95_low_pct: -2.891
          ci95_high_pct: 5.265
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1202005500.0
          candidate_median: 1123638000.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.064
          change_pct: -6.491
          ci95_low_pct: -13.801
          ci95_high_pct: -0.062
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 130318336.0
          candidate_median: 130670592.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.007
          change_pct: 0.779
          ci95_low_pct: -0.296
          ci95_high_pct: 1.094
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 539839583.5
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
    change_pct: -7.178
    reason: "not a speed decision: several roots ship regardless; on an uncontrolled host (load 26 to 34 on 10 cores) wall is non-inferior at +3% on cold-scan-index (the placebo, -2.25% [-11.46%, +2.73%]), default-tree, index-second-report (+0.10% [-0.93%, +2.27%]), content-query, and render-json, with the retained reads flat in component; aggregate-summary (+1.84% [-4.54%, +25.24%]), opened-second-report, and render-yaml are unresolved, so H195 is neither confirmed nor refuted there and those three rerun quiet with H193 (fdu-088k)"
    commit: 3f195a2b
    kept: candidate
---
## What was predicted

H195, registered before this run (`ce69dc45`): review C1 on
[#192](https://github.com/jlevy/fdu/pull/192). The several-roots refactor reads every
one-shot and retained report through a per-root `RootRead`, three merges, and a sort
comparator that ranks by root (a constant for one root since review C8); the command
line and Python enter through `RootsRequest::resolve` and `prepare_roots_report`;
`FileRow` gains `root` within its padding; and the machine renderers test `root` on
every row. By source reading none of it adds work per entry for one root, so the
prediction was wall non-inferior at +3% on every job the refactor reaches, with
`cold-scan-index` as the placebo, since the walk and the index build are untouched.
The registration also said that an interval crossing +3% on this host is inconclusive,
not a regression, and is rerun quieter.

## What was measured

One interleaved run on the `rustup` toolchain store (77,355 entries, 3,427 directories,
depth 17), the subject of exp-212 and exp-213, 3 warmups and 12 timed trials per
variant, exploratory stage, on an uncontrolled host: the 1-minute load average was 26.1
at the start and 34.1 at the end (15-minute 37.0 and 28.1) over 10 cores, the CPU 74%
and 100% busy. No sample was invalid and the tree was unchanged.

The binaries are bound by their variant notes.
The control is the probe built from `23d11b19`, #192’s base, in a clean scratch worktree
with its own target directory (sha256 `2c738e89`). The candidate is the probe built from
`3f195a2b`, #192’s head after every engine change of this round, with a clean tree
(sha256 `2495b561`).

| Job | Wall change [95% interval] | Against +3% |
| --- | --- | --- |
| `cold-scan-index` (placebo) | −2.25% [−11.46%, +2.73%] | non-inferior |
| `default-tree` | −7.18% [−11.59%, −0.12%] | non-inferior (interval below zero) |
| `aggregate-summary` | +1.84% [−4.54%, +25.24%] | inconclusive |
| `index-second-report` | +0.10% [−0.93%, +2.27%] | non-inferior |
| `opened-second-report` | −2.08% [−16.49%, +11.30%] | inconclusive |
| `content-query` | −7.86% [−20.62%, −3.06%] | non-inferior (interval below zero) |
| `render-json` | −8.38% [−14.07%, +0.76%] | non-inferior |
| `render-yaml` | +7.67% [−10.31%, +24.99%] | inconclusive |

The retained reads, whose component the refactor reaches most directly, are flat:
`index-second-report` component −0.14% [−2.35%, +3.26%] and `opened-second-report`
component −0.27% [−3.50%, +1.21%], with user CPU and peak RSS non-inferior on both.
`aggregate-summary` user CPU is +4.44% [+0.68%, +7.95%], the one CPU interval wholly
above zero; its wall and component intervals reach +25%. The two intervals below zero
are not read as speedups: nothing in the change removes work from those paths, and the
placebo itself moved by −2%.

## Decision

Not a speed decision: several roots ship regardless, and this is the guard that one root
did not get slower. On five jobs it holds at +3%, the placebo and both retained reads
among them. On `aggregate-summary`, `opened-second-report`, and `render-yaml` this host,
at a load average near three times its core count, could not resolve the margin either
way, so H195 is neither confirmed nor refuted there.
As registered, those three are rerun on a quiet host; they ride with the Linux release
measurement (H193, `fdu-088k`), whose session runs the same jobs at the top of the
stack. Recorded `rejected` as a speed claim, with the candidate kept.
