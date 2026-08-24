---
title: Skip the identical snapshot rewrite on the cold-scan path
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-067
  title: Skip the identical snapshot rewrite on the cold-scan path
  date: "2026-08-23"
  hypotheses:
    - H100
  subject:
    tree_label: rustup-toolchains
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: fa9b471063e3ffc70ade7d188d8416f9dfdd99f5bb525a74611203aa7681bc8e
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains. Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 175191
    tree_directories: 4956
    tree_files: 170235
    tree_symlinks: 0
    tree_apparent_bytes: 4900108867
    tree_allocated_bytes: 5420306432
    tree_max_depth: 16
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
    trials: 16
    warmups: 3
    interleaved: true
    control: main at 778aa74 with the default-tree probe mode (perf_probe.control)
    candidate: write_atomically compares the encoded bytes against the file and leaves an identical snapshot in place
    control_binary:
      name: control
      sha256: 64fab3d3060e99ef7b2456ffad47a43b282a16a4db913e8d3811b71983cbf9b3
      size_bytes: 1561392
      args: []
    candidate_binary:
      name: candidate
      sha256: fd925c1564331d7f6387d4a1b08f20c44e4cb4dcdb3e162fd7cf10a82150b2b2
      size_bytes: 1561440
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-067-skip-identical-snapshot-rewrite.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 672129271.0
          candidate_median: 660427791.0
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.114
          change_pct: 0.93
          ci95_low_pct: -5.782
          ci95_high_pct: 4.706
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        component_ns:
          control_median: 326043542.0
          candidate_median: 319328458.5
          control_p95_over_median: 1.185
          candidate_p95_over_median: 1.076
          change_pct: 1.133
          ci95_low_pct: -8.071
          ci95_high_pct: 6.492
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        cpu_ns:
          control_median: 2072146500.0
          candidate_median: 2094363500.0
          control_p95_over_median: 1.148
          candidate_p95_over_median: 1.046
          change_pct: -0.755
          ci95_low_pct: -5.75
          ci95_high_pct: 7.031
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        user_cpu_ns:
          control_median: 512107500.0
          candidate_median: 514428500.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.051
          change_pct: 1.306
          ci95_low_pct: -3.197
          ci95_high_pct: 3.998
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        system_cpu_ns:
          control_median: 1552357500.0
          candidate_median: 1592243000.0
          control_p95_over_median: 1.2
          candidate_p95_over_median: 1.067
          change_pct: -2.124
          ci95_low_pct: -6.736
          ci95_high_pct: 7.331
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        peak_rss_bytes:
          control_median: 86712320.0
          candidate_median: 86802432.0
          control_p95_over_median: 1.127
          candidate_p95_over_median: 1.056
          change_pct: 0.085
          ci95_low_pct: -1.525
          ci95_high_pct: 1.736
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 16
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 397725187.5
          candidate_median: 358707104.5
          control_p95_over_median: 1.148
          candidate_p95_over_median: 1.052
          change_pct: -10.612
          ci95_low_pct: -14.85
          ci95_high_pct: -6.054
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 16
        component_ns:
          control_median: 391981187.0
          candidate_median: 353170041.5
          control_p95_over_median: 1.146
          candidate_p95_over_median: 1.054
          change_pct: -10.398
          ci95_low_pct: -15.029
          ci95_high_pct: -6.1
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 16
        cpu_ns:
          control_median: 1785889500.0
          candidate_median: 1777663000.0
          control_p95_over_median: 1.121
          candidate_p95_over_median: 1.057
          change_pct: -1.569
          ci95_low_pct: -6.325
          ci95_high_pct: 4.37
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        user_cpu_ns:
          control_median: 214244000.0
          candidate_median: 196872500.0
          control_p95_over_median: 1.072
          candidate_p95_over_median: 1.108
          change_pct: -1.831
          ci95_low_pct: -13.232
          ci95_high_pct: 3.025
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        system_cpu_ns:
          control_median: 1580559500.0
          candidate_median: 1584947500.0
          control_p95_over_median: 1.123
          candidate_p95_over_median: 1.059
          change_pct: -1.806
          ci95_low_pct: -5.027
          ci95_high_pct: 3.392
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        peak_rss_bytes:
          control_median: 107249664.0
          candidate_median: 105889792.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.011
          change_pct: -1.285
          ci95_low_pct: -3.328
          ci95_high_pct: -0.385
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 16
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
    - job: default-tree-first
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 392807333.5
          candidate_median: 386971479.5
          control_p95_over_median: 1.126
          candidate_p95_over_median: 1.119
          change_pct: 0.099
          ci95_low_pct: -5.782
          ci95_high_pct: 7.604
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        component_ns:
          control_median: 386908333.5
          candidate_median: 381352604.5
          control_p95_over_median: 1.126
          candidate_p95_over_median: 1.118
          change_pct: -0.105
          ci95_low_pct: -5.714
          ci95_high_pct: 7.618
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        cpu_ns:
          control_median: 1779168500.0
          candidate_median: 1767021500.0
          control_p95_over_median: 1.163
          candidate_p95_over_median: 1.066
          change_pct: -0.644
          ci95_low_pct: -8.266
          ci95_high_pct: 4.175
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        user_cpu_ns:
          control_median: 209093000.0
          candidate_median: 217991000.0
          control_p95_over_median: 1.098
          candidate_p95_over_median: 1.088
          change_pct: 2.814
          ci95_low_pct: -1.208
          ci95_high_pct: 8.528
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        system_cpu_ns:
          control_median: 1575369500.0
          candidate_median: 1549582500.0
          control_p95_over_median: 1.167
          candidate_p95_over_median: 1.083
          change_pct: -2.291
          ci95_low_pct: -8.664
          ci95_high_pct: 4.781
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 16
        peak_rss_bytes:
          control_median: 105816064.0
          candidate_median: 106266624.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.028
          change_pct: -0.302
          ci95_low_pct: -1.122
          ci95_high_pct: 1.159
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 16
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
  reference_tools:
    - name: dust
      wall_ns_median: 307787375.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 117
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -10.612
    reason: "default-tree -10.61% [-14.85%, -6.05%] at 16 trials with the first-run and index jobs unchanged and RSS flat: a streamed byte compare for a 60-line change that cannot go stale."
    commit: c013f1a
---
## What was measured

exp-066 established that the repeated default run rewrote a 13.9 MB snapshot it never
reads, on every trial.
This experiment removes the rewrite: `snapshot::write_atomically` now compares the
encoded bytes against the file already at the path — streamed in 1 MiB pieces, so
deciding not to write does not cost a second allocation — and when they are identical it
leaves the file in place and moves only its mtime, which the loader reads as every
cached entry’s observation time and which the just-completed walk has earned.
Anything short of byte equality, including a file of the same length, writes as before.

The mechanism is the same one that closed the warm-open rewrite in `b020a1b`
(`ApplyStats::mutated()`), applied one layer down so that the cold-scan path — the one
`plan_report` routes every one-shot metadata query through — is covered too, along with
the content sidecar, which shares the writer.

Entry point: `write_atomically` and `same_bytes_on_disk` in
`crates/fdu-core/src/snapshot.rs`; the probe’s `snapshot_written` flag now keys on the
file’s identity (inode, creation time elsewhere) rather than its mtime, since the skip
path deliberately moves the mtime.

## Result

On the 175k rustup store, sixteen paired trials, uncontrolled host:

| job | control | candidate | paired change | interval |
| --- | ---: | ---: | ---: | --- |
| `default-tree` | 397.7 ms | 358.7 ms | **−10.61%** | [−14.85%, −6.05%] |
| `default-tree-first` | 392.8 ms | 387.0 ms | +0.10% | [−5.78%, +7.60%] |
| `cold-scan-index` | 672.1 ms | 660.4 ms | +0.93% | [−5.78%, +4.71%] |

Every candidate trial of `default-tree` reports `snapshot_written: false`; every control
trial reports `true`. Peak RSS is flat at 101–102 MiB on the default jobs, and the tail
narrowed: control p95/median 1.148 against candidate 1.052, which is what removing a 14
MB `F_FULLFSYNC` from the end of every run should do to a right tail.
No sample was invalidated and the tree did not move.

## Where the prediction was wrong

The registry row predicted at least 15% on `default-tree`, reasoning from exp-066’s 70
ms gap between the default run and `cold-scan-index`’s component.
The gap is real but it is not all the write: the rendered tree and the index teardown
are in it too, and the write itself is about 40 ms on this subject.
The mechanism was right, the job was right, the size was optimistic by a third.
`fdu-n75m` is the rest of that gap, and it is a latency change rather than a work
change, so it will show on time-to-first-byte rather than here.

## Regime

Exploratory, warm-steady, uncontrolled: the host still carried the runaway
`ANECompilerService` noted in exp-066, so sixteen trials rather than twelve.
The interval’s width (about nine points) is the noise floor of that regime; the effect
clears it with room.
