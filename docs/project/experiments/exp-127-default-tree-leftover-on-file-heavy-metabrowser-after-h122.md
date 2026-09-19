---
title: Default-tree leftover on file-heavy metabrowser after H122
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-127
  title: Default-tree leftover on file-heavy metabrowser after H122
  date: "2026-09-19"
  hypotheses:
    - H128
  subject:
    tree_label: metabrowser-clone
    tree_root_id: 3b5427f76be06cb475a2ea5c609bcd70f5d5f5b8af1280375c6a77b558513f50
    tree_engine_digest: dc0df2630acc6f604f7b76495f8214220d75fc600ac534d8c056f0210185ac5f
    tree_provenance: "An APFS copy-on-write clone of this host's github.com/jlevy/metabrowser checkout, taken 2026-09-19 after concurrent writers mutated the live path. Same shape as the live workspace at copy time. Not reconstructible."
    tree_reconstructible: false
    tree_entries: 146047
    tree_directories: 11517
    tree_files: 133708
    tree_symlinks: 822
    tree_apparent_bytes: 1726062376
    tree_allocated_bytes: 2060058624
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
    control: H125 release probe at be8d4d69
    candidate: same probe (leftover profile)
    control_binary:
      name: control
      sha256: c86ad8cbeeec5a1cecc2b5b6f128a4913d3fec6e643ee8b569fb693001a3090f
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: c86ad8cbeeec5a1cecc2b5b6f128a4913d3fec6e643ee8b569fb693001a3090f
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-127-h128-default-tree-leftover-metabrowser.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 355844708.0
          candidate_median: 359341208.5
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.219
          change_pct: 1.121
          ci95_low_pct: -2.912
          ci95_high_pct: 7.835
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 343438521.0
          candidate_median: 351331375.0
          control_p95_over_median: 1.13
          candidate_p95_over_median: 1.223
          change_pct: 1.893
          ci95_low_pct: -2.031
          ci95_high_pct: 7.846
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1719091500.0
          candidate_median: 1681140500.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.077
          change_pct: -2.797
          ci95_low_pct: -8.392
          ci95_high_pct: 5.116
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 171133000.0
          candidate_median: 171232000.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.018
          change_pct: -0.066
          ci95_low_pct: -2.222
          ci95_high_pct: 2.061
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1549950500.0
          candidate_median: 1513942500.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.083
          change_pct: -3.257
          ci95_low_pct: -9.003
          ci95_high_pct: 5.793
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 56745984.0
          candidate_median: 56819712.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.018
          change_pct: -0.013
          ci95_low_pct: -1.295
          ci95_high_pct: 1.419
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
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 1.121
    reason: "walk still the job on file-heavy metabrowser (93% of component); 1.952 getattrlistbulk/dir; snapshot not loaded; no new cut"
    commit: 59fa413a
---
H128 is the deciding-scale `default-tree` leftover on file-heavy frozen
`metabrowser-clone` after H122 (directory-heavy frameworks: walk 96%, 1.403
`getattrlistbulk`/dir).

Not H86. Not a snapshot load on `fdu PATH` (H108). Not H127 (opened-discovery).

Named before measuring:

- Metric: same-binary 12-pair `default-tree`; walk share of component from counters-on
  `default-tree`.
- Accept as determination if walk is still at least 90% of component, or if a named
  leftover (snapshot write, controls) is at least 3% and not already rejected.
- Same H125 probe both arms (`c86ad8cb…`). `FDU_COUNTERS` unset on the pair.
- Quiet first. If the start gate fails, label uncontrolled.
  Do not lower the 25% bar.
- Do not compile a cut unless a userspace stage is at least 3%.

Official quiet check refused at CPU busy **39.06% > 25.0%**.

## What was measured

Same release probe both variants (sha256 `c86ad8cb…`, 2,536,992 bytes).
Job: harness `default-tree` (snapshot present in scratch; `prepare_report` does not load
it for a metadata query).
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

The pair ran as **uncontrolled**. Official quiet check 39.06% busy.
Harness initial busy 38.02%; final 43.96%. Thermal `normal`. The 25% bar was not
lowered. No RAM disk.

Digest `dc0df263…`. 0 invalid samples.
Tree fingerprint unchanged.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 355.8 ms | 343.4 ms | 54.1 MiB |
| candidate | 359.3 ms | 351.3 ms | 54.2 MiB |

Self-comparison wall +1.12% [−2.91%, +7.83%].

Counters-on attribution (same binary, snapshot present, `snapshot_written` false,
`source=scan`):

| Run | Component | Walk | Finish | Walk / component | Opens | Enum / open |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| first | 344.9 ms | 320.4 ms | 2.6 ms | 92.9% | 11,517 | 1.952 |
| second | 228.8 ms | 203.9 ms | 2.4 ms | 89.1% | 11,517 | 1.952 |
| third | 235.7 ms | 209.9 ms | 2.6 ms | 89.1% | 11,517 | 1.952 |

The claim-grade pair’s component (343 ms) matches the first counters-on hit.
Walk is still the job.
Finish stays under 1%. Snapshot write did not run.
Enum multiplicity is the H127 first-pass figure (file-heavy), not H122’s 1.403
(directory-heavy frameworks).
Absolute wall is about 7× smaller than H122’s 2,408 ms on frameworks because this tree
has 11,517 directories, not 55,256.

No new 20 s sample: H127 already sampled this walk (`__open` 47%, `getattrlistbulk`
33%).

## What the determination said

H122 holds on file-heavy metabrowser: the second `fdu PATH` is still a cold walk.
Snapshot presence does not cheapen it (H108). No userspace cut ≥3%. The leftover remains
directory `__open` plus `getattrlistbulk` at 1.952 calls/dir including EOF.

Do not load a snapshot on `fdu PATH`. Do not mint a Darwin walk cut.
Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
