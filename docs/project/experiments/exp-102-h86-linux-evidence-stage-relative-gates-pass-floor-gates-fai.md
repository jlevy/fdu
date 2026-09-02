---
title: "H86 Linux evidence stage: relative gates pass, floor gates fail"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-102
  title: "H86 Linux evidence stage: relative gates pass, floor gates fail"
  date: "2026-09-02"
  hypotheses:
    - H86
  subject:
    tree_label: linux-450k
    tree_root_id: 65b45ec723560c09be8165ad08a7b7c33cc048c2713b4b8c9b2a906973b9286d
    tree_engine_digest: e6da049852a2172f6b73202db295564076a1dc2868438444d3322f322b414a95
    tree_provenance: "Generated balanced recipe, 450,001 entries, manifest 65aa72b53d5fdae1665d66451b0071b1605f015e8657da69697a25d928dbed6d, semantic digest 0c5230889cbe6ee25ceb6e64560cb012bccd03126565fd8f8d313e7013715e3d, engine digest e6da049852a2172f6b73202db295564076a1dc2868438444d3322f322b414a95"
    tree_reconstructible: true
    tree_entries: 450001
    tree_directories: 56251
    tree_files: 393750
    tree_symlinks: 0
    tree_apparent_bytes: 358665192
    tree_allocated_bytes: 1344430080
    tree_max_depth: 7
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.80GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16856133632
    host_system: Linux 6.18.44-fc-v22
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: c6380f7 immediate immutable control
    candidate: 5d7b86f H86 consumer representation (codex/streaming-performance-parity)
    control_binary:
      name: control
      sha256: 6f541256f006029a127b9f7dfcc95294464f7db79da92ca3f510be5900894758
      size_bytes: 2594584
      args: []
    candidate_binary:
      name: candidate
      sha256: d6fbcec2827f5ea277ec769e71c03f3386e85cb9e5f59676247cd9575c7dd534
      size_bytes: 2710552
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /home/user/perf/results/run-h86-linux-immediate.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1905623645.0
          candidate_median: 1537022159.5
          control_p95_over_median: 1.134
          candidate_p95_over_median: 1.109
          change_pct: -18.165
          ci95_low_pct: -24.254
          ci95_high_pct: -13.719
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 854276236.5
          candidate_median: 614503395.5
          control_p95_over_median: 1.305
          candidate_p95_over_median: 1.073
          change_pct: -25.838
          ci95_low_pct: -34.415
          ci95_high_pct: -19.059
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 3470719000.0
          candidate_median: 2844123000.0
          control_p95_over_median: 1.071
          candidate_p95_over_median: 1.096
          change_pct: -17.971
          ci95_low_pct: -21.474
          ci95_high_pct: -13.139
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 1948230000.0
          candidate_median: 1564570500.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.047
          change_pct: -20.496
          ci95_low_pct: -25.032
          ci95_high_pct: -16.158
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1519554500.0
          candidate_median: 1317559500.0
          control_p95_over_median: 1.118
          candidate_p95_over_median: 1.124
          change_pct: -12.201
          ci95_low_pct: -21.926
          ci95_high_pct: -3.168
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 318380032.0
          candidate_median: 161003520.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.08
          change_pct: -49.163
          ci95_low_pct: -52.606
          ci95_high_pct: -46.158
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1189662207.5
          candidate_median: 821671459.5
          control_p95_over_median: 1.185
          candidate_p95_over_median: 1.089
          change_pct: -31.704
          ci95_low_pct: -34.313
          ci95_high_pct: -29.153
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 1081785914.5
          candidate_median: 803657035.0
          control_p95_over_median: 1.193
          candidate_p95_over_median: 1.088
          change_pct: -26.685
          ci95_low_pct: -29.923
          ci95_high_pct: -24.005
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 2769353500.0
          candidate_median: 2119954500.0
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.137
          change_pct: -19.946
          ci95_low_pct: -23.879
          ci95_high_pct: -17.303
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 1230775500.0
          candidate_median: 803619500.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.031
          change_pct: -36.439
          ci95_low_pct: -39.325
          ci95_high_pct: -28.87
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1514813500.0
          candidate_median: 1347282500.0
          control_p95_over_median: 1.194
          candidate_p95_over_median: 1.124
          change_pct: -7.961
          ci95_low_pct: -12.691
          ci95_high_pct: -5.166
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 328824832.0
          candidate_median: 210688000.0
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.052
          change_pct: -35.048
          ci95_low_pct: -37.681
          ci95_high_pct: -33.123
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 29410223182.0
          candidate_median: 26082340091.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.053
          change_pct: -10.728
          ci95_low_pct: -13.967
          ci95_high_pct: -8.239
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 21665766319.5
          candidate_median: 20507452819.0
          control_p95_over_median: 1.047
          candidate_p95_over_median: 1.055
          change_pct: -5.465
          ci95_low_pct: -7.888
          ci95_high_pct: -2.387
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 32085253500.0
          candidate_median: 29809111000.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.052
          change_pct: -7.195
          ci95_low_pct: -10.028
          ci95_high_pct: -3.771
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 22163574500.0
          candidate_median: 21005891000.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.041
          change_pct: -5.318
          ci95_low_pct: -8.517
          ci95_high_pct: -2.065
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 10043974500.0
          candidate_median: 8765304000.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.085
          change_pct: -11.158
          ci95_low_pct: -14.88
          ci95_high_pct: -9.119
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 1264730112.0
          candidate_median: 1084055552.0
          control_p95_over_median: 1.0
          candidate_p95_over_median: 1.002
          change_pct: -14.283
          ci95_low_pct: -14.294
          ci95_high_pct: -14.26
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - "absolute floor ratio, not paired regression"
    notes: No code change proposed or made; this is an evidence stage against an existing candidate.
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -18.165
    reason: "Relative gates pass (cold-scan-index -18.16% [-24.25,-13.72], default-tree -31.70%, RSS -49.4%/-35.9%, tails inside 1.5/2.0), but the pre-registered Linux floor gates fail: index wall 4.86x the parfloor syscall floor against a 1.4x gate and 5.03x arena_spike RSS against a 3x gate. Both floor cells are stable (max/min 1.204 and 1.391), so the ratios are resolved and reject rather than unresolved."
    commit: 5d7b86fe6d031e76843fe0b8dbcf8663a0d2b53f
---
The Linux second evidence stage for H86, run on the 450,001-entry generated `balanced`
subject (56,251 directories / 393,750 files) on a 4-core KVM Xeon.
It is the stage the campaign has owed since the arena_spike ceiling was measured, and
the delegate’s macOS host could not supply it while blocked at 127 MiB free.

The relative gates pass and pass well.
Against the immediate control `c6380f7`, across twelve paired interleaved trials with
zero invalid samples: `cold-scan-index` wall fell 18.16% (95% interval -24.25% to
-13.72%) with peak RSS down 49.4%; `default-tree` wall fell 31.70% (-34.31% to -29.15%)
with peak RSS down 35.9%; and `opened-discovery`, which only had to stay noninferior
within +3%, improved 10.73% (-13.97% to -8.24%). The candidate’s tail ratios are inside
the pre-registered bounds everywhere: `p95/median` at most 1.109 and `max/min` at most
1.324, against limits of 1.5 and 2.0. The engine digest was verified identical across
all three binaries at worker counts one through four before any timing, and the post-run
tree digest is unchanged.

The absolute floor gates fail, and that is the finding.
`parfloor stat` at four workers gives a parallel syscall floor of 316.4 ms;
`arena_spike` under its pre-registered low-churn warm-steady cell gives 362.8 ms and
30.5 MiB. The candidate’s `cold-scan-index` is 1,537.0 ms, or 4.86x the syscall floor
against a 1.4x gate, and 153.5 MiB, or 5.03x spike RSS against a 3x gate.
`default-tree` is 2.60x the floor and 6.59x spike RSS. H86 moved these a long way -- the
control measured 6.02x/9.96x and 3.76x/10.28x -- but not to the gate.

The rejection is not an artifact of a noisy denominator.
The plan’s escape hatch applies only when the prepared spike cell itself has `max/min`
above 2.0, and both floor cells are tight: `arena_spike` 1.204 and `parfloor` 1.391. The
ratios are therefore resolved and can reject.

The mechanism this leaves behind is the reusable part.
`parfloor` at 316 ms and `arena_spike` at 363 ms differ by only about 15%, so retaining
an index-shaped result over raw parallel enumeration is nearly free at the floor.
The candidate’s `default-tree` is 822 ms.
About 2.6x of consumer-side headroom therefore remains on Linux, and none of it is in
the syscall layer -- which is where the campaign has repeatedly been tempted to look.

Threat to validity, declared before measuring rather than after: this is a shared cloud
KVM with the agent process resident, recorded as `exploratory` stage and `uncontrolled`
host regime. It is strong enough to reject an absolute floor claim, which is a ratio
against denominators measured on the same host in the same session.
It is not a quiet-host verdict and does not substitute for one, and per
platform-tuning.md it makes no bare-metal claim.
