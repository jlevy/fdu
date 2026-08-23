---
title: Flush the rendered report before joining the snapshot writer
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-068
  title: Flush the rendered report before joining the snapshot writer
  date: "2026-08-23"
  hypotheses:
    - H101
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
    trials: 12
    warmups: 3
    interleaved: true
    control: "the command line at c013f1a: report buffered until after the save is joined"
    candidate: "out.flush() before pending_save.join(): same bytes, earlier"
    control_binary:
      name: control
      sha256: fd925c1564331d7f6387d4a1b08f20c44e4cb4dcdb3e162fd7cf10a82150b2b2
      size_bytes: 1561440
      args: []
    candidate_binary:
      name: candidate
      sha256: 8106bbe262211b068c5ecbbec29a44b4161a90a1701c9e5a0c7b26966db0ec8a
      size_bytes: 1561440
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-068-flush-before-join.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 353310708.5
          candidate_median: 361330187.5
          control_p95_over_median: 1.253
          candidate_p95_over_median: 1.177
          change_pct: 1.238
          ci95_low_pct: -14.222
          ci95_high_pct: 13.171
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 348400604.0
          candidate_median: 356227229.5
          control_p95_over_median: 1.253
          candidate_p95_over_median: 1.177
          change_pct: 1.164
          ci95_low_pct: -14.214
          ci95_high_pct: 12.791
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1779972500.0
          candidate_median: 1826650500.0
          control_p95_over_median: 1.13
          candidate_p95_over_median: 1.087
          change_pct: 2.676
          ci95_low_pct: -4.486
          ci95_high_pct: 10.024
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 212891000.0
          candidate_median: 206671500.0
          control_p95_over_median: 1.121
          candidate_p95_over_median: 1.059
          change_pct: -4.037
          ci95_low_pct: -11.632
          ci95_high_pct: 6.11
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1577551500.0
          candidate_median: 1629083500.0
          control_p95_over_median: 1.129
          candidate_p95_over_median: 1.068
          change_pct: 3.284
          ci95_low_pct: -5.087
          ci95_high_pct: 10.565
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 106070016.0
          candidate_median: 106127360.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.015
          change_pct: -0.823
          ci95_low_pct: -1.275
          ci95_high_pct: 1.969
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
    - job: default-tree-first
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 467667812.5
          candidate_median: 417804417.0
          control_p95_over_median: 1.155
          candidate_p95_over_median: 1.193
          change_pct: -4.046
          ci95_low_pct: -23.503
          ci95_high_pct: 12.975
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 451906687.5
          candidate_median: 407631187.5
          control_p95_over_median: 1.159
          candidate_p95_over_median: 1.178
          change_pct: -7.944
          ci95_low_pct: -22.87
          ci95_high_pct: 12.381
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1514920000.0
          candidate_median: 1463482000.0
          control_p95_over_median: 1.124
          candidate_p95_over_median: 1.114
          change_pct: -3.161
          ci95_low_pct: -11.72
          ci95_high_pct: 4.837
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 217472000.0
          candidate_median: 211343000.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.053
          change_pct: -1.905
          ci95_low_pct: -6.524
          ci95_high_pct: 2.78
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1292611000.0
          candidate_median: 1253674000.0
          control_p95_over_median: 1.149
          candidate_p95_over_median: 1.132
          change_pct: -3.816
          ci95_low_pct: -13.526
          ci95_high_pct: 6.49
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 114155520.0
          candidate_median: 113434624.0
          control_p95_over_median: 1.079
          candidate_p95_over_median: 1.03
          change_pct: 1.187
          ci95_low_pct: -5.638
          ci95_high_pct: 4.245
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
      wall_ns_median: 417236625.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 9
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 1.238
    reason: "Time to first byte on the real CLI -7.54% [-8.55%, -5.18%] repeated and -12.47% [-15.66%, -9.84%] first run with total wall unchanged; the engine-path guard in the frontmatter shows the expected nothing for a one-line latency change."
    commit: c013f1a
---
## What was measured

`fdu-n75m` part 1: the command line rendered its report into an 8 KiB `BufWriter`
(`cli.rs:1358`), then joined the snapshot writer (`cli.rs:603`) — serialization over
every entry, the CRC, the temp file, `F_FULLFSYNC`, the rename, and the index teardown
on that thread — and flushed only after `run()` returned (`cli.rs:1423`). A default
depth-2 tree is under 8 KiB, so nothing reached the terminal until all of that had
finished, and the comment at `cli.rs:595` describing render and save as overlapping was
describing an overlap the buffer defeated.

The change is one expression: the rendered report is flushed before the join.
Same bytes, same order, no work added or removed; only when the bytes arrive changes.

**The pre-registered signal is time to first stdout byte on the real command line**,
which no probe job can see — the flush is command-line code.
It was measured with a purpose-built paired script (`ttfb.py`, in the PR description):
control and candidate `fdu` binaries built from adjacent commits, counterbalanced pairs,
twelve trials per mode, the first byte timestamped as it is read from the pipe, the
snapshot isolated per arm under `XDG_CACHE_HOME`. The harness run in the frontmatter is
the engine-path guard: both probe arms carry the same engine, because the change touches
no engine code, and it shows the expected nothing.

## Result

On the 175k rustup store, twelve counterbalanced pairs per mode, uncontrolled host:

| mode | signal | control | candidate | paired change | interval |
| --- | --- | ---: | ---: | ---: | --- |
| repeated run (snapshot present) | time to first byte | 502.0 ms | 466.8 ms | **−7.54%** | [−8.55%, −5.18%] |
| repeated run | total wall | 503.5 ms | 508.4 ms | +0.05% | [−0.23%, +2.55%] |
| first run (no snapshot) | time to first byte | 535.8 ms | 460.7 ms | **−12.47%** | [−15.66%, −9.84%] |
| first run | total wall | 537.2 ms | 510.0 ms | −3.56% | [−6.64%, +0.28%] |

The report now reaches the terminal 41.5 ms before the process exits on a repeated run
and 49.3 ms before on a first run; with the control it arrived 1.4–1.6 ms before exit,
which is to say with it.
The first-run figure is larger because the join it no longer waits for includes the
snapshot write that exp-067 removed from the repeated run.

Total wall is unchanged on the repeated run, as a change that moves no work must leave
it. The first-run wall’s −3.56% has an interval that reaches zero and is not claimed.

## What remains of `fdu-n75m`

Parts 2 and 3 — taking the index teardown off the exit path, and whether a checksummed,
atomically renamed, fail-closed cache file needs `F_FULLFSYNC` rather than `fdatasync`
or nothing — are durability decisions, listed in Tier 3 of the macOS agenda for a
person. The 41.5 ms a repeated run still spends after its last byte is their upper bound
on this subject.

## Regime

Exploratory, warm-steady, uncontrolled and noisier than exp-066/067: the probe guard’s
intervals are ±13–23 points wide here, and `dust` beside it ran 417 ms against 272 ms
two hours earlier on the same tree.
The time-to-first-byte intervals are narrow because the effect is a fixed subtraction at
the end of every run rather than a change in the run’s variable part.
