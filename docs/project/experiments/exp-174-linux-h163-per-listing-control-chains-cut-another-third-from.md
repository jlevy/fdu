---
title: Linux H163 per-listing control chains cut another third from the default summary
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-174
  title: Linux H163 per-listing control chains cut another third from the default summary
  date: "2026-09-28"
  hypotheses:
    - H163
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
    control: 892cef40 probe (H162)
    candidate: "a15b20f4 probe: governing controls resolved once per listing"
    control_binary:
      name: control
      sha256: a731ffa865bc86b1d888f1bf291c8678db1409b8dc301bd3ceb2b00fd330c5a3
      size_bytes: 3735784
      args: []
    candidate_binary:
      name: candidate
      sha256: 058b9e2ab27ab9c4d85a42bac8d2d20766b827696407f3226eaccec6b2918a96
      size_bytes: 3744176
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-174/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 260734920.0
          candidate_median: 166453887.5
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.124
          change_pct: -36.434
          ci95_low_pct: -38.581
          ci95_high_pct: -28.979
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 256849033.0
          candidate_median: 163037332.0
          control_p95_over_median: 1.111
          candidate_p95_over_median: 1.124
          change_pct: -36.816
          ci95_low_pct: -39.058
          ci95_high_pct: -29.23
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 487568000.0
          candidate_median: 401779500.0
          control_p95_over_median: 1.135
          candidate_p95_over_median: 1.11
          change_pct: -17.331
          ci95_low_pct: -19.954
          ci95_high_pct: -12.397
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 297668000.0
          candidate_median: 224901500.0
          control_p95_over_median: 1.151
          candidate_p95_over_median: 1.109
          change_pct: -26.219
          ci95_low_pct: -32.893
          ci95_high_pct: -19.907
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 186483000.0
          candidate_median: 180178500.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.144
          change_pct: -2.336
          ci95_low_pct: -7.724
          ci95_high_pct: 8.331
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
          - peak_rss_bytes is missing a paired percent interval
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 312210991.5
          candidate_median: 211331295.5
          control_p95_over_median: 1.093
          candidate_p95_over_median: 1.156
          change_pct: -35.859
          ci95_low_pct: -39.141
          ci95_high_pct: -21.496
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 306440426.0
          candidate_median: 204871807.5
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.161
          change_pct: -36.241
          ci95_low_pct: -39.533
          ci95_high_pct: -22.494
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 515081000.0
          candidate_median: 423710000.0
          control_p95_over_median: 1.049
          candidate_p95_over_median: 1.034
          change_pct: -19.304
          ci95_low_pct: -22.684
          ci95_high_pct: -16.88
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 333584500.0
          candidate_median: 229875500.0
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.101
          change_pct: -32.116
          ci95_low_pct: -36.226
          ci95_high_pct: -22.783
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 175565000.0
          candidate_median: 185802000.0
          control_p95_over_median: 1.238
          candidate_p95_over_median: 1.112
          change_pct: 2.959
          ci95_low_pct: -9.627
          ci95_high_pct: 12.466
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 37806080.0
          candidate_median: 37029888.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.024
          change_pct: -2.053
          ci95_low_pct: -4.333
          ci95_high_pct: -1.267
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
  reference_tools: []
  complexity:
    lines_changed: 104
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -36.434
    reason: "quiet linux-v6.12 aggregate-summary -36.43% [-38.58%, -28.98%], default-tree -35.86%; no-controls placebo includes zero"
    commit: a15b20f4
    kept: candidate
---
## What was predicted

H163: after H162, per-entry classification still walks each entry’s ancestors through
the control table’s `BTreeMap<PathBuf>`, comparing paths component by component, and a
listing’s children all repeat the same walk.
`ControlTable::chain_for` resolves a directory’s governing controls once, owning them;
`ControlChain::is_ignored` matches each child by slicing the directory’s components plus
its name. The detached index builder resolves one chain per listing after applying that
directory’s own control, and the transient summary fold caches the chain for its last
parent and drops it on every table change.
Watch and reconcile keep the per-entry matcher, which a test shows the chain agrees
with.

Named before timing, in `fdu-356n`: `aggregate-summary` wall down at least 3% with the
interval below zero on `linux-v6.12` against H162, peak RSS non-inferior, and both arms
with `--no-controls` as the placebo.

## What was measured

Quiet 12-pair probe run on `linux-v6.12`: control `892cef40` (H162), candidate
`a15b20f4`, and each with `--no-controls`. No sample was invalid.

- `aggregate-summary`: −36.43% [−38.58%, −28.98%] (260.7 → 166.5 ms).
  Accept.
- `default-tree`: −35.86% [−39.14%, −21.50%] (312.2 → 211.3 ms).
- Placebo, both arms `--no-controls`: `aggregate-summary` −2.17% [−7.51%, +3.65%],
  `default-tree` −4.06% [−8.84%, +0.26%].
- On `linux-balanced-1m`, which has no `.gitignore`, H162 and H163 together moved
  neither job (exp-173’s second placebo).

## Decision

Accepted. With H162 the default summary on this tree fell from 505 to 167 ms and the
default tree from 590 to 211 ms, but the tree still takes about three times pdu’s 70 ms
and the summary 2.5 times fdu’s own `--no-controls` walk.
What remains is matching on one thread; H164 proposes classifying on the walkers.
