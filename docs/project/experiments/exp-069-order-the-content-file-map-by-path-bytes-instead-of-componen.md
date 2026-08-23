---
title: Order the content file map by path bytes instead of components
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-069
  title: Order the content file map by path bytes instead of components
  date: "2026-08-23"
  hypotheses:
    - H102
  subject:
    tree_label: metabrowser-clone
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: f7150a975da1a887f4687731f5554b3647fbae0885d15999af5c4526274910e4
    tree_provenance: "A clone of github.com/jlevy/metabrowser at 433fb6e retained as the loop's base tree, with the node_modules, .venv and .claude worktree state left from its use as a workspace. The clone is reproducible; the workspace state on top of it is not, so the shape is not."
    tree_reconstructible: false
    tree_entries: 60089
    tree_directories: 7350
    tree_files: 52717
    tree_symlinks: 22
    tree_apparent_bytes: 1085218928
    tree_allocated_bytes: 1230254080
    tree_max_depth: 19
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
    control: main at 4d29d6d (perf_probe.control)
    candidate: "ContentIndex::files keyed by a byte-ordered PathKey; prefix-range invalidation with an explicit separator"
    control_binary:
      name: control
      sha256: fd925c1564331d7f6387d4a1b08f20c44e4cb4dcdb3e162fd7cf10a82150b2b2
      size_bytes: 1561440
      args: []
    candidate_binary:
      name: candidate
      sha256: a1d9764c64e166a6a1a0e4e0951811eb9423748ab08955a5c07677c89b9d397a
      size_bytes: 1561440
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-069-content-files-byte-order.json
  results:
    - job: code-sloc
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2088728458.5
          candidate_median: 2155060292.0
          control_p95_over_median: 1.28
          candidate_p95_over_median: 1.241
          change_pct: 0.272
          ci95_low_pct: -20.485
          ci95_high_pct: 28.853
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1748729333.5
          candidate_median: 1864676646.0
          control_p95_over_median: 1.336
          candidate_p95_over_median: 1.253
          change_pct: 0.347
          ci95_low_pct: -21.346
          ci95_high_pct: 34.015
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 13466477000.0
          candidate_median: 13054774500.0
          control_p95_over_median: 1.06
          candidate_p95_over_median: 1.081
          change_pct: -0.851
          ci95_low_pct: -9.354
          ci95_high_pct: 3.138
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 6329182000.0
          candidate_median: 6245136000.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.016
          change_pct: -1.737
          ci95_low_pct: -2.582
          ci95_high_pct: -0.033
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 7133336000.0
          candidate_median: 6781988500.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.164
          change_pct: -2.786
          ci95_low_pct: -16.149
          ci95_high_pct: 7.689
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 153280512.0
          candidate_median: 152141824.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.017
          change_pct: 0.124
          ci95_low_pct: -1.878
          ci95_high_pct: 1.553
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
    - job: code-sloc-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 583738313.0
          candidate_median: 410890083.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.023
          change_pct: -29.971
          ci95_low_pct: -31.733
          ci95_high_pct: -28.673
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 460954333.0
          candidate_median: 285196791.5
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.025
          change_pct: -38.289
          ci95_low_pct: -38.98
          ci95_high_pct: -36.482
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 578359000.0
          candidate_median: 406800500.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.025
          change_pct: -30.162
          ci95_low_pct: -31.638
          ci95_high_pct: -28.639
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 549771000.0
          candidate_median: 375834500.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.019
          change_pct: -31.886
          ci95_low_pct: -33.007
          ci95_high_pct: -30.388
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 28176500.0
          candidate_median: 29406000.0
          control_p95_over_median: 1.288
          candidate_p95_over_median: 1.354
          change_pct: -0.01
          ci95_low_pct: -6.111
          ci95_high_pct: 6.439
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 3911416.5
          candidate_median: 3367666.5
          control_p95_over_median: 1.388
          candidate_p95_over_median: 1.344
          change_pct: -20.035
          ci95_low_pct: -39.924
          ci95_high_pct: 7.687
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 185901056.0
          candidate_median: 185204736.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.031
          change_pct: -0.279
          ci95_low_pct: -2.854
          ci95_high_pct: 1.429
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
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
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1987780687.0
          candidate_median: 1999385479.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.092
          change_pct: 3.095
          ci95_low_pct: -2.485
          ci95_high_pct: 9.115
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1701486020.5
          candidate_median: 1691028479.5
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.112
          change_pct: 2.813
          ci95_low_pct: -3.42
          ci95_high_pct: 12.04
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 12772087500.0
          candidate_median: 13031653500.0
          control_p95_over_median: 1.137
          candidate_p95_over_median: 1.071
          change_pct: -1.248
          ci95_low_pct: -6.331
          ci95_high_pct: 8.724
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 2699778000.0
          candidate_median: 2584176000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.013
          change_pct: -4.269
          ci95_low_pct: -4.819
          ci95_high_pct: -2.54
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 10058202000.0
          candidate_median: 10440282500.0
          control_p95_over_median: 1.176
          candidate_p95_over_median: 1.093
          change_pct: -0.341
          ci95_low_pct: -7.258
          ci95_high_pct: 12.334
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 147800064.0
          candidate_median: 146685952.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.014
          change_pct: -0.562
          ci95_low_pct: -1.027
          ci95_high_pct: 0.288
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
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 577857708.0
          candidate_median: 408863271.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.153
          change_pct: -31.003
          ci95_low_pct: -31.429
          ci95_high_pct: -27.805
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 453463562.5
          candidate_median: 283646104.5
          control_p95_over_median: 1.039
          candidate_p95_over_median: 1.226
          change_pct: -38.639
          ci95_low_pct: -39.306
          ci95_high_pct: -36.429
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 573804000.0
          candidate_median: 404629500.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.023
          change_pct: -31.123
          ci95_low_pct: -31.452
          ci95_high_pct: -28.499
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 542961500.0
          candidate_median: 371706000.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.034
          change_pct: -32.238
          ci95_low_pct: -33.021
          ci95_high_pct: -30.414
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 29595500.0
          candidate_median: 29634000.0
          control_p95_over_median: 1.248
          candidate_p95_over_median: 1.197
          change_pct: -7.484
          ci95_low_pct: -9.721
          ci95_high_pct: 1.556
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        blocked_ns:
          control_median: 3841042.0
          candidate_median: 4088771.0
          control_p95_over_median: 1.464
          candidate_p95_over_median: 14.059
          change_pct: -1.611
          ci95_low_pct: -19.708
          ci95_high_pct: 88.552
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 189800448.0
          candidate_median: 187613184.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.017
          change_pct: -0.385
          ci95_low_pct: -2.836
          ci95_high_pct: 0.509
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
    - job: content-query
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 30697330833.5
          candidate_median: 10029405604.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.097
          change_pct: -67.251
          ci95_low_pct: -67.451
          ci95_high_pct: -66.85
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 28559225333.5
          candidate_median: 8166980729.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.022
          change_pct: -71.454
          ci95_low_pct: -71.557
          ci95_high_pct: -70.888
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 39681710500.0
          candidate_median: 21744181000.0
          control_p95_over_median: 1.099
          candidate_p95_over_median: 1.117
          change_pct: -47.077
          ci95_low_pct: -49.619
          ci95_high_pct: -45.403
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 31201600500.0
          candidate_median: 10673722000.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.018
          change_pct: -65.756
          ci95_low_pct: -65.981
          ci95_high_pct: -65.324
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 8529275000.0
          candidate_median: 11025164000.0
          control_p95_over_median: 1.449
          candidate_p95_over_median: 1.235
          change_pct: 13.262
          ci95_low_pct: 2.2
          ci95_high_pct: 24.176
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 158515200.0
          candidate_median: 158441472.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.015
          change_pct: 0.157
          ci95_low_pct: -0.894
          ci95_high_pct: 0.835
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
    - job: document-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 585677354.0
          candidate_median: 412359604.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.013
          change_pct: -30.238
          ci95_low_pct: -31.641
          ci95_high_pct: -28.844
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 459401313.0
          candidate_median: 286895146.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.02
          change_pct: -37.961
          ci95_low_pct: -39.589
          ci95_high_pct: -37.173
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 581692000.0
          candidate_median: 408758000.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.013
          change_pct: -30.393
          ci95_low_pct: -31.797
          ci95_high_pct: -29.193
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 547721500.0
          candidate_median: 377201500.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.01
          change_pct: -31.558
          ci95_low_pct: -33.388
          ci95_high_pct: -30.71
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 32815000.0
          candidate_median: 31644500.0
          control_p95_over_median: 1.206
          candidate_p95_over_median: 1.09
          change_pct: -7.614
          ci95_low_pct: -15.273
          ci95_high_pct: -0.701
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        blocked_ns:
          control_median: 3276854.0
          candidate_median: 3475916.5
          control_p95_over_median: 1.293
          candidate_p95_over_median: 1.139
          change_pct: -3.685
          ci95_low_pct: -8.295
          ci95_high_pct: 18.436
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 187113472.0
          candidate_median: 192274432.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.003
          change_pct: 1.248
          ci95_low_pct: -0.248
          ci95_high_pct: 2.955
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
    - job: markdown-prose
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1834416895.5
          candidate_median: 2084872916.5
          control_p95_over_median: 1.161
          candidate_p95_over_median: 1.141
          change_pct: 6.417
          ci95_low_pct: -2.966
          ci95_high_pct: 22.752
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1542894020.5
          candidate_median: 1782771104.5
          control_p95_over_median: 1.169
          candidate_p95_over_median: 1.153
          change_pct: 6.841
          ci95_low_pct: -0.83
          ci95_high_pct: 24.783
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 12626147500.0
          candidate_median: 12774510500.0
          control_p95_over_median: 1.1
          candidate_p95_over_median: 1.091
          change_pct: 4.857
          ci95_low_pct: -7.98
          ci95_high_pct: 16.758
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 3050359500.0
          candidate_median: 3013273500.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.02
          change_pct: -2.057
          ci95_low_pct: -3.312
          ci95_high_pct: -0.254
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 9537288500.0
          candidate_median: 9730941500.0
          control_p95_over_median: 1.135
          candidate_p95_over_median: 1.123
          change_pct: 6.507
          ci95_low_pct: -10.382
          ci95_high_pct: 23.835
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 154509312.0
          candidate_median: 154132480.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.014
          change_pct: -0.49
          ci95_low_pct: -1.69
          ci95_high_pct: 1.505
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
    - job: text-prose
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1906260229.0
          candidate_median: 1896753771.0
          control_p95_over_median: 1.288
          candidate_p95_over_median: 1.234
          change_pct: -0.632
          ci95_low_pct: -24.257
          ci95_high_pct: 26.048
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1621289062.5
          candidate_median: 1608232375.0
          control_p95_over_median: 1.321
          candidate_p95_over_median: 1.265
          change_pct: -0.052
          ci95_low_pct: -27.262
          ci95_high_pct: 26.593
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 11266895500.0
          candidate_median: 11551687500.0
          control_p95_over_median: 1.354
          candidate_p95_over_median: 1.321
          change_pct: 0.527
          ci95_low_pct: -3.957
          ci95_high_pct: 21.161
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 3088009000.0
          candidate_median: 2978271500.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.032
          change_pct: -4.158
          ci95_low_pct: -6.453
          ci95_high_pct: -1.939
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 8052901500.0
          candidate_median: 8610438500.0
          control_p95_over_median: 1.489
          candidate_p95_over_median: 1.418
          change_pct: 4.535
          ci95_low_pct: -7.056
          ci95_high_pct: 31.647
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 153255936.0
          candidate_median: 152633344.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.024
          change_pct: -0.963
          ci95_low_pct: -1.62
          ci95_high_pct: 1.248
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
    lines_changed: 133
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -31.003
    reason: "content-cache-hit -31.00% [-31.43%, -27.80%] on a dense 52k-file real checkout with the content digest identical and RSS flat, for a key type that changes nothing observable; the cold jobs are unchanged and content-query fell 67% by the same mechanism."
    commit: 4d29d6d
---
## What was measured

The warm content open on the metabrowser clone (52,717 files, dense at 0.88), profiled
first as the loop requires: a third of the samples were path comparison —
`compare_components` 17.0% and `Components::next` 12.1% — which is
`BTreeMap<PathBuf, _>` descending to a record and re-parsing both paths into components
at every node on the way down.
That is the residue `fdu-78q6`’s 2026-08-21 re-screen named after exp-064 removed the
larger `merge_ancestors` half of the same cost.

The change keys `ContentIndex::files` by a `PathKey` ordered by the path’s bytes — one
`memcmp` per comparison — instead of by components.
The map stays a `BTreeMap`, so the sidecar is still written in a deterministic order and
subtree invalidation still works as a contiguous prefix range: every record beneath a
directory shares the directory’s bytes and a separator, and the separator in the prefix
is what keeps `src-extra/` and `src2/` out of `src/`. Lookups borrow the key as bytes,
so `file(&Path)` allocates nothing.
The order of records in the sidecar changes from component order to byte order; the
loader applies records by key and never depended on the order.

Entry point: `PathKey`, `ContentIndex::invalidate` and `::commit` in
`crates/fdu-core/src/content/content_index.rs`, with tests for the separator boundary
and the deterministic order.

## Result

On the 52,717-file metabrowser clone, twelve paired trials, uncontrolled host:

| job | control | candidate | paired change | interval |
| --- | ---: | ---: | ---: | --- |
| `content-cache-hit` (predicted) | 577.9 ms | 408.9 ms | **−31.00%** | [−31.43%, −27.80%] |
| `code-sloc-cache-hit` | 583.7 ms | 410.9 ms | −29.97% | [−31.73%, −28.67%] |
| `document-cache-hit` | 585.7 ms | 412.4 ms | −30.24% | [−31.64%, −28.84%] |
| `content-query` | 30,697 ms | 10,029 ms | −67.25% | [−67.45%, −66.85%] |
| `content-basic` (cold) | 1,987.8 ms | 1,999.4 ms | +3.10% | [−2.48%, +9.12%] |
| `code-sloc` (cold) | 2,088.7 ms | 2,155.1 ms | +0.27% | [−20.48%, +28.85%] |
| `text-prose` (cold) | 1,906.3 ms | 1,896.8 ms | −0.63% | [−24.26%, +26.05%] |
| `markdown-prose` (cold) | 1,834.4 ms | 2,084.9 ms | +6.42% | [−2.97%, +22.75%] |

The content digest is identical across arms on every trial, no sample was invalidated,
and the tree did not move.
Peak RSS is flat (181 → 179 MiB on the cache hit).
The saving is user CPU — 543 ms → 372 ms on `content-cache-hit` with system CPU
unchanged at 30 ms — which is what removing comparisons should look like.

## Where the prediction was wrong

The registry predicted at least 15% on `content-cache-hit` and measured 31%: the
profile’s 33% path share was the floor of the saving, not the ceiling, because
`compare_components` also drags allocation and `Components` iteration in with it.

The `content-query` result was not predicted and is not this hypothesis’s verdict.
It is the same mechanism in a different place: the metrics views look each file’s record
up by path (`query_report.rs`, `content.file(&file.path)`), so a hundred summaries over
52k files are five million descents, and each one now compares bytes.
It is recorded here as an observed effect; a claim about query latency needs its own
round, and the size of this one says that round is worth running.

The four cold jobs are unchanged, as they should be — their cost is reading 1 GiB of
file bodies — and their intervals are ±20–28 points wide, which is the host, not the
change. `markdown-prose`’s +6.42% sits inside an interval that reaches −2.97% and is not
a regression the change could cause: the cold path performs the same map operations the
warm path does, only fewer times per file.

## What is left on this tier

`Path::hash` and SipHash are the next 8% of the warm profile — the roll-up `HashMap`
keyed by `PathBuf`, and the candidate map the sidecar loader builds — and they want the
same treatment with a byte-keyed hasher.
The structural form, roll-ups keyed by `EntryId` and computed in one bottom-up pass, is
`fdu-jxhk`.

## Regime

Exploratory, warm-steady, uncontrolled (the runaway `ANECompilerService` noted in
exp-066 was still present).
The warm intervals are narrow despite it because the effect is large relative to the
noise; the cold intervals show what the noise is.
