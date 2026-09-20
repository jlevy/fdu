---
title: "Linux content-query stack same versus #91 control"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-140
  title: "Linux content-query stack same versus #91 control"
  date: "2026-09-20"
  hypotheses:
    - H141
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
    candidate: HEAD probe with H138 share-one-every-entry walk
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
    run_artifact: /tmp/fdu-realtree/results/run-exp-140-h141-linux-content-query.json
  results:
    - job: content-query
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 12615994315.5
          candidate_median: 10409312500.5
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.009
          change_pct: -17.598
          ci95_low_pct: -18.072
          ci95_high_pct: -17.174
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 10314680644.0
          candidate_median: 8129485855.5
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.015
          change_pct: -21.325
          ci95_low_pct: -21.611
          ci95_high_pct: -20.762
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 17660812000.0
          candidate_median: 15444420000.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.005
          change_pct: -12.669
          ci95_low_pct: -13.177
          ci95_high_pct: -12.271
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 16867874500.0
          candidate_median: 14628234500.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.004
          change_pct: -13.709
          ci95_low_pct: -14.24
          ci95_high_pct: -13.074
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 787351000.0
          candidate_median: 824961000.0
          control_p95_over_median: 1.108
          candidate_p95_over_median: 1.107
          change_pct: 3.946
          ci95_low_pct: -1.421
          ci95_high_pct: 11.852
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 145512448.0
          candidate_median: 145758208.0
          control_p95_over_median: 1.001
          candidate_p95_over_median: 1.01
          change_pct: 0.236
          ci95_low_pct: -0.505
          ci95_high_pct: 1.037
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
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
    primary_job: content-query
    primary_metric: wall_ns
    change_pct: -17.598
    reason: "same on Linux: content-query wall -17.60% [-18.07%, -17.17%] uncontrolled on reconstructible linux-v6.12; digest identical; no engine patch"
    commit: a5c98d59
---
## What was predicted

H141 is Linux replication of landed H138, not a new cut.

Control is the #91 release probe at `e667b739` (H115 restore rebuild and H120 streaming
sidecar parse-into-apply only; four unfiltered views each walk `every_entry`). Candidate
is this branch’s release probe, which shares one `every_entry` walk across unfiltered
entry-row views (`a5c98d59`). The cache-hit stack is also in the candidate; that change
is not this job.

Named before measuring:

- Metric: `content-query` wall.
- Direction: down.
- Accept as *same*: median at least 3% faster and the 95% paired interval entirely below
  zero; report identity unchanged (content digest identical).
- Different would mean the shared walk misses that rule on Linux, or only a component
  moves.
- Subject: reconstructible `linux-v6.12` (same frozen clone as exp-138 / exp-139).
- Regime: `PERF_HOST_REGIME=quiet` first.
  Do not lower the 25% bar.

Do not revert landed engine on a miss.
Do not mix Darwin milliseconds with Linux milliseconds.
Do not invent a cache-hit skip.

## What was measured

Quiet start passed on Linux’s load/core gate: 0.185/core against 0.25. The cell did not
hold: content-query is a long parallel job, and Linux load average remembered that work.
The quiet attempt was stopped after four consecutive invalid warmups and was not topped
up.

The claim-grade pair is **uncontrolled**. Initial load/core 0.349; final 0.369. The 0.25
bar was not lowered.
No RAM disk.

Subject: the same frozen `linux-v6.12` clone as exp-138 (92,474 entries / 86,643 files /
5,769 directories). Fingerprint unchanged.
Engine digest `a298a9c2…`. Reconstructible.

Host: 4-core KVM Intel Xeon, 16 GiB, Linux 6.12.94+, ext4, virtualized.
Same class as exp-103. `os_cache: warm-steady`.

Job: harness `content-query` (scan + analyze untimed, then 100 four-view reports timed).
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Control: `e667b739` probe, sha256 `0faa3c27…`. Candidate: HEAD probe (`a5c98d59`
engine), sha256 `86083632…`. 0 invalid samples.
Every timed sample was `source=scan` with 100 query iterations, 86,628 analyzed / 86,643
applied. Content digest
`06260f6c4f6d21c99fdf609ef68d99bf03ee8a1b7b47e2196ce56c456ec41e98` on both arms.

| Arm | Wall median | Component | Peak RSS | p95/median wall |
| --- | ---: | ---: | ---: | ---: |
| control | 12,616.0 ms | 10,314.7 ms | 138.8 MiB | 1.018 |
| candidate | 10,409.3 ms | 8,129.5 ms | 139.0 MiB | 1.009 |

## What the accept rule said

Wall −17.60% [−18.07%, −17.17%]. ACCEPT. Median past 3% and the interval entirely below
zero.

Component −21.32% [−21.61%, −20.76%]. User CPU −13.71% [−14.24%, −13.07%]. Peak RSS
+0.24% [−0.51%, +1.04%] non-inferior.

This is the landed H138 share-one-walk against the #91 control in one pair, not a new
increment. Darwin’s cell on metabrowser was −18.76% uncontrolled.
The percentages are not the same number and must not be subtracted from each other.
The mechanism is the same: the shared `every_entry` walk still clears the 3% wall rule.

## Judgment

**Same** on Linux. Uncontrolled confirmatory after a quiet attempt that did not hold.
No engine patch. Do not revert.
Do not retry H138. Do not invent a cache-hit skip.
Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
