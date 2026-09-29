---
title: Linux H157 file fold cuts allocations but misses on the product job after H159
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-191
  title: Linux H157 file fold cuts allocations but misses on the product job after H159
  date: "2026-09-28"
  hypotheses:
    - H157
  subject:
    tree_label: linux-v6.12
    tree_root_id: 14549a49743a72c3c1aadb09f34f8a097211652c22d533110f3c18b91c518d71
    tree_engine_digest: c7a4d447d9bf3ab36d55c385a4bbe3ed367963aca0fc6e8e5671ec3e5ad8b124
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759236097
    tree_allocated_bytes: 1965420544
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16877547520
    host_system: Linux 6.18.44-fc-v37
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: "a15b20f4 probe and CLI: H159 with H162 and H163"
    candidate: "4c283de0: direct file fold and moved walker names"
    control_binary:
      name: control
      sha256: 058b9e2ab27ab9c4d85a42bac8d2d20766b827696407f3226eaccec6b2918a96
      size_bytes: 3744176
      args: []
    candidate_binary:
      name: candidate
      sha256: 5531f5345af4ff0efcbdaae9cfee17e321253a53e23108394ce22d76c0f771e7
      size_bytes: 3741008
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-191/run.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 303179323.0
          candidate_median: 281315604.0
          control_p95_over_median: 1.102
          candidate_p95_over_median: 1.152
          change_pct: -5.513
          ci95_low_pct: -12.125
          ci95_high_pct: -1.287
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 198805260.5
          candidate_median: 170163424.5
          control_p95_over_median: 1.104
          candidate_p95_over_median: 1.096
          change_pct: -14.743
          ci95_low_pct: -19.103
          ci95_high_pct: -7.433
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 608740500.0
          candidate_median: 590943000.0
          control_p95_over_median: 1.113
          candidate_p95_over_median: 1.075
          change_pct: -3.241
          ci95_low_pct: -8.627
          ci95_high_pct: 2.494
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 329758000.0
          candidate_median: 299358000.0
          control_p95_over_median: 1.079
          candidate_p95_over_median: 1.137
          change_pct: -5.91
          ci95_low_pct: -15.994
          ci95_high_pct: -3.835
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 282040500.0
          candidate_median: 287740000.0
          control_p95_over_median: 1.127
          candidate_p95_over_median: 1.087
          change_pct: 3.48
          ci95_low_pct: -5.911
          ci95_high_pct: 9.498
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 33929216.0
          candidate_median: 32894976.0
          control_p95_over_median: 1.06
          candidate_p95_over_median: 1.013
          change_pct: -4.546
          ci95_low_pct: -7.261
          ci95_high_pct: -0.574
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 212596083.0
          candidate_median: 211091183.0
          control_p95_over_median: 1.178
          candidate_p95_over_median: 1.135
          change_pct: -5.105
          ci95_low_pct: -15.25
          ci95_high_pct: 6.433
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 207959535.5
          candidate_median: 205294267.5
          control_p95_over_median: 1.159
          candidate_p95_over_median: 1.135
          change_pct: -4.254
          ci95_low_pct: -15.713
          ci95_high_pct: 5.34
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 535856500.0
          candidate_median: 508671500.0
          control_p95_over_median: 1.055
          candidate_p95_over_median: 1.109
          change_pct: -2.309
          ci95_low_pct: -6.071
          ci95_high_pct: 3.355
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 239021000.0
          candidate_median: 214035000.0
          control_p95_over_median: 1.114
          candidate_p95_over_median: 1.093
          change_pct: -9.887
          ci95_low_pct: -13.17
          ci95_high_pct: -2.279
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 290825000.0
          candidate_median: 297999000.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.15
          change_pct: 5.602
          ci95_low_pct: -1.449
          ci95_high_pct: 12.925
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 35268608.0
          candidate_median: 35463168.0
          control_p95_over_median: 1.039
          candidate_p95_over_median: 1.035
          change_pct: -0.687
          ci95_low_pct: -2.789
          ci95_high_pct: 3.598
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
  reference_tools: []
  complexity:
    lines_changed: 60
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -5.513
    reason: "pre-registered primary, the product indexed-tree contract: control +3% [-1%, +10%] on linux-v6.12 and +1% [-3%, +6%] on balanced-1m; probe cold-scan-index -5.51% [-12.12%, -1.29%] secondary; allocations -40%"
    commit: 4c283de0
    kept: control
---
## What was predicted

H157, reimplemented (`4c283de0`) from exp-161’s description on top of H159: the detached
builder folds a file child straight into its parent’s roll-up through
`InternedRollUp::add_file`, `Index::contribution` builds a file’s contribution with the
same fold, and the walker moves each listed name instead of copying it.
Allocations fell 40% on `/usr` (551k → 331k per 84,579 entries) and an independent
review found no divergence on any route.

Pre-registered in `fdu-o6um` before timing: the product indexed-tree contract
(`fdu --cache off --depth 1 --limit 10`) as primary, paired in the tool harness on
`linux-balanced-1m` and reconstructible `linux-v6.12`, under the accept rule; the
probe’s `cold-scan-index` as secondary.
exp-161 had warned that H157 and H159 draw on one allocator budget, so H159’s arm is the
control.

## What was measured

Control `a15b20f4` (H159 with H162 and H163), candidate `4c283de0`. Quiet cells, 12
pairs, no invalid sample.

- Product contract, CLI against CLI: the control took +3% [−1%, +10%] longer on
  `linux-v6.12` and +1% [−3%, +6%] on `linux-balanced-1m`
  ([linux-v6.12](evidence/exp-191/cli-run-linux-v6.12.json),
  [balanced](evidence/exp-191/cli-run-balanced.json)). Both intervals include zero.
- Probe on `linux-v6.12`: `cold-scan-index` −5.51% [−12.12%, −1.29%]; `default-tree`
  −5.11% [−15.25%, +6.43%].

## Decision

Rejected on its pre-registered primary.
The allocation cut is real, but after H159 its wall effect on the product job is too
small for this host to resolve; exp-161 saw the mirror image, a product-job win with the
probe’s interval crossing zero.
The code stays off the stack, on its branch, for a re-screen after H166 and H167 change
the index tier.
