---
title: Deciding-scale metadata walk profile after current engine
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-118
  title: Deciding-scale metadata walk profile after current engine
  date: "2026-09-19"
  hypotheses:
    - H122
  subject:
    tree_label: system-private-frameworks
    tree_root_id: b718281f3051a0ed5b4fc59d83614845f67e17999095cf2d837a0c551e24869c
    tree_engine_digest: 0c863b0ab28dc47e3db5a0298fe3239a51959056ec5b97c519e49ad1bfd965bf
    tree_provenance: "The sealed macOS system volume's private frameworks, read-only and identical on every Mac running the same OS build (Darwin 25.5.0 here). Reconstructible by installing that build."
    tree_reconstructible: true
    tree_entries: 158705
    tree_directories: 55256
    tree_files: 96542
    tree_symlinks: 6907
    tree_apparent_bytes: 5752378316
    tree_allocated_bytes: 3910119424
    tree_max_depth: 14
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
    control: same probe at 018b4c86
    candidate: same probe self-comparison
    control_binary:
      name: control
      sha256: 4b6b9aa9ff1353cfec5eb24ff5c37d7ddcda4f06494a52927350277bca61610b
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: 4b6b9aa9ff1353cfec5eb24ff5c37d7ddcda4f06494a52927350277bca61610b
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-118-h122-metadata-walk-profile.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1807680666.5
          candidate_median: 1873391771.0
          control_p95_over_median: 1.105
          candidate_p95_over_median: 1.09
          change_pct: 0.181
          ci95_low_pct: -2.909
          ci95_high_pct: 13.425
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1798744958.5
          candidate_median: 1866467375.0
          control_p95_over_median: 1.107
          candidate_p95_over_median: 1.09
          change_pct: 0.192
          ci95_low_pct: -2.996
          ci95_high_pct: 13.419
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 8957558500.0
          candidate_median: 9117425500.0
          control_p95_over_median: 1.449
          candidate_p95_over_median: 1.359
          change_pct: 1.869
          ci95_low_pct: -13.168
          ci95_high_pct: 20.732
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 338019500.0
          candidate_median: 333342000.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.052
          change_pct: -0.824
          ci95_low_pct: -2.235
          ci95_high_pct: 1.297
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 8623460000.0
          candidate_median: 8779329500.0
          control_p95_over_median: 1.465
          candidate_p95_over_median: 1.371
          change_pct: 1.976
          ci95_low_pct: -13.525
          ci95_high_pct: 21.769
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 89038848.0
          candidate_median: 89751552.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.021
          change_pct: 0.73
          ci95_low_pct: -0.542
          ci95_high_pct: 1.434
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
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
  reference_tools:
    - name: dust
      wall_ns_median: 1797495333.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 0.181
    reason: walk is 97 percent of default-tree component; leftover is __open plus getattrlistbulk; no Darwin cut named
    commit: 018b4c86
---
## What was predicted

H122: after the current engine (H115 + H120 + the #91 review fixes), a deciding-scale
metadata one-shot is still a cold walk.
H108 left the default command as a detached walk (~96% of instrumented `fdu PATH` wall).
Overnight optimized cache-hit restore and RSS, not that job.

Named before measuring:

- Determination: walk share (detached walk microseconds over instrumented component) is
  at least 90% of deciding-scale `default-tree` / `fdu PATH` wall on an immutable tree.
- The leftover stage is named (enumerate, stat, or consume) and is not already a
  rejected hypothesis.
- Second run with a snapshot present still walks; it does not load the snapshot (H108 /
  H9).
- Attachment: 12-pair same-binary `default-tree` wall.
  `FDU_COUNTERS` unset on that pair.
  Counters and a sample profile are attribution only.
- Quiet first. If the start gate fails, label uncontrolled.
  Do not lower the 25% bar.
- No engine patch unless the profile names one smallest falsifiable Darwin cut.

Subject: nominated `system-private-frameworks` (sealed, reconstructible).
Experiment id exp-118. exp-113 remains reserved.

## What was measured

Same release probe both variants (`018b4c86`, sha256 `4b6b9aa9…`). Job: harness
`default-tree` after one snapshot seed per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 40.3% CPU busy.
The pair ran as **uncontrolled**. Initial busy 49.45%; final 56.17%. Thermal `normal`.
The 25% bar was not lowered.
No RAM disk. Tree fingerprint matched the nominated digest (`0c863b0a…`) and did not
move.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,807.7 ms | 1,798.7 ms | 84.9 MiB |
| candidate | 1,873.4 ms | 1,866.5 ms | 85.6 MiB |

Self-comparison wall +0.18% [−2.91%, +13.43%]. Dust reference wall 1,797.5 ms.
Component is 99.5% of wall.
User CPU ~338 ms versus system ~8.6 s: the job is kernel-bound.

A later isolated counters-on pair (attribution only):

| Run | Component | Walk | Finish | Walk / component |
| --- | ---: | ---: | ---: | ---: |
| first (empty cache) | 1,913.1 ms | 1,848.8 ms | 5.7 ms | 96.6% |
| second (snapshot present) | 1,846.6 ms | 1,800.0 ms | 8.0 ms | 97.5% |

First `snapshot_written` true; second false (byte-identical, not rewritten).
Both runs: 55,256 directory opens, 158,705 metadata stats, 0 file opens, 0 control
reads, 0 same-parent path comparisons, one detached build of 158,704 entries.
Second run repeated the walk.

An 8-second `/usr/bin/sample` on the profiling build (`--repeat 10`, counters and oracle
off, 80,855 stacks):

| Layer / symbol | Share |
| --- | ---: |
| kernel/syscall | 84.18% |
| `__open` | 56.74% |
| `getattrlistbulk` | 17.79% |
| lock waits (`__psynch_mutexwait` + `semaphore_wait_trap`) | 10.86% |
| `fdu::scan` | 3.10% |
| allocator | 2.92% |
| `fdu::index` | 0.30% |
| `fdu::snapshot` | 0.09% |

Profile artifact: `/tmp/fdu-realtree/results/profile-exp-118-h122-metadata-walk.json`.

## What the determination said

Walk share is 96.6–97.5% of instrumented component, above 90%. **Confirmed.**

The leftover after the walk is finish (6–8 ms, 0.3–0.4% of component) plus report
render. That cannot clear 3% wall.
Inside the walk, the named leftover is directory `__open` and `getattrlistbulk`, not
consume.

## Judgment

H122 holds on the current engine.
Overnight did not change the default command.
A Darwin patch that is not H86/H111 and not a new `unsafe` `openat` (person-gated, same
class as H119) has no 3% target here.
Finish and snapshot rewrite are not the job.
Do not load a snapshot on `fdu PATH` (H108). Do not start a consume or Path rewrite.

No engine change. Do not raise the README 200K files/s or 4M cached lines/s from this
cell (~53k files/s on a directory-heavy prefix; the rustup probe still sits above the
ballpark).

Next measurement is H107 only on a tree whose ignored share is the walk, or H123
(product opened-root / refresh).
Not H121 from this profile.
Not H111 on this host.
