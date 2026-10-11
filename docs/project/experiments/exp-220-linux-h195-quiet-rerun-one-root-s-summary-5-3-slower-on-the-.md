---
title: "Linux: H195 quiet rerun, one root's summary 5.3% slower on the dense tree, render-yaml faster"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-220
  title: "Linux: H195 quiet rerun, one root's summary 5.3% slower on the dense tree, render-yaml faster"
  date: "2026-10-10"
  hypotheses:
    - H195
  subject:
    tree_label: node-modules-dense
    tree_root_id: ffbd3cb6be62b77f7e2e9b5964d1e380e8a3f3b7bb4fda55f462ae4b189841b5
    tree_engine_digest: 62125c60d0dddf9fdb9f89b8c2542c8afd573f3460ccc6f052626099669b7751
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.0, npm 10.9.4); rebuilt 2026-10-10 on this host: 79,957 entries, 9,439 directories, 107 symlinks, no .gitignore, the counts exp-197 recorded."
    tree_reconstructible: true
    tree_entries: 79957
    tree_directories: 9439
    tree_files: 70411
    tree_symlinks: 107
    tree_apparent_bytes: 538992241
    tree_allocated_bytes: 757870592
    tree_max_depth: 12
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
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
    trials: 40
    warmups: 3
    interleaved: true
    control: "23d11b19 probe (#192's base)"
    candidate: "0d6011da probe (#192's head)"
    control_binary:
      name: age
      sha256: 683bf0cb610780c29b0498435dd014a484a01e6bf8add20d03b48c7f20880a94
      size_bytes: 3977416
      args: []
    candidate_binary:
      name: roots
      sha256: d93d5a4361b6717accb93f52eed531936ca1d1f513555484d2f4c4658086c7a0
      size_bytes: 4266000
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-220/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 90835330.5
          candidate_median: 90219556.5
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.192
          change_pct: 5.276
          ci95_low_pct: 2.271
          ci95_high_pct: 7.623
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 40
        component_ns:
          control_median: 87852874.5
          candidate_median: 87614211.5
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.187
          change_pct: 5.321
          ci95_low_pct: 2.198
          ci95_high_pct: 7.727
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 40
        cpu_ns:
          control_median: 327109500.0
          candidate_median: 328934000.0
          control_p95_over_median: 1.095
          candidate_p95_over_median: 1.186
          change_pct: 2.653
          ci95_low_pct: 0.307
          ci95_high_pct: 5.445
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 40
        user_cpu_ns:
          control_median: 47322000.0
          candidate_median: 50455500.0
          control_p95_over_median: 1.374
          candidate_p95_over_median: 1.414
          change_pct: 2.28
          ci95_low_pct: -12.634
          ci95_high_pct: 22.865
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 40
        system_cpu_ns:
          control_median: 279830500.0
          candidate_median: 280279000.0
          control_p95_over_median: 1.134
          candidate_p95_over_median: 1.242
          change_pct: 2.681
          ci95_low_pct: -0.24
          ci95_high_pct: 6.514
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 40
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
    - job: opened-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1677346136.0
          candidate_median: 1674913445.5
          control_p95_over_median: 1.092
          candidate_p95_over_median: 1.111
          change_pct: 0.159
          ci95_low_pct: -1.627
          ci95_high_pct: 3.713
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 40
        component_ns:
          control_median: 1064144.5
          candidate_median: 964212.0
          control_p95_over_median: 1.333
          candidate_p95_over_median: 1.528
          change_pct: -4.609
          ci95_low_pct: -15.334
          ci95_high_pct: 7.293
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 40
        cpu_ns:
          control_median: 1790134500.0
          candidate_median: 1789295000.0
          control_p95_over_median: 1.088
          candidate_p95_over_median: 1.098
          change_pct: 0.061
          ci95_low_pct: -1.786
          ci95_high_pct: 3.703
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 40
        user_cpu_ns:
          control_median: 1243280000.0
          candidate_median: 1236578000.0
          control_p95_over_median: 1.103
          candidate_p95_over_median: 1.096
          change_pct: -0.04
          ci95_low_pct: -2.79
          ci95_high_pct: 3.647
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 40
        system_cpu_ns:
          control_median: 550910500.0
          candidate_median: 551386500.0
          control_p95_over_median: 1.138
          candidate_p95_over_median: 1.153
          change_pct: -2.055
          ci95_low_pct: -5.629
          ci95_high_pct: 2.92
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 40
        peak_rss_bytes:
          control_median: 118626304.0
          candidate_median: 118827008.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.001
          change_pct: 0.131
          ci95_low_pct: 0.091
          ci95_high_pct: 0.192
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 40
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
    - job: render-yaml
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 850409294.0
          candidate_median: 805430363.5
          control_p95_over_median: 1.093
          candidate_p95_over_median: 1.096
          change_pct: -5.478
          ci95_low_pct: -7.779
          ci95_high_pct: -2.34
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 40
        component_ns:
          control_median: 180148935.0
          candidate_median: 163681812.0
          control_p95_over_median: 1.159
          candidate_p95_over_median: 1.323
          change_pct: -9.648
          ci95_low_pct: -12.092
          ci95_high_pct: -7.918
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 40
        cpu_ns:
          control_median: 1103573500.0
          candidate_median: 1060612500.0
          control_p95_over_median: 1.075
          candidate_p95_over_median: 1.101
          change_pct: -4.848
          ci95_low_pct: -7.553
          ci95_high_pct: -0.13
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 40
        user_cpu_ns:
          control_median: 778541000.0
          candidate_median: 731798000.0
          control_p95_over_median: 1.066
          candidate_p95_over_median: 1.108
          change_pct: -4.915
          ci95_low_pct: -7.951
          ci95_high_pct: -1.881
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 40
        system_cpu_ns:
          control_median: 339233000.0
          candidate_median: 333541000.0
          control_p95_over_median: 1.102
          candidate_p95_over_median: 1.105
          change_pct: -2.573
          ci95_low_pct: -4.681
          ci95_high_pct: -0.59
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 40
        peak_rss_bytes:
          control_median: 100632576.0
          candidate_median: 100802560.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.007
          change_pct: 0.179
          ci95_low_pct: -0.02
          ci95_high_pct: 0.501
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 40
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
    change_pct: 5.276
    reason: "not a speed decision: several roots ship regardless; two-arm, 40 pairs, quiet: aggregate-summary on node-modules-dense +5.28% [+2.27%, +7.62%], slower with the interval above zero, but non-inferiority at +3% is neither shown nor refuted, so inconclusive under H195's registered rule, and the figure depends on the estimator (mean +3.53%, Hodges-Lehmann +3.97%, marginal medians -0.68%); exp-217's three-arm b1 cell, both arms outer, puts the shipped head's summary +5.45% [+1.99%, +7.20%] above v0.4.0's; linux-v6.12 +0.92% [-2.75%, +5.38%]; instructions +0.3% and system calls equal, unattributed; opened-second-report +0.16% and +0.72% with upper bounds past +3%, inconclusive; render-yaml -5.48% and -6.35%, faster"
    commit: 0d6011da
    kept: candidate
---
## What was predicted

H195, registered before exp-214: the several-roots refactor on
[#192](https://github.com/jlevy/fdu/pull/192) adds no per-entry work for one root, so
wall is non-inferior at +3% on every job it reaches.
exp-214, on a loaded macOS host, held it on five jobs and could not resolve
`aggregate-summary`, `opened-second-report`, or `render-yaml`; those were carried to the
quiet Linux session of `fdu-088k`. The Linux rerun was registered (`843bf2ab`) before it
ran, as two-arm cells of 40 pairs on both real trees, because the session’s three-arm
cells had put H195’s control in the middle slot of every ordinal.

## What was measured

Two two-arm cells, quiet regime, 3 warmups and 40 pairs fixed in advance, exploratory
stage; no sample invalid, no tree changed.
The control is the probe built from `23d11b19`, #192’s base (sha256 `683bf0cb`); the
candidate the probe built from `0d6011da`, #192’s head (`d93d5a43`).

| Job | `node-modules-dense` (this record) | `linux-v6.12` |
| --- | --- | --- |
| `aggregate-summary` | **+5.28% [+2.27%, +7.62%]** | +0.92% [−2.75%, +5.38%] |
| `opened-second-report` | +0.16% [−1.63%, +3.71%] | +0.72% [−2.70%, +4.27%] |
| `render-yaml` | **−5.48% [−7.78%, −2.34%]** | **−6.35% [−7.94%, −4.62%]** |

The dense tree’s `aggregate-summary` figure is the one in the session that depends on
how it is read.

- **Run order.** The second run of each pair was about 5% slower (92.4 against 87.8 ms
  by median), and the order alternates.
  Split by which arm ran first, the pairs give +6.71% and +2.93% by median, an arm
  effect of about +4.8% and an order effect of about +1.9%. By mean they give +7.36% and
  −0.31%, an arm effect of about +3.5% and an order effect of about +3.8%: by means the
  run order moves each half as much as the build does.
- **Estimator.** The mean of the 40 per-pair changes is +3.53% [+0.31%, +6.90%] (a
  percentile bootstrap made as the harness makes its median’s), the Hodges–Lehmann
  estimate +3.97%, and the difference of the two arms’ own medians −0.68% (90.84 against
  90.22 ms). 27 of the 40 pairs are positive, and they range from −18.87% to +31.94%. No
  other two-arm cell of two builds in the session spreads its estimators across more
  than the +3% margin; this one spreads across six points.
- **Corroboration.** exp-217’s three-arm cell `run-b1-nmd-retained` (20 pairs) ran the
  same job on this tree with both compared arms in the outer slots: the shipped head’s
  summary is +5.45% [+1.99%, +7.20%] above v0.4.0’s. In that cell `23d11b19`, from the
  middle slot, is +1.53% [−0.64%, +5.01%] against v0.4.0, and `0d6011da` is +3.31%
  [−4.08%, +8.97%] against `23d11b19`, both unresolved.
  On `linux-v6.12` (`run-b2-lnx-retained`) the shipped head against v0.4.0 is −1.47%
  [−5.61%, +6.98%].

Across the session’s two-arm cells of two builds, the order effect is within about ±2.5%
by median split and reaches +3.84% by mean split, on this record’s summary.
`order-split.txt` covers every job of every committed two-arm run of exp-217 to exp-221,
made by `order.py.txt` from the committed artifacts:

- **By median split,** 2 of the 17 cells of two builds pass ±2%: −2.37% on exp-219’s
  `default-tree` on `linux-v6.12` (`run-h3-lnx-roots-h197`) and +2.46% on
  `opened-second-report` on `linux-v6.12` here (`run-g2-lnx-h195`).
- **By mean split,** every cell but this record’s summary is within ±2.2%.
- **The deciding cells** are small: +1.39% (exp-217), −0.34% (exp-218), and +0.50% and
  +1.11% (exp-219).
- **exp-221’s two cells** compare one build in two configurations, about +420% apart.
  They split by 16 and 21 points by median and by about 1.3 by mean, at most a twentieth
  of the effect.

The summary’s extra time has no counted cause: on `node-modules-dense` the candidate
runs 282.9M and 283.4M instructions against the control’s 282.4M and 282.1M (callgrind,
two runs each, +0.3%), and makes the same `statx`, `openat`, and `getdents64` calls as
v0.4.0 (72,076, 9,448, 18,878; exp-217’s `syscalls-probe.txt`, which traced the summary
at v0.4.0 and the shipped head, not at `23d11b19`). This guest exposes no hardware
counters to look further.

`render-yaml` is faster at #192’s head on both trees, render component −9.65% and
−12.07%, with instruction counts equal across the two heads (9.772G against 9.777G over
the whole process); it too is unattributed.

## Decision

Not a speed decision: several roots ship regardless.
H195 holds on `render-yaml`, which is faster, and on `opened-second-report` only to
within its interval, which reaches +3.71% and +4.27%, so that job is inconclusive rather
than non-inferior. On `aggregate-summary` on the directory-dense tree, one root through
the several-roots door is slower at #192’s head, with the paired interval above zero
(+5.28% [+2.27%, +7.62%]). Non-inferiority at +3% is neither shown nor refuted, so the
job is inconclusive under the registered rule, which reads an interval that crosses +3%
on this host as inconclusive rather than a regression.
How much slower depends on the estimator: about +3.5% to +5.3% over the pairs, +3.5% to
+4.8% balanced for run order, and −0.7% between the arms’ own medians.
exp-217’s outer-arm comparison of the shipped head with v0.4.0, +5.45% [+1.99%, +7.20%],
corroborates a slower summary at the shipped head.
The kernel tree is inconclusive, and instruction and system-call counts do not explain
the dense tree’s difference.
Recorded `rejected` as a speed claim with the candidate kept, and the summary’s cost
opened as its own bead (`fdu-4arl`) for a host with hardware counters.
