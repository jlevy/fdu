---
title: Type-id get-mut on roll-up add on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-111
  title: Type-id get-mut on roll-up add on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H114
  subject:
    tree_label: metabrowser-clone
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 3fbfed48354ed91f6933c70a8e798f21bbe7233d939926a8b768f7857428c541
    tree_provenance: "A clone of github.com/jlevy/metabrowser used as this host's source-checkout subject. The 2026-08 nominated path (fdu/benchmarks/corpus/realtree/metabrowser at 433fb6e plus workspace state) is gone from disk; this live checkout replaces it. The clone is reproducible; workspace state on top of it is not."
    tree_reconstructible: false
    tree_entries: 145931
    tree_directories: 11512
    tree_files: 133597
    tree_symlinks: 822
    tree_apparent_bytes: 1724995969
    tree_allocated_bytes: 2058641408
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
    control: current HEAD with H112 timers at c06d09e7
    candidate: get_mut before entry for ContentRollUp type-id String
    control_binary:
      name: control
      sha256: 1103334a7d28c80d120c94eb2d4cd2a9d2a6c0a90a96804d8f919467694b0a49
      size_bytes: 2503920
      args: []
    candidate_binary:
      name: candidate
      sha256: 5b655ce32de84be32ed75d0f2f899b7748dbfbfbd62bc348f8ef9d093a9b1f83
      size_bytes: 2503920
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-111-rollup-typeid-get-mut.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1328853271.0
          candidate_median: 1290897000.0
          control_p95_over_median: 1.321
          candidate_p95_over_median: 1.034
          change_pct: -0.558
          ci95_low_pct: -17.916
          ci95_high_pct: 4.79
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1003423375.0
          candidate_median: 952039854.5
          control_p95_over_median: 1.26
          candidate_p95_over_median: 1.048
          change_pct: -1.342
          ci95_low_pct: -16.896
          ci95_high_pct: 4.309
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1264705000.0
          candidate_median: 1243477000.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.011
          change_pct: -0.786
          ci95_low_pct: -5.501
          ci95_high_pct: 1.483
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 1123413000.0
          candidate_median: 1107636500.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.011
          change_pct: -0.963
          ci95_low_pct: -4.509
          ci95_high_pct: 0.2
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 141292000.0
          candidate_median: 136200000.0
          control_p95_over_median: 1.167
          candidate_p95_over_median: 1.078
          change_pct: -1.816
          ci95_low_pct: -14.232
          ci95_high_pct: 12.599
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 59694271.0
          candidate_median: 40177187.5
          control_p95_over_median: 7.253
          candidate_p95_over_median: 2.753
          change_pct: 15.901
          ci95_low_pct: -77.829
          ci95_high_pct: 97.778
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 405258240.0
          candidate_median: 407683072.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.002
          change_pct: 0.642
          ci95_low_pct: 0.295
          ci95_high_pct: 0.795
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
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
          involuntary_context_switches: inconclusive
          major_faults: inconclusive
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 16
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: get_mut before entry on by_type; no unsafe; reverted after reject
  verdict:
    decision: rejected
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -0.558
    reason: wall -0.56 percent but interval includes zero; type-id alloc trim reverted
    commit: null
---
## What was predicted

H112 scoped the remaining sidecar restore cost to apply / commit / `merge_ancestors`.
H113 rejected the leftover completeness-count walk on wall.
Parse and candidate install stay closed.

H114 is the smallest apply/install increment that is not an instruction-only trim and
not the EntryId composite: `ContentRollUp::add` allocates a type-id `String` on every
ancestor merge through `entry(to_string())`, even when that type is already in the map.
`get_mut` before `entry` is the same pattern `merge_ancestors` already uses for
`PathBuf`. exp-108 counted 1,121,963 roll-up merges on this subject.

Named before measuring:

- Metric: `content-cache-hit` wall on deciding-scale `metabrowser-clone`.
- Direction: down.
- Accept: median at least 3% faster and the 95% paired interval entirely below zero;
  content digest identical.
- Control: this branch HEAD at `c06d09e7`, including the H112 off-by-default restore
  phase timers. Claim-grade pair with `FDU_COUNTERS` unset.

H103 is the precedent if the mechanism is real and wall does not move: revert.

## What was measured

Subject: nominated `metabrowser-clone` (live checkout, 145,931 entries / 133,597 files /
11,512 directories, max depth 19). Same shape and engine digest as exp-109 / exp-110
(`3fbfed48…`). The tree did not mutate during the pair.

Job: harness `content-cache-hit` after one `content-seed` per variant into an isolated
scratch snapshot. 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 49.5% CPU busy.
The pair ran as **uncontrolled**. Initial busy 49.25%; final 25.78%. The 25% bar was not
lowered. No RAM disk.

Control is the release probe at `c06d09e7`, sha256 `1103334a…`. Candidate is that probe
plus `get_mut` before `entry` on the type-id map, sha256 `5b655ce3…`. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,328.9 ms | 1,003.4 ms | 386.5 MiB |
| candidate | 1,290.9 ms | 952.0 ms | 388.8 MiB |

Every timed sample was `source=content-cache` with 133,597 cache hits and 0 applied.
Content digest `3b8cfa7183353657e70de585bc1262bbbf29161a9c9422a4a390526e94346ef1`, the
same digest exp-108, exp-109, and exp-110 recorded.

Pair 10 spiked (control 2,242 ms, candidate 1,214 ms).
That is host noise on an uncontrolled cell, and it widens the interval.

## What the accept rule said

Wall −0.56% [−17.92%, +4.79%]. REJECT. The median is below 3% and the interval includes
zero.

Component −1.34% [−16.90%, +4.31%] and user CPU −0.96% [−4.51%, +0.20%] also include
zero. There is no leftover mechanism signal to promote.

## Judgment

H114 is rejected on the wall rule.
Removing the per-merge type-id `String` alloc is not the remaining restore win.
The engine change is reverted.

H83 stays open, scoped to rebuilding roll-ups per file times depth, not to this alloc
trim. That named mechanism is H115 / `fdu-jxhk`: one bottom-up pass after restore
inserts. Do not retry another alloc-trim on this map.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
