---
title: H111 Linux floor and RSS gates fail on current engine
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-141
  title: H111 Linux floor and RSS gates fail on current engine
  date: "2026-09-20"
  hypotheses:
    - H111
  subject:
    tree_label: linux-450k
    tree_root_id: b3459e9451517d4c81d92f78310218b6f158f3fc53e0b5748186122d2f255006
    tree_engine_digest: b77ebb296d346faf853a2d9db41fea20d347e52dfafee01a1d23a357330a7907
    tree_provenance: "Generated balanced recipe, 450,001 entries, manifest f93bffc36eab67a0d2d72909f3552c7bc235e073eff11b5e793e1a5a8407c938, semantic digest 0c5230889cbe6ee25ceb6e64560cb012bccd03126565fd8f8d313e7013715e3d. Reconstructible: python -m benchmarks.generate create --recipe balanced --entries 450000. Same semantic digest as exp-103."
    tree_reconstructible: true
    tree_entries: 450001
    tree_directories: 56251
    tree_files: 393750
    tree_symlinks: 0
    tree_apparent_bytes: 358665192
    tree_allocated_bytes: 1344430080
    tree_max_depth: 7
    tree_mutated_during_run: false
    host_cpu: Intel(R) Xeon(R) Processor
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16791945216
    host_system: Linux 6.12.94+
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: HEAD release probe both arms
    candidate: same probe; floor scoreboard is the verdict
    control_binary:
      name: control
      sha256: 8608363254b7e7f47854106f7db2532e372f7367ec515b96690b47d1022c55ed
      size_bytes: 3025544
      args: []
    candidate_binary:
      name: candidate
      sha256: 8608363254b7e7f47854106f7db2532e372f7367ec515b96690b47d1022c55ed
      size_bytes: 3025544
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-141-h111-linux-floor-companion.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 419114022.5
          candidate_median: 429808792.0
          control_p95_over_median: 1.075
          candidate_p95_over_median: 1.063
          change_pct: 1.986
          ci95_low_pct: -0.356
          ci95_high_pct: 2.385
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 415012948.5
          candidate_median: 425597944.0
          control_p95_over_median: 1.075
          candidate_p95_over_median: 1.062
          change_pct: 1.887
          ci95_low_pct: -0.355
          ci95_high_pct: 2.397
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1196932000.0
          candidate_median: 1195340000.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.045
          change_pct: 0.151
          ci95_low_pct: -0.463
          ci95_high_pct: 0.895
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 400796500.0
          candidate_median: 401468500.0
          control_p95_over_median: 1.116
          candidate_p95_over_median: 1.086
          change_pct: -0.355
          ci95_low_pct: -2.751
          ci95_high_pct: 2.473
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 786420000.0
          candidate_median: 793661000.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.067
          change_pct: 0.751
          ci95_low_pct: -2.606
          ci95_high_pct: 4.262
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 321736704.0
          candidate_median: 321736704.0
          control_p95_over_median: 1.0
          candidate_p95_over_median: 1.0
          change_pct: 0.0
          ci95_low_pct: 0.0
          ci95_high_pct: 0.0
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: noninferior
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
    notes: No engine change. Floor scoreboard is the verdict; this run JSON is a same-binary default-tree companion so perf-record can lift a measured pair.
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 1.986
    reason: "H111 floor/RSS gates fail: 450k index 1.78x parfloor vs 1.4x; RSS 5.20x arena_spike vs 3x; aggregate on nominated reals 1.59x and 1.86x vs 1.25x"
    commit: null
    kept: neither
---
## What was predicted

H111 is the pre-registered Linux floor and RSS stage of H86, not a rewrite.

Named before measuring:

- Subject: generated `linux-450k` (450,001 entries, semantic digest `0c523088…`, same
  recipe as exp-103). Required for the index and RSS gates.
  Cannot decide an accept as a real tree.
- Nominated real subjects for the aggregate gate: reconstructible `linux-v6.12` and this
  image’s `/usr` prefix.
- Gates, as already registered: index ≤1.4× `parfloor stat`; aggregate ≤1.25× on
  nominated real subjects; RSS ≤3× `arena_spike`; p95/median ≤1.5×.
- Void the floor and RSS ratios only when the prepared `arena_spike` cell has `max/min`
  above 2.0.
- Regime: `PERF_HOST_REGIME=quiet` first.
  Do not lower the 25% bar.
- Host is virtualized (4-core KVM). Record that.
  A pass would close H111. A fail that names leftover the Darwin composite left on the
  table gets a new id after H142, not a rewrite of H19–H22 / H60 / H7.
- Do not restart H86.

## What was measured

`make perf-floor` equivalent: `parfloor-stat`, `parfloor-enum`, `arena-spike`,
`aggregate` (`summary --no-controls`), `index` (`scan-index`), 4 workers, 30 trials, 3
warmups, interleaved.
Probe is the same HEAD binary as exp-138–140 (`86083632…`). Scoreboard:
[scoreboard-2026-09-20-linux-h111-floor.md](../reports/scoreboard-2026-09-20-linux-h111-floor.md).

Quiet was requested.
The start snapshot was 0.074/core.
Spike compile and the first subject’s 4-worker walks raised load; 121 of 150 timed
trials on `linux-450k` breached the 0.25/core gate.
The later two subjects had 0 breaches.
The table is labeled **uncontrolled** / screening-grade.
The 25% bar was not lowered.
No RAM disk.

Host: 4-core KVM Intel Xeon, 16 GiB, Linux 6.12.94+, ext4, virtualized.
Same class as exp-103. `os_cache: warm-steady`.

`linux-450k` is the balanced recipe at 450,000 target entries (450,001 including root).
Manifest `f93bffc3…`. Semantic digest `0c523088…` matches exp-103. Reconstructible:
`python -m benchmarks.generate create --recipe balanced --entries 450000`. The tree did
not mutate.

Companion self-pair (this artifact’s run JSON): same HEAD probe both arms,
`default-tree`, 12 timed pairs, uncontrolled, on the same `linux-450k` tree.
That pair is not the accept rule; it exists so `make perf-record` can lift a measured
run. The verdict is the floor table.

## What the accept rule said

On `linux-450k`, `arena_spike` spread 1.11 and `parfloor` 1.13, both under 2.0, so the
ratios can reject.

| Gate | Measured | Threshold | Result |
| --- | ---: | ---: | --- |
| 450k index ×floor | 1.78 | 1.40 | fail |
| 450k index RSS ×`arena_spike` | 5.20 | 3.00 | fail |
| 450k index p95/median | 1.018 | 1.50 | pass |
| `linux-v6.12` aggregate ×floor | 1.59 | 1.25 | fail |
| `usr-prefix` aggregate ×floor | 1.86 | 1.25 | fail |

450k index 321.76 ms / floor 180.92 ms.
Peak RSS 158.5 MiB / `arena_spike` 30.5 MiB. Aggregate on 450k is 1.46× (also above
1.25; that gate is scored on nominated reals).

The generated tree still hides distance from the floor: `linux-v6.12` index is 18.52×
and `/usr` index is 5.35×. Those index rows are context, not the 450k index gate.
`linux-v6.12` `arena_spike` spread 2.65, so that subject’s ceiling/RSS ratios abstain.

exp-103 on a similar 4-core VM failed at 2.60× wall and 6.59× RSS. This engine is closer
and still over the line.
Do not subtract the two hosts’ milliseconds.

## Judgment

**Fail.** H111’s pre-registered Linux floor/RSS gates do not pass on the current engine.
Leftover is the Darwin composite left on the table: the portable `getdents64`+`statx`
walk (H140) plus retained-index RSS above `arena_spike`. No engine patch.
Do not restart H86. Do not retry H19–H22 / H60 / H7 / H71. Do not raise the README 200K
files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
