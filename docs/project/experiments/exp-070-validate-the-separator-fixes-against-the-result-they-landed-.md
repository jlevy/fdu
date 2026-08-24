---
title: Validate the separator fixes against the result they landed on
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-070
  title: Validate the separator fixes against the result they landed on
  date: "2026-08-24"
  hypotheses: []
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
    control: exp-069 accepted binary at 50260a5
    candidate: "f204abb: normalized() on the content key path, for Path equality on Windows"
    control_binary:
      name: control
      sha256: a1d9764c64e166a6a1a0e4e0951811eb9423748ab08955a5c07677c89b9d397a
      size_bytes: 1561440
      args: []
    candidate_binary:
      name: candidate
      sha256: 1091c3407fbedf0cf5254d13da7d8093704db305094dff713a5126c8f03c3111
      size_bytes: 1561440
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-070-validate-separator-fixes.json
  results:
    - job: code-sloc
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2133082042.0
          candidate_median: 2103242375.5
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.123
          change_pct: 0.597
          ci95_low_pct: -5.57
          ci95_high_pct: 6.149
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1818731666.5
          candidate_median: 1780365145.5
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.149
          change_pct: 0.731
          ci95_low_pct: -5.511
          ci95_high_pct: 5.768
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 13176137500.0
          candidate_median: 13191014000.0
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.027
          change_pct: 0.025
          ci95_low_pct: -2.646
          ci95_high_pct: 2.02
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 6267695000.0
          candidate_median: 6269690000.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.016
          change_pct: 0.195
          ci95_low_pct: -0.631
          ci95_high_pct: 0.725
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 6917658500.0
          candidate_median: 6751261500.0
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.071
          change_pct: -0.601
          ci95_low_pct: -6.359
          ci95_high_pct: 2.471
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 151060480.0
          candidate_median: 151715840.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.006
          change_pct: -0.136
          ci95_low_pct: -1.28
          ci95_high_pct: 1.042
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
          control_median: 437537563.0
          candidate_median: 439567208.0
          control_p95_over_median: 1.093
          candidate_p95_over_median: 1.052
          change_pct: -0.724
          ci95_low_pct: -5.264
          ci95_high_pct: 3.643
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 306332521.0
          candidate_median: 302188667.0
          control_p95_over_median: 1.06
          candidate_p95_over_median: 1.058
          change_pct: -0.59
          ci95_low_pct: -4.562
          ci95_high_pct: 2.89
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 428500000.0
          candidate_median: 428536000.0
          control_p95_over_median: 1.049
          candidate_p95_over_median: 1.026
          change_pct: -0.868
          ci95_low_pct: -3.659
          ci95_high_pct: 1.952
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 384567500.0
          candidate_median: 385321000.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.013
          change_pct: -1.053
          ci95_low_pct: -2.84
          ci95_high_pct: 1.577
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 41706000.0
          candidate_median: 41713000.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.273
          change_pct: 1.557
          ci95_low_pct: -6.682
          ci95_high_pct: 5.322
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 9037563.0
          candidate_median: 8476395.5
          control_p95_over_median: 2.086
          candidate_p95_over_median: 2.615
          change_pct: 16.363
          ci95_low_pct: -52.554
          ci95_high_pct: 108.183
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 188121088.0
          candidate_median: 185434112.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.003
          change_pct: -1.476
          ci95_low_pct: -3.105
          ci95_high_pct: 0.217
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
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1820352167.0
          candidate_median: 1878188645.5
          control_p95_over_median: 1.109
          candidate_p95_over_median: 1.181
          change_pct: -0.105
          ci95_low_pct: -4.757
          ci95_high_pct: 6.045
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1517197708.0
          candidate_median: 1529718916.5
          control_p95_over_median: 1.119
          candidate_p95_over_median: 1.23
          change_pct: -0.815
          ci95_low_pct: -5.212
          ci95_high_pct: 5.352
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 10552065500.0
          candidate_median: 10534454500.0
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.099
          change_pct: 4.319
          ci95_low_pct: -6.158
          ci95_high_pct: 6.575
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 2596100000.0
          candidate_median: 2600148500.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.031
          change_pct: -0.841
          ci95_low_pct: -3.094
          ci95_high_pct: 2.53
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 7953631500.0
          candidate_median: 7977174500.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.105
          change_pct: 4.491
          ci95_low_pct: -7.785
          ci95_high_pct: 8.739
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 147144704.0
          candidate_median: 146808832.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.007
          change_pct: 0.101
          ci95_low_pct: -0.842
          ci95_high_pct: 0.701
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
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 452244041.5
          candidate_median: 450784458.5
          control_p95_over_median: 1.092
          candidate_p95_over_median: 1.137
          change_pct: -1.281
          ci95_low_pct: -5.658
          ci95_high_pct: 1.862
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 312934167.0
          candidate_median: 316128104.0
          control_p95_over_median: 1.114
          candidate_p95_over_median: 1.153
          change_pct: -0.934
          ci95_low_pct: -3.971
          ci95_high_pct: 2.23
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 437065500.0
          candidate_median: 431593500.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.059
          change_pct: -0.893
          ci95_low_pct: -3.182
          ci95_high_pct: 0.687
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 389925000.0
          candidate_median: 385050000.0
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.029
          change_pct: -1.025
          ci95_low_pct: -2.649
          ci95_high_pct: 0.64
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 45975000.0
          candidate_median: 48106000.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.234
          change_pct: -0.872
          ci95_low_pct: -11.117
          ci95_high_pct: 8.17
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 13688541.5
          candidate_median: 18477458.0
          control_p95_over_median: 2.743
          candidate_p95_over_median: 2.983
          change_pct: -4.492
          ci95_low_pct: -46.017
          ci95_high_pct: 80.562
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 187293696.0
          candidate_median: 184696832.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.003
          change_pct: -1.272
          ci95_low_pct: -2.858
          ci95_high_pct: 0.28
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
          control_median: 10584786270.5
          candidate_median: 10669697979.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.067
          change_pct: 0.937
          ci95_low_pct: 0.083
          ci95_high_pct: 2.381
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 8814323937.5
          candidate_median: 8894805374.5
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.08
          change_pct: 1.035
          ci95_low_pct: 0.079
          ci95_high_pct: 4.378
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 19436964000.0
          candidate_median: 19104003500.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.052
          change_pct: -1.208
          ci95_low_pct: -5.697
          ci95_high_pct: 2.954
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 11115093000.0
          candidate_median: 11091152500.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.041
          change_pct: 0.327
          ci95_low_pct: -0.251
          ci95_high_pct: 1.266
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 8370843500.0
          candidate_median: 7994797500.0
          control_p95_over_median: 1.093
          candidate_p95_over_median: 1.127
          change_pct: -3.894
          ci95_low_pct: -13.084
          ci95_high_pct: 8.979
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 158859264.0
          candidate_median: 156041216.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.029
          change_pct: -1.417
          ci95_low_pct: -2.495
          ci95_high_pct: -0.181
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
    - job: document-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 445397750.5
          candidate_median: 438290229.5
          control_p95_over_median: 1.497
          candidate_p95_over_median: 1.471
          change_pct: 0.066
          ci95_low_pct: -4.243
          ci95_high_pct: 2.645
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 309068270.5
          candidate_median: 305938333.5
          control_p95_over_median: 1.306
          candidate_p95_over_median: 1.304
          change_pct: -1.039
          ci95_low_pct: -4.764
          ci95_high_pct: 3.632
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 430386500.0
          candidate_median: 422962500.0
          control_p95_over_median: 1.243
          candidate_p95_over_median: 1.24
          change_pct: -0.601
          ci95_low_pct: -2.769
          ci95_high_pct: 2.377
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 386213000.0
          candidate_median: 381464500.0
          control_p95_over_median: 1.239
          candidate_p95_over_median: 1.227
          change_pct: -0.463
          ci95_low_pct: -2.43
          ci95_high_pct: 1.488
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 44930500.0
          candidate_median: 41738500.0
          control_p95_over_median: 1.34
          candidate_p95_over_median: 1.359
          change_pct: -5.777
          ci95_low_pct: -10.231
          ci95_high_pct: 6.16
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 13760625.0
          candidate_median: 12700729.0
          control_p95_over_median: 9.558
          candidate_p95_over_median: 11.691
          change_pct: 5.639
          ci95_low_pct: -54.906
          ci95_high_pct: 38.397
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 187146240.0
          candidate_median: 187047936.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.005
          change_pct: -0.403
          ci95_low_pct: -2.671
          ci95_high_pct: 0.022
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
          control_median: 2066167020.5
          candidate_median: 2052727750.0
          control_p95_over_median: 1.205
          candidate_p95_over_median: 1.622
          change_pct: 6.759
          ci95_low_pct: -4.389
          ci95_high_pct: 10.982
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1738732896.0
          candidate_median: 1712486916.5
          control_p95_over_median: 1.201
          candidate_p95_over_median: 1.65
          change_pct: 6.275
          ci95_low_pct: -5.556
          ci95_high_pct: 12.125
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 9826362000.0
          candidate_median: 9558897000.0
          control_p95_over_median: 1.104
          candidate_p95_over_median: 1.111
          change_pct: -2.301
          ci95_low_pct: -7.543
          ci95_high_pct: 2.029
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 3033821000.0
          candidate_median: 3006945500.0
          control_p95_over_median: 1.128
          candidate_p95_over_median: 1.172
          change_pct: 0.099
          ci95_low_pct: -1.801
          ci95_high_pct: 2.038
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 6669097000.0
          candidate_median: 6201773000.0
          control_p95_over_median: 1.176
          candidate_p95_over_median: 1.211
          change_pct: -4.913
          ci95_low_pct: -11.288
          ci95_high_pct: 3.921
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 152231936.0
          candidate_median: 153255936.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.017
          change_pct: 0.668
          ci95_low_pct: -0.675
          ci95_high_pct: 1.176
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
          control_median: 2025988562.5
          candidate_median: 2036407083.0
          control_p95_over_median: 2.447
          candidate_p95_over_median: 1.682
          change_pct: -2.354
          ci95_low_pct: -27.349
          ci95_high_pct: 12.65
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1695495125.0
          candidate_median: 1703939604.0
          control_p95_over_median: 2.663
          candidate_p95_over_median: 1.795
          change_pct: -1.391
          ci95_low_pct: -26.069
          ci95_high_pct: 13.281
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 9549059000.0
          candidate_median: 9128230500.0
          control_p95_over_median: 1.08
          candidate_p95_over_median: 1.185
          change_pct: -2.661
          ci95_low_pct: -12.215
          ci95_high_pct: 10.039
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 2985081000.0
          candidate_median: 2961742500.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.017
          change_pct: 0.069
          ci95_low_pct: -1.148
          ci95_high_pct: 1.057
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 6605328000.0
          candidate_median: 6199300000.0
          control_p95_over_median: 1.118
          candidate_p95_over_median: 1.259
          change_pct: -3.972
          ci95_low_pct: -16.91
          ci95_high_pct: 16.186
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 152526848.0
          candidate_median: 151822336.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.016
          change_pct: 0.022
          ci95_low_pct: -1.836
          ci95_high_pct: 0.977
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
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -1.281
    reason: "The pre-registered warm job moves -1.28% [-5.66%, +1.86%], inside the non-inferiority margin, so exp-069 -31% still describes what ships; content-query +0.94% [+0.08%, +2.38%] excludes zero but user CPU is flat-to-lower and system CPU falls, so the run bounds the fixes below the margin rather than showing a cost."
    commit: f204abb
---
## Why this run exists

exp-069 was measured at `50260a5`. Two commits then changed the file it measured:
`6c7a099` and `f204abb`, both fixing the same Windows defect — `Path` equality ignores
which separator a component boundary uses where the platform accepts more than one, and
a byte-ordered key does not, so on Windows a lookup by one spelling missed a record
committed under the other and four tests failed there.

`f204abb`’s fix puts a `normalized()` call on `file()`, `commit()` and `invalidate()` —
the exact hot path exp-069’s −31% describes.
Its commit message asserted that this is free on unix, because `normalized`’s first
condition is `MAIN_SEPARATOR != '/'` and the compiler folds it away.
That was an assertion, not a measurement, and this loop does not accept the difference.
exp-055 is the precedent: after review fixes, re-measure before the claim stands.

Control is exp-069’s own accepted binary (`50260a5`, kept at
`/tmp/fdu-realtree/perf_probe.control`); candidate is `f204abb`. The pre-registered
signal, written to `fdu-78q6` before the run, was non-inferiority: the 95% interval’s
upper bound at most +3% on `content-cache-hit` and `content-query`.

## Result

Metabrowser clone, 52,717 files, twelve paired trials, uncontrolled host.

| job | control | candidate | paired change | interval |
| --- | ---: | ---: | ---: | --- |
| `content-cache-hit` (pre-registered) | 452.2 ms | 450.8 ms | −1.28% | [−5.66%, +1.86%] |
| `content-query` (pre-registered) | 10,584.8 ms | 10,669.7 ms | +0.94% | [+0.08%, +2.38%] |
| `code-sloc-cache-hit` | 437.5 ms | 439.6 ms | −0.72% | [−5.26%, +3.64%] |
| `document-cache-hit` | 445.4 ms | 438.3 ms | +0.07% | [−4.24%, +2.65%] |
| `content-basic` | 1,820.4 ms | 1,878.2 ms | −0.10% | [−4.76%, +6.04%] |
| `code-sloc` | 2,133.1 ms | 2,103.2 ms | +0.60% | [−5.57%, +6.15%] |
| `text-prose` | 2,026.0 ms | 2,036.4 ms | −2.35% | [−27.35%, +12.65%] |
| `markdown-prose` | 2,066.2 ms | 2,052.7 ms | +6.76% | [−4.39%, +10.98%] |

No sample was invalidated, the tree did not move, and the content digest is identical
across both arms on every trial of every job.

**The pre-registered warm job is unaffected**: `content-cache-hit` moves −1.28% with an
upper bound of +1.86%, inside the margin.
So exp-069’s −31% still describes what ships.

**`content-query` is the one interval that excludes zero**, at +0.94% [+0.08%, +2.38%] —
detectable by the arithmetic, and inside the pre-registered margin.
It should not be read as the mechanism, and the resource metrics are why: user CPU is
11,115 ms against 11,091 ms — the candidate is *lower* — and system CPU falls from 8,371
ms to 7,995 ms. A change that added per-lookup work would show as user CPU on the job
that does five million lookups.
It did not. A wall shift of under one percent with CPU flat-to-lower, on a host carrying
load average 13, is drift, and the honest statement is that this run bounds the cost of
the fixes below the margin rather than proving it is zero.

`code-sloc-cache-hit`’s upper bound of +3.64% is the one figure outside the ±3% margin.
Its median is −0.72% and its sibling cache-hit jobs sit inside, so it is interval width
on a noisy host rather than a signal, but it is what the run measured and the margin was
declared before the run.

## What this does not settle

`normalized` returns a `Cow<Path>` and decides at runtime on a constant.
A `cfg(unix)` form that is the identity function by construction would be free rather
than free-if-the-optimizer-cooperates, and would retire the question this experiment
could only bound. That is a one-line change and its own round; it is recorded on
`fdu-78q6`.

## Regime

Exploratory, warm-steady, uncontrolled: load average 13 from an ordinary desktop
(WindowServer, two browsers, another agent’s session).
The runaway `ANECompilerService` that shaped exp-066 through exp-069 is gone.
Twelve trials; the cold jobs’ intervals reach ±27 points, which is the host rather than
any of these changes.
