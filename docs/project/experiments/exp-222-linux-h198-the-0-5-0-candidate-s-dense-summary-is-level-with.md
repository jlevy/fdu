---
title: "Linux: H198, the 0.5.0 candidate's dense summary is level with the age column's base, inconclusive at +3%"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-222
  title: "Linux: H198, the 0.5.0 candidate's dense summary is level with the age column's base, inconclusive at +3%"
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
    control: "23d11b19 probe (the age column's base)"
    candidate: "1c42c91f probe (the 0.5.0 candidate, #202's head)"
    control_binary:
      name: age
      sha256: 683bf0cb610780c29b0498435dd014a484a01e6bf8add20d03b48c7f20880a94
      size_bytes: 3977416
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
    run_artifact: docs/project/experiments/evidence/exp-222/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 97180556.0
          candidate_median: 98050188.0
          control_p95_over_median: 1.175
          candidate_p95_over_median: 1.207
          change_pct: -0.216
          ci95_low_pct: -3.501
          ci95_high_pct: 3.887
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        component_ns:
          control_median: 93614139.0
          candidate_median: 94178742.0
          control_p95_over_median: 1.182
          candidate_p95_over_median: 1.213
          change_pct: 0.313
          ci95_low_pct: -3.176
          ci95_high_pct: 3.624
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        cpu_ns:
          control_median: 345802000.0
          candidate_median: 341608500.0
          control_p95_over_median: 1.152
          candidate_p95_over_median: 1.234
          change_pct: -0.367
          ci95_low_pct: -2.445
          ci95_high_pct: 3.416
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        user_cpu_ns:
          control_median: 88657000.0
          candidate_median: 86367500.0
          control_p95_over_median: 1.294
          candidate_p95_over_median: 1.487
          change_pct: -2.228
          ci95_low_pct: -7.47
          ci95_high_pct: 3.762
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        system_cpu_ns:
          control_median: 255767000.0
          candidate_median: 257486500.0
          control_p95_over_median: 1.174
          candidate_p95_over_median: 1.26
          change_pct: 3.206
          ci95_low_pct: -1.49
          ci95_high_pct: 6.286
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
          control_median: 105627696.0
          candidate_median: 105621031.5
          control_p95_over_median: 1.218
          candidate_p95_over_median: 1.179
          change_pct: -0.364
          ci95_low_pct: -4.313
          ci95_high_pct: 4.439
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        component_ns:
          control_median: 100974660.0
          candidate_median: 101087478.0
          control_p95_over_median: 1.223
          candidate_p95_over_median: 1.166
          change_pct: 0.363
          ci95_low_pct: -4.385
          ci95_high_pct: 4.719
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        cpu_ns:
          control_median: 361834500.0
          candidate_median: 358723500.0
          control_p95_over_median: 1.183
          candidate_p95_over_median: 1.182
          change_pct: -0.116
          ci95_low_pct: -3.994
          ci95_high_pct: 3.688
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        user_cpu_ns:
          control_median: 103580000.0
          candidate_median: 101570500.0
          control_p95_over_median: 1.351
          candidate_p95_over_median: 1.411
          change_pct: -2.388
          ci95_low_pct: -10.188
          ci95_high_pct: 4.912
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        system_cpu_ns:
          control_median: 253373500.0
          candidate_median: 254546000.0
          control_p95_over_median: 1.225
          candidate_p95_over_median: 1.187
          change_pct: 0.107
          ci95_low_pct: -4.042
          ci95_high_pct: 2.581
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
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
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -0.216
    reason: "not a speed decision: the release gate's reading of fdu-4arl; two-arm, 100 pairs, quiet: aggregate-summary on node-modules-dense -0.22% [-3.50%, +3.89%] against +2% to +6% predicted, inconclusive under H195's registered rule (the interval crosses +3%), so the tag proceeds and fdu-4arl stays open; mean +1.70% [-1.49%, +4.93%], Hodges-Lehmann +1.70%, order effect -0.21%; exp-220's +5.28% does not reproduce; (C) the several-roots head 0d6011da against the candidate -3.77% [-7.44%, +0.91%] with a +3.59% order effect"
    commit: 1c42c91f
    kept: candidate
---
## What was predicted

H198 (B) and (C) were registered before any timed sample: `871bc6bd` reached GitHub at
17:59:22Z, and the first sample ran at 18:10:29Z. The prediction comes from exp-220,
which measured one root’s `aggregate-summary` on `node-modules-dense` at the
several-roots head (`0d6011da`) +5.28% [+2.27%, +7.62%] slower than at the age column’s
base (`23d11b19`). That was 40 pairs, unattributed, and inconclusive at H195’s +3%
margin (`fdu-4arl`). The [#202](https://github.com/jlevy/fdu/pull/202) handoff required
the issue resolved before tagging, with a larger fixed sample.

- **(B), the deciding cell:** the age column’s base against the release candidate, at
  100 pairs. Predicted +2% to +6%. H195’s rule decides it on the paired median:
  - upper bound at most +3%: non-inferior, and `fdu-4arl` closes as not reproduced;
  - lower bound above +3%: inferior, and the tag waits for the maintainer;
  - an interval crossing +3%: inconclusive, which that rule does not count as a
    regression. The tag proceeds and `fdu-4arl` stays open.

  The secondary estimators and the run-order split are reported but do not decide.

- **(C):** runs only if (B) is not non-inferior.
  It is the same cell with the several-roots head against the candidate, to place any
  change before or after #192.

## What was measured

**Host.** A 4-vCPU Firecracker guest (Intel Xeon at 2.8 GHz, Linux 6.18.44-fc-v114,
ext4), as root, warm page cache.
This is a different processor from the 2.1 GHz guest of exp-217 to exp-221, so absolute
times are not comparable across those sessions.

**Regime.** Quiet:
- each cell started only after one-second CPU busy had stayed under 15% and load per
  core at most 0.25 for ten seconds;
- the harness held CPU busy under 25% around every sample;
- 3 warmups and 100 pairs, fixed in advance;
- no invalid sample, and the tree was verified unchanged across each run.

**The subject.** `node-modules-dense` was rebuilt on 2026-10-10 and still matches that
day’s fingerprint.

**Binaries.** All built before any timing, with nothing else running during the cells:

| Arm | Commit | Binary |
| --- | --- | --- |
| The age column’s base | `23d11b19` | the probe exp-217 to exp-220 timed, reused (its hash matches theirs) |
| The several-roots head | `0d6011da` | the same, reused |
| The candidate | `1c42c91f`, #202’s head: `e6b8850b` plus `README.md` | built for this record |

| Cell | Job | Control → candidate (median wall) | Paired median [95%] | Mean [95% bootstrap] | Hodges–Lehmann |
| --- | --- | --- | --- | --- | --- |
| (B) base → candidate | **`aggregate-summary`** | 97.18 → 98.05 ms | **−0.22% [−3.50%, +3.89%]** | +1.70% [−1.49%, +4.93%] | +1.70% |
| (B) | `default-tree` | 105.63 → 105.62 ms | −0.36% [−4.31%, +4.44%] | +0.38% [−2.77%, +3.46%] | +0.23% |
| (C) several-roots head → candidate | `aggregate-summary` | 99.64 → 95.85 ms | −3.77% [−7.44%, +0.91%] | −1.92% [−4.60%, +0.77%] | −2.63% |
| (C) | `default-tree` | 105.10 → 105.18 ms | −2.34% [−4.19%, +2.07%] | −0.11% [−2.38%, +2.17%] | −0.60% |

In (B), 49 of 100 summary pairs have the candidate slower.
Splitting the pairs by which arm ran first gives an arm effect of −0.01% and an order
effect of −0.21% on the summary (`order-split.txt`).

In (C) the order effect is large: the summary’s pairs give +0.22% where the
several-roots head ran first and −6.96% where the candidate did.
That is an arm effect of −3.37% and an order effect of +3.59%. So (C) places the
candidate at or below the several-roots head, but does not size the difference.

## Decision

Not a speed decision.
This is the release gate’s reading of `fdu-4arl`.

- **(B) is inconclusive under the registered rule.** The paired median is −0.22%, but
  the interval reaches +3.89%, past the +3% margin.
  The secondary estimators classify the same way: the bootstrap mean’s interval also
  crosses +3%.
- **exp-220’s +5.28% does not reproduce at the candidate.** Every point estimate sits
  between −0.22% and +1.70%.
- **(C) finds nothing slower in the candidate than at the several-roots head.** Whatever
  exp-220 measured at `0d6011da`, if it was real, is not in the candidate.

By the rule registered before the cells, the tag proceeds.
`fdu-4arl` stays open for attribution on a host with hardware counters, with this
reading recorded. Recorded `rejected` as a speed claim with the candidate kept, as
exp-220 was.
