---
title: "Linux adaptive unlock is silent; named-job --threads 8 is not a 3% win"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-146
  title: "Linux adaptive unlock is silent; named-job --threads 8 is not a 3% win"
  date: "2026-09-20"
  hypotheses:
    - H84
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
    control: "HEAD automatic workers (available.clamp(1,6)=4)"
    candidate: same probe --threads 8
    control_binary:
      name: control
      sha256: 2bc59b685deda30378ab83d0a9782ea8711451611bd03918f37183c602b978b7
      size_bytes: 3030680
      args: []
    candidate_binary:
      name: candidate
      sha256: 2bc59b685deda30378ab83d0a9782ea8711451611bd03918f37183c602b978b7
      size_bytes: 3030680
      args:
        - "--threads"
        - "8"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-146-h84-linux-threads-8.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 427289258.0
          candidate_median: 437997480.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.02
          change_pct: 1.754
          ci95_low_pct: 0.128
          ci95_high_pct: 4.146
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 425668426.0
          candidate_median: 436196430.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.021
          change_pct: 1.693
          ci95_low_pct: 0.142
          ci95_high_pct: 4.185
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 529829000.0
          candidate_median: 530239500.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.013
          change_pct: -0.375
          ci95_low_pct: -0.928
          ci95_high_pct: 0.958
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 440200500.0
          candidate_median: 434206500.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.035
          change_pct: -1.002
          ci95_low_pct: -3.228
          ci95_high_pct: 3.289
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 93043500.0
          candidate_median: 94039000.0
          control_p95_over_median: 1.165
          candidate_p95_over_median: 1.135
          change_pct: 4.532
          ci95_low_pct: -13.069
          ci95_high_pct: 18.17
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 37150720.0
          candidate_median: 37613568.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.004
          change_pct: 1.418
          ci95_low_pct: 0.956
          ci95_high_pct: 1.717
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: rejected
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 498430403.0
          candidate_median: 504486420.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.033
          change_pct: 0.254
          ci95_low_pct: -0.547
          ci95_high_pct: 2.333
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 422604672.5
          candidate_median: 428953514.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.036
          change_pct: 0.601
          ci95_low_pct: -0.374
          ci95_high_pct: 2.195
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 606469000.0
          candidate_median: 604581000.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.017
          change_pct: -0.183
          ci95_low_pct: -2.411
          ci95_high_pct: 0.935
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 509833500.0
          candidate_median: 508491000.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.021
          change_pct: -0.469
          ci95_low_pct: -4.296
          ci95_high_pct: 2.25
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 96331000.0
          candidate_median: 98414000.0
          control_p95_over_median: 1.088
          candidate_p95_over_median: 1.136
          change_pct: -3.247
          ci95_low_pct: -9.326
          ci95_high_pct: 13.128
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 36810752.0
          candidate_median: 37511168.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.004
          change_pct: 1.796
          ci95_low_pct: 1.063
          ci95_high_pct: 2.358
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
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: inconclusive
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
    notes: screen only; no engine change
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: 1.754
    reason: "unlock silent at ~2us/entry; named jobs --threads 8 no 3% win (aggregate +1.75% regression, index +0.25%); --no-controls is a warm sign not a shipped PORTABLE constant"
    commit: 0a979786
    kept: neither
---
## What was predicted

H84: `ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY` (30 µs) never fires against the Linux warm
floor (~1.5 µs), so an automatic scan stays at `available.clamp(1, 6)`. On this 4-core
host that start is already 4, with a reserve of 8 (`available * 2` clamped to 16).

Named before measuring:

- Confirm `adaptive_scale_ups` stays 0 and ns/entry stays far below 30 µs.
- Screen `--threads` on the named jobs `aggregate-summary` and `cold-scan-index`.
- A warm pair that clears 3% is a sign, not a shipped `PORTABLE` constant.
- Do not treat a 4-core VM sweep as H76 queue-depth evidence.
- Quiet first. If the gate fails or samples invalidate, label uncontrolled.
  Do not lower the 25% bar.

## What was measured

Unlock is silent. Three `FDU_COUNTERS=1` `scan-index --diagnostics` hits on
`linux-v6.12`:

| Hit | Component | Cal. entries | Cal. work | ns/entry | Expansions | Outcome |
| ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 1 | 434.1 ms | 16,438 | 32.9 ms | 2,002 | 0 | held |
| 2 | 433.5 ms | 16,385 | 30.7 ms | 1,874 | 0 | held |
| 3 | 462.9 ms | 16,403 | 30.2 ms | 1,843 | 0 | held |

`available_parallelism` 4, `initial_workers` 4, `maximum_workers` 8,
`peak_active_workers` 4, `slow_threshold_ns_per_entry` 30,000. Calibration fills the
16,384-entry window and holds.
Same silence on default `summary` (reserve expansions 0).

Named-job screen: same HEAD probe (`2bc59b68…`), automatic vs `--threads 8`, 3 warmups,
12 timed pairs, interleaved, `FDU_COUNTERS` unset.
Quiet start 0.054/core did not hold (invalid from aggregate pair #08). Claim-grade pair
is **uncontrolled**. Initial load/core 0.256; final 0.253. 0 invalid.
No RAM disk.

| Job | Control wall | Candidate wall | Change |
| --- | ---: | ---: | --- |
| `aggregate-summary` (default, gitignore-on) | 427.3 ms | 438.0 ms | +1.75% [+0.13%, +4.15%] regression |
| `cold-scan-index` | 498.4 ms | 504.5 ms | +0.25% [−0.55%, +2.33%] |

Default summary is index-bound (component ~426 ms, same class as `scan-index`).
`--threads 8` oversubscribes this 4-core host: involuntary context switches +85% on
aggregate. Peak RSS +1.4%. Not a 3% win.

`--no-controls` aggregate (transient floor, no retained index) is a different job.
That path *does* move:

| Subject | Threads | Regime | Control | Candidate | Change |
| --- | ---: | --- | ---: | ---: | --- |
| `linux-v6.12` | 8 | quiet, 0 invalid, load/core 0.034 | 33.6 ms | 31.8 ms | −5.42% [−6.74%, −3.37%] |
| `linux-v6.12` | 2 | quiet, 0 invalid | 34.9 ms | 59.0 ms | +70.47% [+66.01%, +73.02%] |
| `linux-450k` | 8 | quiet, 0 invalid | 241.8 ms | 215.7 ms | −10.45% [−13.34%, −8.05%] |
| `linux-450k` | 6 | uncontrolled (quiet incomplete) | 246.5 ms | 224.7 ms | −8.78% [−10.80%, −7.09%] |
| `linux-450k` | 16 | uncontrolled | 239.2 ms | 197.7 ms | −17.51% [−18.49%, −16.45%] |

`linux-450k` cannot decide a real-tree accept.
The `--no-controls` `linux-v6.12` quiet pair is a sign.
Both `--no-controls` accepts fail adaptive qualification on `minor_faults` (more workers
fault more). No knee yet at 16 on the transient tier.

## What the determination said

Unlock is silent on Linux warm (~2 µs/entry vs 30 µs).
Named jobs do not clear 3% at `--threads 8`. The transient `--no-controls` aggregate
does, and is still climbing at 16 workers.

No engine patch. Do not change `PORTABLE` to `measured`. Do not lower the unlock
threshold: that would give the default gitignore-on path the 8-worker pool that just
regressed it. H76 / `fdu-lf3v` still sizes the cold scalar on bare metal.
`fdu-tk1b` stays open.
