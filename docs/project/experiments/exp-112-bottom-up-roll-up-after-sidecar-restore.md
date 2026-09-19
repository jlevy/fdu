---
title: Bottom-up roll-up after sidecar restore
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-112
  title: Bottom-up roll-up after sidecar restore
  date: "2026-09-19"
  hypotheses:
    - H115
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
    control: current HEAD with H112 timers at 2736ec16
    candidate: one bottom-up ContentRollUp rebuild after restore inserts
    control_binary:
      name: control
      sha256: db4a47333c98dd1c1a70c625933e77ca43cef5056d5de5de977a30982dfc0a29
      size_bytes: 2503920
      args: []
    candidate_binary:
      name: candidate
      sha256: e3ffaf6ffa3dfec58c4115aa5654ab0adc9b3b49c514088a491b22e016d49c49
      size_bytes: 2520464
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-112-bottom-up-rollup-after-restore.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1287649395.5
          candidate_median: 1174093646.0
          control_p95_over_median: 1.509
          candidate_p95_over_median: 1.032
          change_pct: -9.688
          ci95_low_pct: -26.019
          ci95_high_pct: -7.135
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 959069354.5
          candidate_median: 855608958.0
          control_p95_over_median: 1.69
          candidate_p95_over_median: 1.045
          change_pct: -10.535
          ci95_low_pct: -30.228
          ci95_high_pct: -8.804
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 1243618500.0
          candidate_median: 1145324000.0
          control_p95_over_median: 1.077
          candidate_p95_over_median: 1.017
          change_pct: -8.624
          ci95_low_pct: -10.994
          ci95_high_pct: -7.005
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 1113937500.0
          candidate_median: 1022610500.0
          control_p95_over_median: 1.048
          candidate_p95_over_median: 1.009
          change_pct: -8.227
          ci95_low_pct: -11.105
          ci95_high_pct: -7.732
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 129681000.0
          candidate_median: 123741500.0
          control_p95_over_median: 1.328
          candidate_p95_over_median: 1.097
          change_pct: -2.231
          ci95_low_pct: -22.524
          ci95_high_pct: 0.127
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        blocked_ns:
          control_median: 42399312.5
          candidate_median: 23791729.0
          control_p95_over_median: 14.602
          candidate_p95_over_median: 2.417
          change_pct: -48.536
          ci95_low_pct: -86.751
          ci95_high_pct: -6.304
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 406454272.0
          candidate_median: 408756224.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.003
          change_pct: 0.539
          ci95_low_pct: 0.168
          ci95_high_pct: 0.819
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
    lines_changed: 167
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "restore-only commit_without_rollup plus ContentRollUp::merge; incremental commit unchanged; no unsafe"
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -9.688
    reason: "wall -9.69 percent [-26.02%, -7.13%]; user CPU confirms the apply cut; restore-only rebuild kept"
    commit: 7798fdc1
---
## What was predicted

H112 scoped the remaining sidecar restore cost to apply / commit / `merge_ancestors`.
H113 rejected the leftover completeness-count walk on wall.
H114 rejected the type-id `String` alloc on `ContentRollUp::add`.

H115 is the remaining named apply/install cut: sidecar restore still rebuilds content
roll-ups per file times depth.
After every restore insert, one bottom-up pass (each file into its parent, then each
directory into its parent from the deepest path) does the same totals in O(files +
dirs). Incremental `commit` and `invalidate` stay on `merge_ancestors`. This is not the
H86 EntryId rewrite.

Named before measuring:

- Metric: `content-cache-hit` wall on deciding-scale `metabrowser-clone`.
- Direction: down.
- Accept: median at least 3% faster and the 95% paired interval entirely below zero;
  content digest identical; intermediate directory roll-ups unchanged.
- Control: this branch HEAD at `2736ec16`, including the H112 off-by-default restore
  phase timers. Claim-grade pair with `FDU_COUNTERS` unset.

exp-108’s 1,121,963 `rollup_merges` is the metadata `merge_upward` counter, not
`ContentIndex::merge_ancestors` (2.43% named-symbol samples on that profile).
The experiment still tests the content restore shape.

## What was measured

Subject: nominated `metabrowser-clone` (live checkout, 145,931 entries / 133,597 files /
11,512 directories, max depth 19). Same shape and engine digest as exp-109 / exp-110 /
exp-111 (`3fbfed48…`). The tree did not mutate during the pair.

Job: harness `content-cache-hit` after one `content-seed` per variant into an isolated
scratch snapshot. 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 28.9% CPU busy.
The pair ran as **uncontrolled**. Initial busy 28.71%; final 59.14%. The 25% bar was not
lowered. No RAM disk.

Control is the release probe at `2736ec16`, sha256 `db4a4733…`. Candidate is that probe
plus restore inserts without ancestor merges and one `ContentIndex::rebuild_rollups`
after the apply loop, sha256 `e3ffaf6f…`. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,287.6 ms | 959.1 ms | 387.6 MiB |
| candidate | 1,174.1 ms | 855.6 ms | 389.8 MiB |

Every timed sample was `source=content-cache` with 133,597 cache hits and 0 applied.
Content digest `3b8cfa7183353657e70de585bc1262bbbf29161a9c9422a4a390526e94346ef1`, the
same digest exp-108, exp-109, exp-110, and exp-111 recorded.

A unit test compares incremental `commit` roll-ups to deferred inserts plus rebuild at
`""`, `a`, `a/b`, and `a/b/c`, including a directory that holds only nested files.

Control wall p95/median is 2.42× from host spikes (pairs at 1.94 s and 2.69 s).
Candidate wall p95/median is 1.05×. User CPU does not follow those spikes: −8.23%
[−11.11%, −7.73%].

## What the accept rule said

Wall −9.69% [−26.02%, −7.13%]. ACCEPT. The median is past 3% and the interval is
entirely below zero.

Component −10.54% [−30.23%, −8.80%] and user CPU −8.23% [−11.11%, −7.73%] agree.
Peak RSS +0.54% [+0.17%, +0.82%], within the 5% resource limit.

## Judgment

H115 is accepted on the wall rule.
The restore-only bottom-up rebuild is the remaining named apply/install cut, and it
moved wall. The engine change is kept.

The 1.12M metadata counter was the wrong name for the work; the content ancestor walk
was still large enough to clear the bar.
User CPU with a tight interval below zero is what makes that claim on an uncontrolled
cell.

`fdu-jxhk` remains the EntryId composite.
Do not restart that rewrite from this result.
Do not retry H114, parse-speed, or the H113 file-count shortcut.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
