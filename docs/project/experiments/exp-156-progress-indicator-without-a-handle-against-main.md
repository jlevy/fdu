---
title: Progress indicator without a handle against main
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-156
  title: Progress indicator without a handle against main
  date: "2026-09-24"
  hypotheses:
    - H150
  subject:
    tree_label: system-private-frameworks
    tree_root_id: b718281f3051a0ed5b4fc59d83614845f67e17999095cf2d837a0c551e24869c
    tree_engine_digest: 0c863b0ab28dc47e3db5a0298fe3239a51959056ec5b97c519e49ad1bfd965bf
    tree_provenance: "The sealed macOS system volume's private frameworks, read-only and identical on every Mac running the same OS build (Darwin 25.5.0 here). Reconstructible by installing that build."
    tree_reconstructible: true
    tree_entries: 158705
    tree_directories: 55256
    tree_files: 96542
    tree_symlinks: 6907
    tree_apparent_bytes: 5752378316
    tree_allocated_bytes: 3910119424
    tree_max_depth: 14
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
    trials: 20
    warmups: 3
    interleaved: true
    control: main at 0059ddd5
    candidate: progress-indicator branch at ead98807 with no handle attached
    control_binary:
      name: control
      sha256: d728256165531c9bff31995d68e5ea2b5468d74b36aeed448b64c2a97bd5ed9d
      size_bytes: 2834624
      args: []
    candidate_binary:
      name: candidate
      sha256: 72f2e7cd7f8b14be1ba4e9c7c3ffb6d7712aa0b338554d73ad7cb5c48c2b017b
      size_bytes: 2884256
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /private/tmp/fdu-prog/perf/results/run-exp-156-progress-no-handle-vs-main.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2416353729.5
          candidate_median: 2201355791.5
          control_p95_over_median: 1.236
          candidate_p95_over_median: 1.373
          change_pct: -3.042
          ci95_low_pct: -13.842
          ci95_high_pct: 2.92
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 2409701417.0
          candidate_median: 2194660104.0
          control_p95_over_median: 1.236
          candidate_p95_over_median: 1.37
          change_pct: -3.064
          ci95_low_pct: -13.941
          ci95_high_pct: 2.888
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 16386646500.0
          candidate_median: 15074405500.0
          control_p95_over_median: 1.326
          candidate_p95_over_median: 1.407
          change_pct: -6.993
          ci95_low_pct: -19.014
          ci95_high_pct: 5.234
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 338016000.0
          candidate_median: 326522500.0
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.099
          change_pct: -2.945
          ci95_low_pct: -6.25
          ci95_high_pct: -0.139
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 20
        system_cpu_ns:
          control_median: 16049320500.0
          candidate_median: 14740108000.0
          control_p95_over_median: 1.331
          candidate_p95_over_median: 1.414
          change_pct: -7.097
          ci95_low_pct: -19.263
          ci95_high_pct: 5.403
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 72212480.0
          candidate_median: 72458240.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.021
          change_pct: 0.683
          ci95_low_pct: 0.17
          ci95_high_pct: 1.455
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 20
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
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2494082562.5
          candidate_median: 2432490187.5
          control_p95_over_median: 1.149
          candidate_p95_over_median: 2.155
          change_pct: -1.713
          ci95_low_pct: -8.119
          ci95_high_pct: 8.21
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 2108358374.5
          candidate_median: 1997392270.5
          control_p95_over_median: 1.212
          candidate_p95_over_median: 1.816
          change_pct: -2.688
          ci95_low_pct: -10.129
          ci95_high_pct: 6.246
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 13977982500.0
          candidate_median: 12381928000.0
          control_p95_over_median: 1.424
          candidate_p95_over_median: 1.432
          change_pct: -8.251
          ci95_low_pct: -17.614
          ci95_high_pct: 5.329
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 620553500.0
          candidate_median: 615434000.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.068
          change_pct: -0.515
          ci95_low_pct: -1.645
          ci95_high_pct: 0.681
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        system_cpu_ns:
          control_median: 13354538500.0
          candidate_median: 11763144500.0
          control_p95_over_median: 1.443
          candidate_p95_over_median: 1.454
          change_pct: -8.715
          ci95_low_pct: -18.624
          ci95_high_pct: 5.493
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 72769536.0
          candidate_median: 72925184.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.018
          change_pct: 0.279
          ci95_low_pct: -0.359
          ci95_high_pct: 0.91
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
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 4724448271.0
          candidate_median: 4664527104.0
          control_p95_over_median: 1.151
          candidate_p95_over_median: 1.101
          change_pct: -1.098
          ci95_low_pct: -7.318
          ci95_high_pct: 6.633
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 2139274791.5
          candidate_median: 2137305604.0
          control_p95_over_median: 1.218
          candidate_p95_over_median: 1.136
          change_pct: -1.327
          ci95_low_pct: -8.449
          ci95_high_pct: 4.59
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 29253960000.0
          candidate_median: 29071522000.0
          control_p95_over_median: 1.274
          candidate_p95_over_median: 1.216
          change_pct: 7.82
          ci95_low_pct: -14.332
          ci95_high_pct: 15.702
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 840110500.0
          candidate_median: 848783500.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.027
          change_pct: 0.837
          ci95_low_pct: -0.649
          ci95_high_pct: 3.269
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 28410618500.0
          candidate_median: 28222112000.0
          control_p95_over_median: 1.283
          candidate_p95_over_median: 1.222
          change_pct: 7.987
          ci95_low_pct: -14.667
          ci95_high_pct: 16.197
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 75456512.0
          candidate_median: 76218368.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.028
          change_pct: 1.468
          ci95_low_pct: -0.184
          ci95_high_pct: 2.141
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
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2531465374.5
          candidate_median: 2523644250.0
          control_p95_over_median: 1.091
          candidate_p95_over_median: 1.136
          change_pct: -0.59
          ci95_low_pct: -4.1
          ci95_high_pct: 1.796
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 46113479.5
          candidate_median: 46255354.0
          control_p95_over_median: 1.322
          candidate_p95_over_median: 1.399
          change_pct: 5.918
          ci95_low_pct: -4.881
          ci95_high_pct: 18.354
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 13951650500.0
          candidate_median: 13155148000.0
          control_p95_over_median: 1.275
          candidate_p95_over_median: 1.381
          change_pct: -0.528
          ci95_low_pct: -6.268
          ci95_high_pct: 16.007
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 640398000.0
          candidate_median: 637106000.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.029
          change_pct: -1.329
          ci95_low_pct: -2.254
          ci95_high_pct: -0.481
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 20
        system_cpu_ns:
          control_median: 13306670500.0
          candidate_median: 12520316500.0
          control_p95_over_median: 1.288
          candidate_p95_over_median: 1.398
          change_pct: -0.516
          ci95_low_pct: -6.472
          ci95_high_pct: 16.695
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 88391680.0
          candidate_median: 89268224.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.011
          change_pct: 1.148
          ci95_low_pct: 0.969
          ci95_high_pct: 1.56
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 20
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2469352437.5
          candidate_median: 2457579042.0
          control_p95_over_median: 1.083
          candidate_p95_over_median: 1.111
          change_pct: -1.777
          ci95_low_pct: -6.327
          ci95_high_pct: 2.897
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 2461129896.0
          candidate_median: 2449452292.0
          control_p95_over_median: 1.084
          candidate_p95_over_median: 1.112
          change_pct: -1.823
          ci95_low_pct: -6.348
          ci95_high_pct: 2.914
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 18300896500.0
          candidate_median: 17995330000.0
          control_p95_over_median: 1.169
          candidate_p95_over_median: 1.229
          change_pct: -6.734
          ci95_low_pct: -10.244
          ci95_high_pct: 4.246
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 361891500.0
          candidate_median: 356625500.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.05
          change_pct: -4.056
          ci95_low_pct: -5.289
          ci95_high_pct: -1.018
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 17944063500.0
          candidate_median: 17646397000.0
          control_p95_over_median: 1.171
          candidate_p95_over_median: 1.232
          change_pct: -6.777
          ci95_low_pct: -10.412
          ci95_high_pct: 4.416
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 88276992.0
          candidate_median: 89358336.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.021
          change_pct: 1.585
          ci95_low_pct: 0.903
          ci95_high_pct: 2.226
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 20
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
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1901401770.5
          candidate_median: 1755332021.0
          control_p95_over_median: 2.187
          candidate_p95_over_median: 1.461
          change_pct: -4.677
          ci95_low_pct: -12.876
          ci95_high_pct: 4.861
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 1468656042.0
          candidate_median: 1344080854.5
          control_p95_over_median: 2.278
          candidate_p95_over_median: 1.461
          change_pct: -5.59
          ci95_low_pct: -15.507
          ci95_high_pct: 1.453
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 5335369000.0
          candidate_median: 5183553500.0
          control_p95_over_median: 1.207
          candidate_p95_over_median: 1.361
          change_pct: 0.024
          ci95_low_pct: -4.794
          ci95_high_pct: 10.453
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 725280500.0
          candidate_median: 717014500.0
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.034
          change_pct: -0.059
          ci95_low_pct: -0.731
          ci95_high_pct: 0.982
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        system_cpu_ns:
          control_median: 4610203000.0
          candidate_median: 4459403000.0
          control_p95_over_median: 1.221
          candidate_p95_over_median: 1.427
          change_pct: -0.019
          ci95_low_pct: -5.481
          ci95_high_pct: 12.141
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 89833472.0
          candidate_median: 89899008.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.004
          change_pct: 0.046
          ci95_low_pct: -0.073
          ci95_high_pct: 0.274
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
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 480684021.0
          candidate_median: 486010583.5
          control_p95_over_median: 2.702
          candidate_p95_over_median: 3.671
          change_pct: 5.773
          ci95_low_pct: -0.531
          ci95_high_pct: 30.436
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 105908125.0
          candidate_median: 109674333.5
          control_p95_over_median: 1.989
          candidate_p95_over_median: 1.552
          change_pct: 0.573
          ci95_low_pct: -7.016
          ci95_high_pct: 4.81
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 416614500.0
          candidate_median: 412216000.0
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.105
          change_pct: 0.975
          ci95_low_pct: -0.113
          ci95_high_pct: 1.801
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 389088000.0
          candidate_median: 390025500.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.076
          change_pct: 1.215
          ci95_low_pct: 0.014
          ci95_high_pct: 2.355
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 20
        system_cpu_ns:
          control_median: 25798000.0
          candidate_median: 24041500.0
          control_p95_over_median: 1.444
          candidate_p95_over_median: 1.493
          change_pct: -5.578
          ci95_low_pct: -16.522
          ci95_high_pct: 5.086
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        blocked_ns:
          control_median: 62409021.0
          candidate_median: 69891583.5
          control_p95_over_median: 13.566
          candidate_p95_over_median: 19.215
          change_pct: 42.142
          ci95_low_pct: -5.24
          ci95_high_pct: 224.222
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 81076224.0
          candidate_median: 81272832.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.001
          change_pct: 0.222
          ci95_low_pct: 0.161
          ci95_high_pct: 0.303
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 20
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
  reference_tools:
    - name: dust
      wall_ns_median: 1843166750.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 1173
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Whole engine diff against main in crates/fdu-core/src (1139 insertions, 34 deletions), tests and docs included; the no-handle path adds one Option check per walker chunk or directory."
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -1.777
    reason: "default-tree wall -1.78% [-6.33%, +2.90%], noninferior at +3%; all seven metadata job intervals include zero; uncontrolled, busy host"
    commit: ead98807
    kept: candidate
---
# Progress indicator without a handle against main

## What was predicted

H150: the progress indicator (`fdu-vngp`) adds a `Progress` handle to the engine,
carried on `ScanConfig::progress`. With no handle attached, which is every existing
probe mode and every non-interactive `fdu` run, a walker pays one `Option` check per
chunk of directories it hands over (per directory on revalidation), a phase store is
skipped, and the serial walk sends batches through a closure.
Predicted: no measurable change.
Gate, fixed before the run: on every measured metadata job the paired wall interval
includes zero, and on the primary job, `default-tree`, its upper bound is at most +3%
(the loop’s noninferiority margin).

## What was changed

Nothing new in this record: it measures the `claude/progress-indicator` branch (#120,
stacked on #119) at `ead98807`, which holds the handle, its walk counters, phases, and
the probe’s `--progress` flag (not used here), against `main` at `0059ddd5`. Both probes
were built `--release -p fdu-core --example perf_probe --no-default-features`, the
control in a detached worktree of `origin/main` with its own target directory, and both
were copied outside every tree before the run.

## What was measured

Subject: `system-private-frameworks` (sealed system volume, 158,705 entries / 96,542
files / 55,256 directories), chosen because it cannot drift and because it is 35%
directories, the most per-directory work a no-handle check could add.
Fingerprint re-observed to the nominated digest `0c863b0a…`; no mutation, no baseline
drift.

Jobs: `default-tree` plus the six-job metadata default set.
3 warmups, 20 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Labeled **uncontrolled**. `PERF_HOST_REGIME=quiet` refused at the start gate (CPU busy
84.6% > 25%) after about 20 minutes of waiting for the host to settle; other agents’
test suites and `mediaanalysisd` were running.
Initial busy 80.4%; final 94.0%. Thermal `normal`, AC power.
The 25% bar was not lowered.
0 invalid samples.

Control `d7282561…` (2,834,624 bytes); candidate `72f2e7cd…` (2,884,256 bytes).

| Job | Wall change [95% CI] | Noninferior at +3%? |
| --- | ---: | --- |
| `default-tree` (primary) | −1.78% [−6.33%, +2.90%] | yes |
| `aggregate-summary` | −3.04% [−13.84%, +2.92%] | yes |
| `cold-scan-index` | −1.71% [−8.12%, +8.21%] | inconclusive |
| `cold-scan-producer` | −1.10% [−7.32%, +6.63%] | inconclusive |
| `cold-snapshot-save` | −0.59% [−4.10%, +1.80%] | yes |
| `warm-revalidate` | −4.68% [−12.88%, +4.86%] | inconclusive |
| `warm-snapshot-load` | +5.77% [−0.53%, +30.44%] | inconclusive |

Every wall interval includes zero.
None of the four inconclusive rows shows a cost: each lower bound is below zero.
They are too wide, on this host, to bound one at +3%. `warm-snapshot-load` reads no
tree, so no progress site can run on it; its component moved +0.57% [−7.02%, +4.81%].

Two things did move and are stated rather than hidden:

- Peak RSS on `default-tree` +1.58% [+0.90%, +2.23%] (84.2 → 85.2 MiB), and +0.68% to
  +1.15% on the other report and save jobs, with minor faults up by a similar share.
  That is inside the harness’s 5% RSS limit.
  One `FDU_COUNTERS=1` run of each probe on `default-tree` shows the same engine work:
  1,046,576 against 1,046,573 allocations, 243.66 MB against 243.63 MB allocated,
  identical opens, enumeration calls, stats, upserts and roll-up merges.
  So the extra megabyte is not more allocation; the candidate binary is 49.6 KB larger,
  and the rest was not isolated.
- The harness classified `default-tree` as `inconclusive` overall only because voluntary
  context switches “straddle” their limit: that metric is 0 on 17 of 20 samples in each
  arm (the rest 1–35), so its percentage is a ratio of near-zero counts.

## What the determination said

With no handle attached, the progress branch is indistinguishable from `main` on every
metadata job in this cell, and the default command is noninferior at +3%. The claim is
exploratory: an uncontrolled cell on a busy host.
A quiet re-run would be needed to bound the secondary jobs at +3%.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
