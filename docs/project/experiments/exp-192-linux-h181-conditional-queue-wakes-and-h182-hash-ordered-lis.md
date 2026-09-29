---
title: Linux H181 conditional queue wakes and H182 hash-ordered listings do not move wall time
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-192
  title: Linux H181 conditional queue wakes and H182 hash-ordered listings do not move wall time
  date: "2026-09-29"
  hypotheses:
    - H181
    - H182
  subject:
    tree_label: linux-v6.12
    tree_root_id: 4ffe9d749fe638cd6c746668f7ae66d88e50225fa6903eabd8c4b9405b1aa245
    tree_engine_digest: 55a09ea65f48d38d88d7e2eacde95fa81d0a9970baabfeeba6f9e9418f478704
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293209
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v49
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: "20933081 probe: H169 head"
    candidate: "7233dad6 probe: H181 + H182 bundle"
    control_binary:
      name: control
      sha256: 1b69d391a91fdb3296d51c97a396d2365731941ab631ef333f03ff0a016882b8
      size_bytes: 3861144
      args: []
    candidate_binary:
      name: bundle
      sha256: e17d6b70241e45ffb2408cf3b4fd73840ba73d1df7b8847281defc65c0de007d
      size_bytes: 3894464
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-192/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 76122974.5
          candidate_median: 75897944.0
          control_p95_over_median: 1.152
          candidate_p95_over_median: 1.094
          change_pct: 1.409
          ci95_low_pct: -6.357
          ci95_high_pct: 5.874
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 72786855.5
          candidate_median: 72616932.0
          control_p95_over_median: 1.154
          candidate_p95_over_median: 1.095
          change_pct: 2.389
          ci95_low_pct: -5.73
          ci95_high_pct: 5.573
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 273342000.0
          candidate_median: 265101500.0
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.078
          change_pct: -1.597
          ci95_low_pct: -6.124
          ci95_high_pct: 1.345
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 90257500.0
          candidate_median: 94141000.0
          control_p95_over_median: 1.363
          candidate_p95_over_median: 1.326
          change_pct: 9.734
          ci95_low_pct: -5.65
          ci95_high_pct: 15.899
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 177825500.0
          candidate_median: 174435500.0
          control_p95_over_median: 1.173
          candidate_p95_over_median: 1.112
          change_pct: -6.528
          ci95_low_pct: -12.574
          ci95_high_pct: 5.282
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
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
          control_median: 74117303.5
          candidate_median: 74047053.5
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.172
          change_pct: 0.2
          ci95_low_pct: -2.039
          ci95_high_pct: 1.913
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 71018722.5
          candidate_median: 70697204.0
          control_p95_over_median: 1.077
          candidate_p95_over_median: 1.164
          change_pct: 0.613
          ci95_low_pct: -2.562
          ci95_high_pct: 2.594
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 260473500.0
          candidate_median: 259314500.0
          control_p95_over_median: 1.06
          candidate_p95_over_median: 1.098
          change_pct: 1.203
          ci95_low_pct: -3.225
          ci95_high_pct: 4.155
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 78277500.0
          candidate_median: 85780000.0
          control_p95_over_median: 1.337
          candidate_p95_over_median: 1.182
          change_pct: 2.963
          ci95_low_pct: -10.579
          ci95_high_pct: 13.161
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 178985000.0
          candidate_median: 179919500.0
          control_p95_over_median: 1.132
          candidate_p95_over_median: 1.152
          change_pct: -3.422
          ci95_low_pct: -5.8
          ci95_high_pct: 1.017
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
    lines_changed: 700
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 0.2
    reason: "quiet 20-pair bundle cell: default-tree +0.20% [-2.04%, +1.91%] on linux-v6.12 and -1.40% [-7.42%, +3.42%] on node-modules-dense; queue futex wakes 1,426 -> 3-10 and consumer instructions -3% are real but below the wall bar"
    commit: d1c667f6
    kept: control
---
## What was predicted

Two small consumer and queue trims from the Fable mid-night sweep, measured as one
bundle because each alone was predicted below the 3% bar:
- **H181:** `DirectoryQueue::extend` called `notify_all` unconditionally, and std’s
  futex condvar issues a `futex(WAKE)` syscall whether or not a walker waits.
  A waiter count under the queue’s lock makes the wake conditional.
- **H182:** under the H172 tier, each listing is ordered by `(fnv32(name), position)`
  instead of by name with byte comparisons, with name order restored for the survivors
  that need it.

Pre-registered in the registry (`f05d1405`) before any timed sample:
- **Candidate and control:** the bundle, merged on the H169 head, against the H169 head.
- **Deciding:** `default-tree` wall −3% with the interval below zero on `linux-v6.12`
  and `node-modules-dense`, 20 pairs.
- **Secondary:** `aggregate-summary`.
- **Prediction:** −0.5 to −1 ms (H181) plus −1.5 to −2 ms (H182) on the kernel tree.

## What was measured

Implementation: `8ac2c61d` (H181), `d1c667f6` (H182), merged on the H169 head as
`7233dad6`. The answer check was identical in 171 comparisons, and an Opus review passed
both changes: a model of the queue found no lost wakeup in 116,000 interleavings.
- **H181:** `strace -c` of `perf_probe default-tree --threads 4` on `node-modules-dense`
  shows the queue’s condvar wakes falling from 1,426 to 3–10 per run, and total `futex`
  calls about 14% lower.
- **H182:** callgrind shows about 13M fewer main-thread instructions on `linux-v6.12`,
  about 3%.

Quiet, 20 pairs, no invalid samples (the `linux-v6.12` cell was refused once at the
start gate and rerun whole).
Control: `20933081` probe (H169 head).

| Subject, job | Control | Bundle | Change |
| --- | ---: | ---: | --- |
| `linux-v6.12`, `default-tree` | 74.1 ms | 74.0 ms | +0.20% [−2.04%, +1.91%] |
| `linux-v6.12`, `aggregate-summary` | 76.1 ms | 75.9 ms | +1.41% [−6.36%, +5.87%] |
| `node-modules-dense`, `default-tree` | 81.1 ms | 79.7 ms | −1.40% [−7.42%, +3.42%] |
| `node-modules-dense`, `aggregate-summary` | 70.2 ms | 69.4 ms | −0.40% [−5.57%, +0.79%] |

The `node-modules-dense` run is kept beside this record’s evidence.

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-node-modules-dense.json.gz`, the `node-modules-dense` leg.

## Decision

Rejected; neither change is merged.
Both mechanisms are real: the wake syscalls fall two orders of magnitude and the
consumer loses about 3% of its instructions.
Neither moves wall time on four vCPUs, where the consumer is no longer on the critical
path and walkers seldom park.
This is the same lesson as H158 for consumer wakes.
The branch `perf/h181-h182` was kept locally and not pushed.
