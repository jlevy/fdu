---
title: Linux walk leftover is still the getdents64 plus statx floor
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-139
  title: Linux walk leftover is still the getdents64 plus statx floor
  date: "2026-09-20"
  hypotheses:
    - H140
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
    control: HEAD release probe both arms
    candidate: same probe leftover profile
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
    run_artifact: /tmp/fdu-realtree/results/run-exp-139-h140-linux-walk-leftover-uncontrolled.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 433645679.5
          candidate_median: 439365936.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.052
          change_pct: 0.636
          ci95_low_pct: -0.45
          ci95_high_pct: 3.575
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 431655573.5
          candidate_median: 437553750.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.052
          change_pct: 0.655
          ci95_low_pct: -0.425
          ci95_high_pct: 3.547
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 548078500.0
          candidate_median: 555488500.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.044
          change_pct: 1.056
          ci95_low_pct: 0.208
          ci95_high_pct: 2.296
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 444203000.0
          candidate_median: 449948000.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.046
          change_pct: 3.859
          ci95_low_pct: -0.905
          ci95_high_pct: 5.687
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 106205000.0
          candidate_median: 101907000.0
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.17
          change_pct: -9.14
          ci95_low_pct: -14.164
          ci95_high_pct: 6.608
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 45813760.0
          candidate_median: 45991936.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.003
          change_pct: 0.34
          ci95_low_pct: -0.218
          ci95_high_pct: 0.871
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
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 0.636
    reason: "walk still 95.7-96.1% of default-tree component; leftover is getdents64+statx floor; no userspace cut; do not retry H71"
    commit: a5c98d59
---
## What was predicted

H140 is the Linux leftover after the current engine, the H122/H128 analog.

Named before measuring:

- Job: same-binary 12-pair `default-tree` on reconstructible `linux-v6.12`.
- Walk share of component from counters-on `default-tree`.
- Determination: walk still at least 90% of component, leftover is `getdents64`+`statx`
  (expected), or a userspace stage at least 3% Darwin did not see.
- Do not compile a walk trim.
  Do not retry H71.
- Quiet first. If the gate fails or samples invalidate, label uncontrolled.
  Do not lower the 25% bar.

## What was measured

Quiet start passed (load/core 0.184). The cell did not hold: default-tree is a parallel
walk, and Linux load average remembered that work.
23 of 24 timed samples invalidated.
That incomplete quiet run is not a verdict and was not topped up.
Saved as `run-exp-139-h140-linux-walk-leftover.json`.

The claim-grade pair is **uncontrolled**. Same HEAD probe both arms (sha256
`86083632…`). 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.
Initial load/core 0.225; final 0.231. The 0.25 bar was not lowered.
No RAM disk.

Subject: the same frozen `linux-v6.12` clone as exp-138 (92,474 entries / 86,643 files /
5,769 directories). Fingerprint unchanged.
0 invalid samples on the uncontrolled pair.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 433.6 ms | 431.7 ms | 43.7 MiB |
| candidate | 439.4 ms | 437.6 ms | 43.9 MiB |

Self-comparison wall +0.64% [−0.45%, +3.58%]. Every timed sample was `source=scan` with
`snapshot_written` false.

Counters-on attribution (same binary, snapshot present):

| Run | Component | Walk | Finish | Walk / component | Opens | Stats |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| first | 452.8 ms | 433.4 ms | 2.0 ms | 95.7% | 5,769 | 92,474 |
| second | 435.9 ms | 418.1 ms | 1.6 ms | 95.9% | 5,769 | 92,474 |
| third | 468.9 ms | 450.9 ms | 1.9 ms | 96.1% | 5,769 | 92,474 |

`dir_enumeration_calls` stayed 0, the documented portable-path value.
One `directory open` per directory including the root.
One metadata stat per fingerprint entry.
File opens 0.

One first-run (`snapshot_written` true) was also the walk: 92.0% of component (426.4 ms
walk / 463.3 ms). Finish stayed ~2 ms.

`strace` is not installed on this image and has no apt candidate, so getdents64
multiplicity is not a counted ground truth here.
The playbook’s Linux floor remains 2.00 `getdents64`/dir plus per-entry `statx`. H71
already refuted rearranging that layer.

## What the determination said

**Same** leftover identity as Darwin H122/H128, different syscalls.
Walk is still the job (≥90% of component).
The leftover is the Linux syscall floor (`getdents64`+`statx`+one open per directory),
not a new userspace stage ≥3%.

No engine patch. Do not compile a walk trim.
Do not retry H71. Do not load a snapshot on `fdu PATH`. Do not raise the README 200K
files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
