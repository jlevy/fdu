---
title: "Linux cache-hit stack same versus #91 control"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-138
  title: "Linux cache-hit stack same versus #91 control"
  date: "2026-09-20"
  hypotheses:
    - H139
  subject:
    tree_label: linux-v6.12
    tree_root_id: d7c0dad8d82c8bb394f459b1dd77c9e8af60d0482cca92199519668546a5147e
    tree_engine_digest: a298a9c2c8f8ee910d22b87093a739993cdf153104590f4638eeee740b239bd0
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone .git directory as git left it."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293224
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
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
    control: "e667b739 #91 probe with H115 and H120 only"
    candidate: HEAD probe with H125 H129 H131 H133 stacked
    control_binary:
      name: control
      sha256: 0faa3c274d53df5d3294beb7f2c53b9c89efc01b0d58122203576645f67dec23
      size_bytes: 3016080
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
    run_artifact: /tmp/fdu-realtree/results/run-exp-138-h139-linux-cache-hit-stack.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 760890341.5
          candidate_median: 588926986.5
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.062
          change_pct: -22.483
          ci95_low_pct: -23.463
          ci95_high_pct: -21.394
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 669114189.5
          candidate_median: 503156482.5
          control_p95_over_median: 1.06
          candidate_p95_over_median: 1.044
          change_pct: -25.052
          ci95_low_pct: -25.825
          ci95_high_pct: -23.401
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 760363000.0
          candidate_median: 588397000.0
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.062
          change_pct: -22.554
          ci95_low_pct: -23.471
          ci95_high_pct: -21.416
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 703974500.0
          candidate_median: 551356500.0
          control_p95_over_median: 1.049
          candidate_p95_over_median: 1.06
          change_pct: -22.074
          ci95_low_pct: -22.58
          ci95_high_pct: -20.754
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 67969500.0
          candidate_median: 43945000.0
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.275
          change_pct: -33.734
          ci95_low_pct: -47.024
          ci95_high_pct: -9.25
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        blocked_ns:
          control_median: 555680.5
          candidate_median: 544167.5
          control_p95_over_median: 1.322
          candidate_p95_over_median: 1.244
          change_pct: -4.436
          ci95_low_pct: -13.855
          ci95_high_pct: 11.671
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 174045184.0
          candidate_median: 156200960.0
          control_p95_over_median: 1.001
          candidate_p95_over_median: 1.0
          change_pct: -10.244
          ci95_low_pct: -10.318
          ci95_high_pct: -10.223
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: superior
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
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -22.483
    reason: "same on Linux: content-cache-hit wall -22.48% [-23.46%, -21.39%] quiet on reconstructible linux-v6.12; RSS -10.24%; digest identical; no engine patch"
    commit: a5c98d59
---
## What was predicted

H139 is Linux replication of the landed #92 cache-hit stack, not a new cut.

Control is the #91 release probe at `e667b739` (H115 restore rebuild and H120 streaming
sidecar parse-into-apply only).
Candidate is this branch’s release probe, which stacks H125, H129, H131, and H133 on
that control.
H138 is also in the candidate binary; that change is a `content-query` walk
share and is not this job.

Named before measuring:

- Metric: `content-cache-hit` wall.
- Direction: down.
- Accept as *same*: median at least 3% faster and the 95% paired interval entirely below
  zero; content digest identical.
  RSS is reported, not the accept metric.
- Different would mean the stacked commits miss that rule, or only RSS / a component
  moves.
- Subject: reconstructible `linux-v6.12` (a clean metabrowser clone on this host is 916
  entries; Darwin’s 146k tree was workspace state).
- Regime: `PERF_HOST_REGIME=quiet` first.
  Do not lower the 25% bar.

Do not revert landed engine on a miss.
Do not mix Darwin milliseconds with Linux milliseconds.

## What was measured

Quiet start passed on Linux’s load/core gate: 0.115/core against 0.25. The cell held.
Final snapshot 0.153/core.
Instantaneous CPU busy is not collected on Linux; the 25% busy bar is the same load/core
rule the harness already uses here.
It was not lowered.

Subject: shallow clone of `github.com/torvalds/linux` tag `v6.12` at `adc21867` (92,474
entries / 86,643 files / 5,769 directories, max depth 14). Apparent 1.76 GiB / allocated
1.97 GiB (dense). Engine digest `a298a9c2…`. Reconstructible.
The tree did not mutate.

Host: 4-core KVM Intel Xeon, 16 GiB, Linux 6.12.94+, ext4, virtualized.
Same class as exp-103. `os_cache: warm-steady`. No RAM disk.

Job: harness `content-cache-hit` after one `content-seed` per variant into isolated
scratch. 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Control: `e667b739` probe, sha256 `0faa3c27…`. Candidate: HEAD probe (`a5c98d59`
engine), sha256 `86083632…`. 0 invalid samples.
Every timed sample was `source=content-cache` with 86,643 cache hits and 0 applied.
Content digest `06260f6c4f6d21c99fdf609ef68d99bf03ee8a1b7b47e2196ce56c456ec41e98` on
both arms.

| Arm | Wall median | Component | Peak RSS | p95/median wall |
| --- | ---: | ---: | ---: | ---: |
| control | 760.9 ms | 669.1 ms | 166.0 MiB | 1.065 |
| candidate | 588.9 ms | 503.2 ms | 149.0 MiB | 1.062 |

## What the accept rule said

Wall −22.48% [−23.46%, −21.39%]. ACCEPT. Median past 3% and the interval entirely below
zero.

Component −25.05% [−25.82%, −23.40%]. User CPU −22.07% [−22.58%, −20.75%]. Peak RSS
−10.24% [−10.32%, −10.22%]. All exclude zero.

This is the compounded H125+H129+H131+H133 stack against the #91 control in one pair,
not a new increment.
Darwin’s sequential pairs on metabrowser compounded about −28% uncontrolled.
The percentages are not the same number and must not be subtracted from each other.
The mechanism is the same: the landed stack still clears the 3% wall rule.

## Judgment

**Same** on Linux. Quiet confirmatory.
No engine patch. Do not revert.
Do not retry the individual cache-hit increments.
Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
