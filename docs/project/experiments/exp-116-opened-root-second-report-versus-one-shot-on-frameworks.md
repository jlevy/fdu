---
title: Opened-root second report versus one-shot on frameworks
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-116
  title: Opened-root second report versus one-shot on frameworks
  date: "2026-09-19"
  hypotheses:
    - H117
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
    control: same probe default-tree one-shot
    candidate: opened-second-report second retained tree read
    control_binary:
      name: control
      sha256: 816345d911a4fcd98c1131691eb9da8c0f9dcceeb3c44dd468597018a54feda7
      size_bytes: 2537008
      args: []
    candidate_binary:
      name: candidate
      sha256: 816345d911a4fcd98c1131691eb9da8c0f9dcceeb3c44dd468597018a54feda7
      size_bytes: 2537008
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-116-h117-opened-second-report.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2612167500.5
          candidate_median: 2361886187.5
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.097
          change_pct: -2.158
          ci95_low_pct: -3.88
          ci95_high_pct: 2.753
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 2604637833.5
          candidate_median: 2350070417.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.099
          change_pct: -2.317
          ci95_low_pct: -4.218
          ci95_high_pct: 2.886
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 21664774500.0
          candidate_median: 17047917000.0
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.255
          change_pct: -3.808
          ci95_low_pct: -13.113
          ci95_high_pct: 1.26
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 362661500.0
          candidate_median: 351048000.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.089
          change_pct: -0.902
          ci95_low_pct: -3.916
          ci95_high_pct: 4.183
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 21290064500.0
          candidate_median: 16688454000.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.26
          change_pct: -3.893
          ci95_low_pct: -13.308
          ci95_high_pct: 1.359
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 89284608.0
          candidate_median: 89464832.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.011
          change_pct: 0.037
          ci95_low_pct: -0.409
          ci95_high_pct: 0.828
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
          - major_faults does not establish non-regression
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
          major_faults: inconclusive
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: opened-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 4003361167.0
          candidate_median: 3995320270.5
          control_p95_over_median: 1.128
          candidate_p95_over_median: 1.161
          change_pct: -0.08
          ci95_low_pct: -2.91
          ci95_high_pct: 1.4
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 1553354.5
          candidate_median: 1626229.5
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.086
          change_pct: 3.87
          ci95_low_pct: -4.525
          ci95_high_pct: 7.116
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 4630055000.0
          candidate_median: 4614147500.0
          control_p95_over_median: 1.05
          candidate_p95_over_median: 1.054
          change_pct: 0.35
          ci95_low_pct: -1.039
          ci95_high_pct: 1.119
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 2336377000.0
          candidate_median: 2334072000.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.026
          change_pct: -0.163
          ci95_low_pct: -0.604
          ci95_high_pct: 0.643
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 2296187000.0
          candidate_median: 2282535000.0
          control_p95_over_median: 1.097
          candidate_p95_over_median: 1.096
          change_pct: 0.821
          ci95_low_pct: -2.184
          ci95_high_pct: 2.833
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 205783040.0
          candidate_median: 205717504.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.007
          change_pct: -0.092
          ci95_low_pct: -0.445
          ci95_high_pct: 0.49
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
          - major_faults does not establish non-regression
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
          major_faults: inconclusive
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 2660713562.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 120
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: probe mode opened-second-report plus harness job; no engine serving change; no CLI flag
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -2.158
    reason: opened second report 1.6ms versus default-tree 2612ms (1630x); paired default-tree wall -2.158% (does not pass acceptance); determination kept; not a snapshot load
    commit: "984e4618"
---
## What was predicted

H108 confirmed that a metadata one-shot (`fdu PATH` / probe `default-tree`) throws the
index away and the second command stays a cold scan.
An opened root retains the index.
A second opened-root tree report on an unchanged Darwin tree should be at least 3%
faster than a one-shot of the same request, and several-fold if retention is the cost.

Named before measuring:

- Metric: second retained `ReadProjection::Report` (same `ViewSpec::Tree` query as
  `default-tree`) versus `default-tree` process wall.
- Subject: `system-private-frameworks` (H108 tree).
- Accept: opened second report at least 3% faster; expected several-fold.
  One-shot stays a cold scan.
  Not a license to load a snapshot on `fdu PATH`.
- Instrument: new probe mode `opened-second-report` (public `OpenedIndex` API, not a CLI
  flag). Discovery and the first report are setup; the component timer is only the second
  read.
- Same binary on both harness variants.
  `FDU_COUNTERS` unset.

Process wall of `opened-second-report` includes discovery and is not the claim.

## What was measured

Subject: nominated `system-private-frameworks` (158,705 entries / 96,542 files / 55,256
directories, max depth 14). Digest unchanged from the H108 nomination (`0c863b0a…`). The
tree did not mutate.

Jobs: harness `default-tree` (one-shot `prepare_report` + text render) and
`opened-second-report` (open, wait until Ready, two tree reports, time the second).
3 warmups, 12 timed pairs, interleaved, same probe both variants (sha256 `816345d9…`).
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 25.5% CPU busy.
The pair ran as **uncontrolled**. Initial busy 36.44%; final 15.0%. The 25% bar was not
lowered. No RAM disk.

0 invalid samples.

| Job | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| `default-tree` | 2,612.2 ms | 2,604.6 ms | 85.1 MiB |
| `opened-second-report` | 4,003.4 ms | 1.6 ms | 196.2 MiB |

`default-tree` remains a cold scan (`prepare_report` does not load the metadata
snapshot). The opened process wall is discovery plus two reports; only the 1.6 ms
component is the second report.

## What the accept rule said

Second retained report 1.6 ms versus one-shot wall 2,612.2 ms.
That is about 1,630× faster (99.94% down).
ACCEPT. The 3% bar is cleared by a wide margin, in the direction the registry named.

Same-binary `candidate_vs_control` is noise, as expected.

## Judgment

H117 is confirmed as a determination.
Retention, not a snapshot load, is the remaining metadata lever.
The probe mode is kept; no serving-policy change and no CLI flag.

Do not load a snapshot on `fdu PATH` from this result.
Do not raise the README 200K files/s from this cell.

H115 remains the standing *speed* best.
H117 is the standing metadata-retention determination.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
