---
title: Progress handle attached against no handle
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-157
  title: Progress handle attached against no handle
  date: "2026-09-24"
  hypotheses:
    - H151
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
    control: progress-indicator probe at ead98807 with no handle
    candidate: "the same probe with --progress: a handle attached and polled every 80 ms"
    control_binary:
      name: control
      sha256: 72f2e7cd7f8b14be1ba4e9c7c3ffb6d7712aa0b338554d73ad7cb5c48c2b017b
      size_bytes: 2884256
      args: []
    candidate_binary:
      name: candidate
      sha256: 72f2e7cd7f8b14be1ba4e9c7c3ffb6d7712aa0b338554d73ad7cb5c48c2b017b
      size_bytes: 2884256
      args:
        - "--progress"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /private/tmp/fdu-prog/perf/results/run-exp-157-progress-handle-attached.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2289714062.5
          candidate_median: 2323001979.5
          control_p95_over_median: 1.088
          candidate_p95_over_median: 1.169
          change_pct: 5.755
          ci95_low_pct: -5.341
          ci95_high_pct: 10.906
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 2283429583.0
          candidate_median: 2316617208.0
          control_p95_over_median: 1.088
          candidate_p95_over_median: 1.17
          change_pct: 5.801
          ci95_low_pct: -5.289
          ci95_high_pct: 11.078
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 15143554500.0
          candidate_median: 14643062000.0
          control_p95_over_median: 1.116
          candidate_p95_over_median: 1.251
          change_pct: 11.477
          ci95_low_pct: -17.545
          ci95_high_pct: 30.682
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 343210500.0
          candidate_median: 340724000.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.073
          change_pct: -0.627
          ci95_low_pct: -1.448
          ci95_high_pct: 6.727
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 14804967000.0
          candidate_median: 14298410000.0
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.256
          change_pct: 11.643
          ci95_low_pct: -17.902
          ci95_high_pct: 31.534
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 89317376.0
          candidate_median: 89522176.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.018
          change_pct: 0.544
          ci95_low_pct: -0.349
          ci95_high_pct: 1.054
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
    lines_changed: 318
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "The probe flag, poller, and handle check (271 insertions, 47 deletions in perf_probe.rs, tests included); the engine handle itself is exp-156."
  verdict:
    decision: in-progress
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 5.755
    reason: "default-tree wall +5.75% [-5.34%, +10.91%]: upper bound past the +3% gate, lower bound below it, so neither noninferiority nor a cost is established; uncontrolled cell at 100% to 72% busy, user CPU -0.63%; needs a quiet re-run"
    commit: ead98807
    kept: control
---
# Progress handle attached against no handle

## What was predicted

H151: with a `Progress` handle attached and read by a poller every 80 ms, the default
command costs the same as without one.
The engine’s share of the indicator is what an interactive `fdu` run adds: walkers add
their counts to three shared atomics once per chunk of directories (at most four
directories), phases are stored at boundaries, and a second thread takes a snapshot on
every tick. Gate, fixed before the run: `default-tree` paired wall interval upper bound
at most +3%.

## What was changed

`ead98807` adds `--progress` to the probe for the two modes that prepare a one-shot
report, `default-tree` and `summary`. The report goes through
`prepare_report_with_progress` while a thread polls `Progress::snapshot` every 80 ms,
the command line ticker’s redraw interval, and is joined as soon as the report is
prepared, as the ticker is.
It polls from the start, where the ticker waits 500 ms, so it reads at least as often as
the command line. It draws nothing.
After the component timer stops, the probe refuses the run unless the handle’s files and
bytes equal the report’s walked totals, so every candidate sample had a live handle.
Every other mode refuses the flag, as does `--diagnostics` with it.

## What was measured

One immutable binary (`72f2e7cd…`, 2,884,256 bytes) in both arms, the candidate carrying
`--progress` through the harness’s per-variant arguments
(`--variant "candidate=PATH --progress"`), so code generation is identical.

Subject: `system-private-frameworks` (158,705 entries), digest `0c863b0a…` unchanged, no
mutation. Job: `default-tree` only, 3 warmups, 12 timed pairs, interleaved.
The coordinator narrowed the cell to that job and 12 pairs to free the host; the
`summary` mode and the `--no-controls` summary fold were not measured with a handle.

Labeled **uncontrolled**. `PERF_HOST_REGIME=quiet` refused at the start gate (CPU busy
100% > 25%). Initial busy 100.0%; final 72.4%. Thermal `normal`. The 25% bar was not
lowered. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| no handle | 2,289.7 ms | 2,283.4 ms | 85.2 MiB |
| handle attached | 2,323.0 ms | 2,316.6 ms | 85.4 MiB |

Wall +5.75% [−5.34%, +10.91%]. Component +5.80% [−5.29%, +11.08%]. The upper bound is
past +3%, so the gate is not met; the lower bound is below +3%, so inferiority is not
established either. The two marginal medians differ by +1.45%.

Where the cost would come from:

- The pairs split by time.
  The first six pairs ran the candidate slower by +8.2% to +36.7%; the last six ran
  between −18.1% and +3.3%, as the host fell from 100% to 72% busy.
  A fixed per-run cost would not change sign with host load.
- User CPU moved −0.63% [−1.45%, +6.73%]; the engine did not do measurably more work in
  user space. System CPU +11.64% [−17.90%, +31.53%] and involuntary context switches
  +2.11% [−8.15%, +10.09%] are too wide to read.
  Minor faults +0.65% [+0.23%, +0.92%] and peak RSS +0.54% [−0.35%, +1.05%]: one extra
  thread stack and a few pages.
- An estimate, not a measurement: 55,256 directories in chunks of at most four is 13,800
  to 55,256 flushes of three relaxed adds to one shared line, and about 29 snapshots.
  Even at a contended line transfer each, that is milliseconds of CPU across ten
  threads, not the 130 ms the median would imply.

No `FDU_COUNTERS=1` attribution was run for this pair: the counters count filesystem,
index and allocation work, not atomics, and the host was handed to another measurement
as soon as this pair finished.

## What the determination said

Not decided. This cell cannot show that the handle stays within noise, and it does not
show a cost the mechanism can account for.
H151 stays open until a quiet cell of at least 12 pairs on `default-tree` holds the +3%
gate; `summary` (which falls closed to the index) and the `--no-controls` summary fold
should be paired in the same cell.
The probe flag is kept for that re-run.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
