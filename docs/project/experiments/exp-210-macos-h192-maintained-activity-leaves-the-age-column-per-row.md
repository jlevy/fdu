---
title: "macOS: H192 maintained activity leaves the age column per-row work, 7 and 18 microseconds a report"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-210
  title: "macOS: H192 maintained activity leaves the age column per-row work, 7 and 18 microseconds a report"
  date: "2026-10-09"
  hypotheses:
    - H192
  subject:
    tree_label: rustup
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: 4304d9d4071fd4478a0510edf80c4b78594f98b84969d90a1d7230eb0dc94d78
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains (root_id 36ce9b22). Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 77355
    tree_directories: 3427
    tree_files: 73928
    tree_symlinks: 0
    tree_apparent_bytes: 3750189949
    tree_allocated_bytes: 3978313728
    tree_max_depth: 17
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
    control: "148ef78e probe: main before the age column"
    candidate: "probe built from the working tree that became 10ae731f, 44 minutes before that commit existed, so only the binary's sha256 ties it to source (review C4 on #191); it predates 1b3ac793 (the folded tree's restored stats) and e25f12e3 (the tree age cell). The age column, each directory keeping its newest activity beside its roll-up"
    control_binary:
      name: control
      sha256: d2ac70ff129f6c510100a0f58a27677015fec20af2f8d29b732f6e29e2182041
      size_bytes: 3363840
      args: []
    candidate_binary:
      name: candidate
      sha256: b6f433efcc9a5d89a4159adb6ad1bea3b95c5318be77098e00f375ecb6f463f7
      size_bytes: 3380352
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-210/run.json.gz
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 296164375.0
          candidate_median: 289845000.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.053
          change_pct: -0.642
          ci95_low_pct: -4.986
          ci95_high_pct: 2.865
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 123349979.0
          candidate_median: 122065354.5
          control_p95_over_median: 1.121
          candidate_p95_over_median: 1.061
          change_pct: 0.405
          ci95_low_pct: -10.564
          ci95_high_pct: 8.771
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 815156000.0
          candidate_median: 793189000.0
          control_p95_over_median: 1.053
          candidate_p95_over_median: 1.115
          change_pct: 0.342
          ci95_low_pct: -5.465
          ci95_high_pct: 4.863
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 215090000.0
          candidate_median: 213169000.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.042
          change_pct: -0.57
          ci95_low_pct: -3.145
          ci95_high_pct: 0.421
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 603322500.0
          candidate_median: 580091000.0
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.165
          change_pct: 0.307
          ci95_low_pct: -7.521
          ci95_high_pct: 8.066
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 27303936.0
          candidate_median: 27828224.0
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.031
          change_pct: 2.911
          ci95_low_pct: -1.82
          ci95_high_pct: 4.393
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
          - "voluntary_context_switches straddles its +50% regression limit"
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
          involuntary_context_switches: within-limit
          major_faults: inconclusive
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 128018521.0
          candidate_median: 128065875.5
          control_p95_over_median: 1.461
          candidate_p95_over_median: 1.443
          change_pct: 3.368
          ci95_low_pct: -11.8
          ci95_high_pct: 13.102
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 122516229.0
          candidate_median: 123525458.5
          control_p95_over_median: 1.427
          candidate_p95_over_median: 1.394
          change_pct: 4.743
          ci95_low_pct: -11.967
          ci95_high_pct: 13.613
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 594271000.0
          candidate_median: 591587000.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.063
          change_pct: -1.312
          ci95_low_pct: -5.743
          ci95_high_pct: 8.368
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 40236000.0
          candidate_median: 39924000.0
          control_p95_over_median: 1.06
          candidate_p95_over_median: 1.087
          change_pct: -1.183
          ci95_low_pct: -7.162
          ci95_high_pct: 11.028
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 554034500.0
          candidate_median: 553221500.0
          control_p95_over_median: 1.037
          candidate_p95_over_median: 1.058
          change_pct: -1.884
          ci95_low_pct: -5.862
          ci95_high_pct: 8.09
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 12722176.0
          candidate_median: 12525568.0
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.061
          change_pct: 0.784
          ci95_low_pct: -4.313
          ci95_high_pct: 5.751
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
          - "peak_rss_bytes straddles its +5% regression limit"
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
    - job: index-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 320021708.5
          candidate_median: 335331437.5
          control_p95_over_median: 1.133
          candidate_p95_over_median: 1.076
          change_pct: 5.686
          ci95_low_pct: -6.417
          ci95_high_pct: 14.153
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 129166.5
          candidate_median: 134062.5
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.026
          change_pct: 5.788
          ci95_low_pct: 2.661
          ci95_high_pct: 7.52
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 960181000.0
          candidate_median: 982057000.0
          control_p95_over_median: 1.197
          candidate_p95_over_median: 1.07
          change_pct: 2.045
          ci95_low_pct: -15.046
          ci95_high_pct: 7.301
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 217673000.0
          candidate_median: 217375500.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.036
          change_pct: -0.977
          ci95_low_pct: -1.665
          ci95_high_pct: 3.006
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 749391500.0
          candidate_median: 762396000.0
          control_p95_over_median: 1.246
          candidate_p95_over_median: 1.088
          change_pct: 2.569
          ci95_low_pct: -18.051
          ci95_high_pct: 10.097
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 27967488.0
          candidate_median: 28139520.0
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.039
          change_pct: 1.244
          ci95_low_pct: -2.414
          ci95_high_pct: 4.859
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
    - job: opened-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1097003041.5
          candidate_median: 1113784208.5
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.071
          change_pct: 1.557
          ci95_low_pct: 1.059
          ci95_high_pct: 3.745
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 135020.5
          candidate_median: 152916.5
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.024
          change_pct: 13.701
          ci95_low_pct: 11.68
          ci95_high_pct: 14.688
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1151145000.0
          candidate_median: 1172395000.0
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.043
          change_pct: 1.578
          ci95_low_pct: 0.904
          ci95_high_pct: 3.762
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 823827500.0
          candidate_median: 835037500.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.035
          change_pct: 1.171
          ci95_low_pct: 0.16
          ci95_high_pct: 3.175
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 325375500.0
          candidate_median: 337357500.0
          control_p95_over_median: 1.1
          candidate_p95_over_median: 1.065
          change_pct: 3.349
          ci95_low_pct: 1.699
          ci95_high_pct: 5.988
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 115933184.0
          candidate_median: 114933760.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.004
          change_pct: -0.856
          ci95_low_pct: -1.227
          ci95_high_pct: -0.361
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
          control_median: 234549021.0
          candidate_median: 232622666.5
          control_p95_over_median: 1.048
          candidate_p95_over_median: 1.034
          change_pct: -0.244
          ci95_low_pct: -3.214
          ci95_high_pct: 0.818
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 62945479.5
          candidate_median: 64169271.0
          control_p95_over_median: 1.112
          candidate_p95_over_median: 1.008
          change_pct: 2.205
          ci95_low_pct: 0.531
          ci95_high_pct: 2.708
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 232585000.0
          candidate_median: 229767000.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.035
          change_pct: -0.517
          ci95_low_pct: -2.267
          ci95_high_pct: 1.092
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 226456500.0
          candidate_median: 223570000.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.033
          change_pct: -0.796
          ci95_low_pct: -1.794
          ci95_high_pct: 0.559
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 5924500.0
          candidate_median: 5851500.0
          control_p95_over_median: 1.285
          candidate_p95_over_median: 1.174
          change_pct: 0.57
          ci95_low_pct: -18.665
          ci95_high_pct: 17.612
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 2579458.5
          candidate_median: 2496041.5
          control_p95_over_median: 3.401
          candidate_p95_over_median: 1.243
          change_pct: -10.983
          ci95_low_pct: -50.377
          ci95_high_pct: 40.496
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 31711232.0
          candidate_median: 31563776.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.003
          change_pct: -0.335
          ci95_low_pct: -0.516
          ci95_high_pct: -0.052
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
  reference_tools:
    - name: dust
      wall_ns_median: 184929104.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 208
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "10ae731f, the maintained maximum replacing the pass on complete indexes"
  verdict:
    decision: rejected
    primary_job: index-second-report
    primary_metric: component_ns
    change_pct: 5.788
    reason: "not a speed decision: the cost record of the age column, a correctness feature that ships regardless, held to the per-row bar review A on #191 set after this run: second tree report over a retained Index +5.79% [+2.66%, +7.52%], 7.5 us, and over an opened root +13.70% [+11.68%, +14.69%], 18.5 us, against the 0.66 and 4.7 ms the pass added (exp-209); cold-scan-index wall -0.64% [-4.99%, +2.87%] and warm-snapshot-load wall -0.24% non-inferior, snapshot load component +2.21% [+0.53%, +2.71%]; opened-second-report wall +1.56% [+1.06%, +3.75%], not replicated in exp-211; default-tree wall +3.37% [-11.80%, +13.10%] not resolved on the loaded host; Linux unmeasured (fdu-088k, H193)"
    commit: 10ae731f
    kept: candidate
---
## What was predicted

H192, the age column’s second design: each directory keeps the newest modification time
of any kind beneath it beside `newest_mtime_ns`, absorbing an addition or a later time
in O(depth) and rebuilt after a removal or an earlier time by the stale-maximum repair
`recompute_newest_upward` already runs.
A tree over a complete index then reads only the rows it shows, so a retained report was
expected to return to the control’s time apart from the column’s own per-row work, with
the one-shot default report, a cold walk into an index, and a snapshot load unchanged.

The plan’s goal as first written, no measurable change in the retained regime, was not
the bar this result could meet, and review A on #191 asked for one it states.
The bar it is judged against was set there, after this run: a retained report’s cost may
grow with the rows it shows, never with the entries beneath them, and a cold walk and a
snapshot load stay within the +3% non-inferiority margin.

## What was measured

One interleaved probe run on the same subject and host as
[exp-209](exp-209-macos-h191-a-per-report-activity-pass-makes-a-retained-tree-.md):
control the pre-age `main` (`148ef78e`), candidate the maintained maximum (`10ae731f`).
3 warmups and 12 timed trials per variant, exploratory stage; the 1-minute load average
was 17 at the start and 29 at the end over 10 cores, the CPU 26% to 48% busy.
No sample was invalid and the tree was unchanged.

- `index-second-report` component: 0.129 ms to 0.134 ms, +5.79% [+2.66%, +7.52%], a
  median 7.5 µs. Primary.
- `opened-second-report` component: 0.135 ms to 0.153 ms, +13.70% [+11.68%, +14.69%],
  18.5 µs. Its wall moved +1.56% [+1.06%, +3.75%], with CPU, user, and system time also
  above zero: that is the open-plus-discovery lifecycle, not the report, and
  [exp-211](exp-211-macos-h192-replicated-over-an-opened-root-23-microseconds-a-.md) did
  not reproduce it (−0.80% [−3.83%, +2.54%]).
- `default-tree` wall +3.37% [−11.80%, +13.10%], not resolved: an interval this wide
  bounds nothing at the +3% margin.
  Peak RSS +0.78% [−4.31%, +5.75%]; exp-209’s +10.4% was never attributed to the pass
  (review C5 on #191), so this says nothing about it.
- `cold-scan-index` wall −0.64% [−4.99%, +2.87%], non-inferior.
- `warm-snapshot-load` wall −0.24% [−3.21%, +0.82%], non-inferior; component +2.21%
  [+0.53%, +2.71%], non-inferior at +3%.

## Decision

Not a speed decision, recorded as exp-196 records the automount fix: `rejected` as a
speed claim, with the candidate kept, because the age column ships regardless of this
measurement (review C3 on #191). As first recorded it was an accept, and the accept rule
cannot produce one here: both primaries regressed with intervals wholly above zero, and
the per-row bar it was judged against was set after this run, which the loop never
allows as an accept.
The run was also uncontrolled, and every job’s fail-closed qualification is
`inconclusive`, which supports exploration only.

What it does record is the column’s price against that bar: 7.5 µs and 18.5 µs a report,
against the 0.66 ms and 4.7 ms the pass added, with a cold walk and a snapshot load
non-inferior at +3% on wall.
The one-shot default report’s change was not resolved on this host.
Every figure is from one loaded macOS host.
The measured binary predates two later changes on measured paths, `1b3ac793` (the folded
tree stats directories and symlinks again) and `e25f12e3` (the tree’s age cell);
[exp-212](exp-212-macos-the-age-column-re-measured-at-the-shipped-head-per-row.md)
re-measures the shipped head with each binary tied to its commit.
Linux is unmeasured, and there the folded default tree also reads each directory’s and
symlink’s time again, giving back H185; `fdu-088k` measures it as the pre-registered
H193. [exp-211](exp-211-macos-h192-replicated-over-an-opened-root-23-microseconds-a-.md)
replicates the opened root’s figure.
