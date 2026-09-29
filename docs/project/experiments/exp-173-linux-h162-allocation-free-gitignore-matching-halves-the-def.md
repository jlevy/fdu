---
title: Linux H162 allocation-free gitignore matching halves the default summary on a source tree
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-173
  title: Linux H162 allocation-free gitignore matching halves the default summary on a source tree
  date: "2026-09-28"
  hypotheses:
    - H162
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
    control: "aa58a6b1 probe (H159 layer): per-entry heap rows"
    candidate: "892cef40 probe: stack-buffered components, fixed-length fast path, stack DP rows"
    control_binary:
      name: control
      sha256: fadb2e564753663a03663fbd3d8f33d474020d837bbde8f589a9b8e17b5de02b
      size_bytes: 3735352
      args: []
    candidate_binary:
      name: candidate
      sha256: a731ffa865bc86b1d888f1bf291c8678db1409b8dc301bd3ceb2b00fd330c5a3
      size_bytes: 3735784
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-173/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 504851480.0
          candidate_median: 260244409.0
          control_p95_over_median: 1.137
          candidate_p95_over_median: 1.063
          change_pct: -47.021
          ci95_low_pct: -52.426
          ci95_high_pct: -44.348
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 500445583.0
          candidate_median: 255360919.0
          control_p95_over_median: 1.138
          candidate_p95_over_median: 1.069
          change_pct: -47.709
          ci95_low_pct: -52.762
          ci95_high_pct: -44.85
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 737007500.0
          candidate_median: 500712500.0
          control_p95_over_median: 1.088
          candidate_p95_over_median: 1.03
          change_pct: -31.809
          ci95_low_pct: -36.124
          ci95_high_pct: -29.251
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 565811500.0
          candidate_median: 310884000.0
          control_p95_over_median: 1.063
          candidate_p95_over_median: 1.107
          change_pct: -45.142
          ci95_low_pct: -48.519
          ci95_high_pct: -40.127
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 178007000.0
          candidate_median: 183804500.0
          control_p95_over_median: 1.126
          candidate_p95_over_median: 1.096
          change_pct: 1.229
          ci95_low_pct: -3.41
          ci95_high_pct: 9.494
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 29083648.0
          candidate_median: 28839936.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          noninferiority: unknown
          pairs: 0
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
          control_median: 590130354.5
          candidate_median: 320285656.5
          control_p95_over_median: 1.121
          candidate_p95_over_median: 1.146
          change_pct: -46.192
          ci95_low_pct: -48.676
          ci95_high_pct: -44.962
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 581735254.5
          candidate_median: 314986721.5
          control_p95_over_median: 1.129
          candidate_p95_over_median: 1.144
          change_pct: -46.493
          ci95_low_pct: -49.181
          ci95_high_pct: -45.445
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 770205500.0
          candidate_median: 522617000.0
          control_p95_over_median: 1.101
          candidate_p95_over_median: 1.106
          change_pct: -31.683
          ci95_low_pct: -34.664
          ci95_high_pct: -28.116
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 576203000.0
          candidate_median: 339111000.0
          control_p95_over_median: 1.122
          candidate_p95_over_median: 1.096
          change_pct: -41.849
          ci95_low_pct: -45.378
          ci95_high_pct: -39.966
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 186124000.0
          candidate_median: 193702500.0
          control_p95_over_median: 1.159
          candidate_p95_over_median: 1.125
          change_pct: 3.155
          ci95_low_pct: -4.871
          ci95_high_pct: 8.77
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 38846464.0
          candidate_median: 37959680.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.006
          change_pct: -2.743
          ci95_low_pct: -3.509
          ci95_high_pct: -2.094
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
    lines_changed: 150
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -47.021
    reason: "quiet linux-v6.12 aggregate-summary -47.02% [-52.43%, -44.35%], default-tree -46.19%; no-controls and no-gitignore placebos include zero; allocations 7.2M to 402k"
    commit: 892cef40
    kept: candidate
---
## What was predicted

H162: on a source tree with many `.gitignore` files, the default summary and tree are
bound by ignore classification on their one classifying thread, and most of that cost is
allocation, not matching.
Comparing fdu with pdu on reconstructible `linux-v6.12` (358 `.gitignore` files, 1,593
rules) found the default tree at 564 ms and the default summary at 490 ms against 70 ms
for pdu and 65–76 ms for fdu with `--no-gitignore` (hyperfine screens,
[the pdu brief](../research/research-2026-09-28-pdu-and-the-linux-peer-gap.md)).
`FDU_COUNTERS=1` counted 7.2 million allocations with classification on and 388,225
without, for 92,474 entries.
`Gitignore::matches` collected each relative path into a `Vec`, and
`segment_path_matches` allocated a fresh row per call and per segment for every anchored
or path pattern, once per control file on each entry’s path.

The change (`892cef40`) collects components into a 32-slot stack buffer, matches a
pattern without `**` only against a path of its own length, segment for segment, and
keeps the `**` recurrence on 64-slot stack rows.
A differential test pins both paths to the old heap-row algorithm at depths that
straddle both buffers.

Named before timing, in `fdu-oltm`: `aggregate-summary` (controls on) wall down at least
3% with the interval below zero on `linux-v6.12`, peak RSS non-inferior; both arms with
`--no-controls` as a placebo, and `linux-balanced-1m`, which holds no `.gitignore`, as a
second placebo.

## What was measured

Quiet 12-pair probe run on `linux-v6.12`, four variants: control and candidate, and each
with `--no-controls`. No sample was invalid and the tree did not change.

- `aggregate-summary`: −47.02% [−52.43%, −44.35%] (504.9 → 260.2 ms).
  Accept.
- `default-tree`: −46.19% [−48.68%, −44.96%] (590.1 → 320.3 ms).
- Placebo, both arms `--no-controls`: `aggregate-summary` +0.21% [−6.89%, +2.89%],
  `default-tree` +0.55% [−6.00%, +2.97%].
- `linux-balanced-1m` placebo (this layer against H163, which adds nothing on a tree
  without `.gitignore`): `aggregate-summary` +1.34% [−4.64%, +3.47%], `default-tree`
  +0.78% [−2.27%, +2.00%]
  ([evidence/exp-173/balanced-placebo-run.json](evidence/exp-173/balanced-placebo-run.json)).

A screen of the CLI on the same tree counted 401,995 allocations after the change,
against 7,216,227 before.

## Decision

Accepted. Classification still costs about 190 ms above the `--no-controls` walk on this
tree; H163 takes the per-entry ancestor lookups next.
The change is platform-neutral Rust and unmeasured on macOS, where exp-170’s trees hold
59 and zero `.gitignore` files.
