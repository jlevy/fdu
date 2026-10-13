---
title: "Linux: H198, the 0.5.0 candidate against v0.4.0, default tree +7.9% dense, summaries non-inferior"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-223
  title: "Linux: H198, the 0.5.0 candidate against v0.4.0, default tree +7.9% dense, summaries non-inferior"
  date: "2026-10-13"
  hypotheses:
    - H198
  subject:
    tree_label: node-modules-dense
    tree_root_id: ffbd3cb6be62b77f7e2e9b5964d1e380e8a3f3b7bb4fda55f462ae4b189841b5
    tree_engine_digest: 62125c60d0dddf9fdb9f89b8c2542c8afd573f3460ccc6f052626099669b7751
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.0, npm 10.9.4); rebuilt 2026-10-10 on this host: 79,957 entries, 9,439 directories, 107 symlinks, no .gitignore, the counts exp-197 recorded; fingerprint unchanged on 2026-10-13."
    tree_reconstructible: true
    tree_entries: 79957
    tree_directories: 9439
    tree_files: 70411
    tree_symlinks: 107
    tree_apparent_bytes: 538992241
    tree_allocated_bytes: 757870592
    tree_max_depth: 12
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.80GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16874930176
    host_system: Linux 6.18.44-fc-v114
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 100
    warmups: 3
    interleaved: true
    control: c041ed1c probe (v0.4.0)
    candidate: "1c42c91f probe (the 0.5.0 candidate, #202's head)"
    control_binary:
      name: v040
      sha256: 6188c098c55222b5d81f0639333bcb63908f0a7bbf75aa0b6401f1ddca816c60
      size_bytes: 3947224
      args: []
    candidate_binary:
      name: final
      sha256: bdd13649132bf69891d359e0560cf338cf9302a443a62dbd04f07b4700ab7194
      size_bytes: 4268384
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-223/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 94392375.0
          candidate_median: 94520130.5
          control_p95_over_median: 1.214
          candidate_p95_over_median: 1.177
          change_pct: -2.067
          ci95_low_pct: -4.624
          ci95_high_pct: 2.278
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 100
        component_ns:
          control_median: 90766789.5
          candidate_median: 90743164.0
          control_p95_over_median: 1.215
          candidate_p95_over_median: 1.178
          change_pct: -0.856
          ci95_low_pct: -4.213
          ci95_high_pct: 2.457
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 100
        cpu_ns:
          control_median: 331979500.0
          candidate_median: 331631000.0
          control_p95_over_median: 1.201
          candidate_p95_over_median: 1.209
          change_pct: 0.725
          ci95_low_pct: -1.746
          ci95_high_pct: 3.406
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        user_cpu_ns:
          control_median: 83222000.0
          candidate_median: 84245000.0
          control_p95_over_median: 1.364
          candidate_p95_over_median: 1.361
          change_pct: 4.007
          ci95_low_pct: -0.871
          ci95_high_pct: 10.391
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        system_cpu_ns:
          control_median: 246757000.0
          candidate_median: 246775000.0
          control_p95_over_median: 1.225
          candidate_p95_over_median: 1.241
          change_pct: 0.446
          ci95_low_pct: -3.206
          ci95_high_pct: 3.536
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
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
          control_median: 95340119.5
          candidate_median: 102616420.0
          control_p95_over_median: 1.122
          candidate_p95_over_median: 1.143
          change_pct: 7.912
          ci95_low_pct: 5.459
          ci95_high_pct: 12.975
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 100
        component_ns:
          control_median: 90841550.5
          candidate_median: 97831231.0
          control_p95_over_median: 1.128
          candidate_p95_over_median: 1.147
          change_pct: 7.957
          ci95_low_pct: 5.308
          ci95_high_pct: 13.019
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 100
        cpu_ns:
          control_median: 329565000.0
          candidate_median: 344261500.0
          control_p95_over_median: 1.138
          candidate_p95_over_median: 1.169
          change_pct: 5.385
          ci95_low_pct: 1.096
          ci95_high_pct: 9.997
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 100
        user_cpu_ns:
          control_median: 94677500.0
          candidate_median: 101162000.0
          control_p95_over_median: 1.384
          candidate_p95_over_median: 1.239
          change_pct: 6.909
          ci95_low_pct: -0.787
          ci95_high_pct: 15.211
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        system_cpu_ns:
          control_median: 227923000.0
          candidate_median: 243450000.0
          control_p95_over_median: 1.198
          candidate_p95_over_median: 1.229
          change_pct: 7.319
          ci95_low_pct: 2.356
          ci95_high_pct: 12.594
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 100
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
  reference_tools: []
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
    change_pct: 7.912
    reason: "not a speed decision: the 0.5.0 release's Linux standing; two-arm, 100 pairs, quiet: default-tree +7.91% [+5.46%, +12.97%] on node-modules-dense against +3% to +7% predicted, the age column's statx (exp-217, exp-218), and +3.83% [-1.64%, +8.48%] on linux-v6.12 against -2% to +3%; aggregate-summary -2.07% [-4.62%, +2.28%] and -2.06% [-4.47%, +1.82%], non-inferior at +3% on both trees"
    commit: 1c42c91f
    kept: candidate
---
## What was predicted

H198 (A) was registered before any timed sample: `871bc6bd` reached GitHub at 17:59:22Z,
and the first sample ran at 18:10:29Z. It sets the 0.5.0 release candidate’s standing
against v0.4.0 on Linux, as the [#202](https://github.com/jlevy/fdu/pull/202) handoff’s
item 3 asks: `default-tree` and `aggregate-summary` on `node-modules-dense` and
`linux-v6.12`. It is reported as the release’s standing, not a decision.

Predicted:
- `default-tree`: +3% to +7% on the dense tree and −2% to +3% on the kernel tree.
  This is exp-217 and exp-218’s give-back, one `statx` per directory and per symlink.
- `aggregate-summary`: 0 to +6% on the dense tree and −2% to +3% on the kernel tree.

## What was measured

**Host.** The same 4-vCPU Firecracker guest as exp-222 (Intel Xeon at 2.8 GHz, Linux
6.18.44-fc-v114, ext4), as root, warm page cache.

**Regime.** Quiet:
- one two-arm run per tree, 3 warmups and 100 pairs fixed in advance;
- no invalid sample, and each tree verified unchanged across its run.

**Binaries.** Built and hashed before any timing, with nothing else running during the
cells; exp-222’s `binaries.sha256.txt` lists them:
- v0.4.0 is the probe built from the release’s tag commit `c041ed1c` (sha256
  `6188c098`). exp-217 used `148ef78e`, `main` after that release.
- The candidate is `1c42c91f` (sha256 `bdd13649`).

| Tree | Job | v0.4.0 → candidate (median wall) | Paired median [95%] | Mean [95% bootstrap] | Hodges–Lehmann | Prediction |
| --- | --- | --- | --- | --- | --- | --- |
| `node-modules-dense` | `default-tree` | 95.34 → 102.62 ms | **+7.91% [+5.46%, +12.97%]** | +9.51% [+6.87%, +12.37%] | +9.03% | +3% to +7% |
| `node-modules-dense` | `aggregate-summary` | 94.39 → 94.52 ms | −2.07% [−4.62%, +2.28%] | +0.32% [−2.52%, +3.22%] | −0.55% | 0 to +6% |
| `linux-v6.12` | `default-tree` | 107.68 → 111.33 ms | +3.83% [−1.64%, +8.48%] | +4.63% [+1.54%, +7.88%] | +4.33% | −2% to +3% |
| `linux-v6.12` | `aggregate-summary` | 109.35 → 106.70 ms | −2.06% [−4.47%, +1.82%] | +1.06% [−1.78%, +3.75%] | −0.04% | −2% to +3% |

Split by which arm ran first, the run-order effect is under ±2% on every `default-tree`
cell (`order-split.txt`). The summaries reach +1.61% on the dense tree and +2.10% on the
kernel tree by median, and +2.91% on the dense tree by mean.

## Decision

Not a speed decision.
This is the 0.5.0 release’s measured standing against 0.4.0 on Linux.

- **The default tree is slower by the age column’s price.** On the dense tree it is
  +7.91%, just above the predicted range.
  That is inside exp-217’s interval for the same comparison against `148ef78e`
  [+3.53%, +10.46%], and five and a half points of it are the column on one codebase
  (exp-218). On the directory-sparse kernel tree it is +3.83% and not resolved; its
  mean’s interval excludes zero.
- **The summary is non-inferior at +3% on both trees.** That is −2.07% and −2.06%, so
  0.5.0’s several-roots reader costs a single-root summary nothing measurable against
  0.4.0.

Recorded `rejected` as a speed claim with the candidate kept, as exp-217 recorded the
same comparison.
