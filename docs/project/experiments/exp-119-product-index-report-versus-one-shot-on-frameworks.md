---
title: Product Index.report versus one-shot on frameworks
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-119
  title: Product Index.report versus one-shot on frameworks
  date: "2026-09-19"
  hypotheses:
    - H123
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
    candidate: "index-second-report second query::report"
    control_binary:
      name: control
      sha256: b02fb0c9bcf97e0699b281d424ae2ef5568a5251ed6f9a387d5b78acebaffc06
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: b02fb0c9bcf97e0699b281d424ae2ef5568a5251ed6f9a387d5b78acebaffc06
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-119-h123-index-second-report.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2078314583.5
          candidate_median: 2140398166.5
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.096
          change_pct: 1.5
          ci95_low_pct: -0.195
          ci95_high_pct: 6.201
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 2064896083.0
          candidate_median: 2124134562.5
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.085
          change_pct: 1.549
          ci95_low_pct: -0.292
          ci95_high_pct: 6.522
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 8258347500.0
          candidate_median: 8788672500.0
          control_p95_over_median: 1.204
          candidate_p95_over_median: 1.114
          change_pct: 9.027
          ci95_low_pct: -5.046
          ci95_high_pct: 16.583
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 319091500.0
          candidate_median: 315800000.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.039
          change_pct: -0.219
          ci95_low_pct: -1.651
          ci95_high_pct: 2.025
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 7947167500.0
          candidate_median: 8474837500.0
          control_p95_over_median: 1.21
          candidate_p95_over_median: 1.117
          change_pct: 9.319
          ci95_low_pct: -5.248
          ci95_high_pct: 17.212
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 89522176.0
          candidate_median: 89120768.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.015
          change_pct: -0.153
          ci95_low_pct: -1.419
          ci95_high_pct: 1.254
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
    - job: index-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3131539145.5
          candidate_median: 3178856000.5
          control_p95_over_median: 1.663
          candidate_p95_over_median: 1.701
          change_pct: 0.869
          ci95_low_pct: -5.542
          ci95_high_pct: 15.522
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1672666.0
          candidate_median: 1915916.5
          control_p95_over_median: 1.664
          candidate_p95_over_median: 1.257
          change_pct: 10.57
          ci95_low_pct: -6.626
          ci95_high_pct: 28.077
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 8457730000.0
          candidate_median: 9102295000.0
          control_p95_over_median: 1.227
          candidate_p95_over_median: 1.108
          change_pct: 8.065
          ci95_low_pct: -7.116
          ci95_high_pct: 17.506
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 590948500.0
          candidate_median: 593640500.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.02
          change_pct: 1.322
          ci95_low_pct: 1.01
          ci95_high_pct: 2.32
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 7875677000.0
          candidate_median: 8497338000.0
          control_p95_over_median: 1.246
          candidate_p95_over_median: 1.116
          change_pct: 8.538
          ci95_low_pct: -7.569
          ci95_high_pct: 18.799
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 73662464.0
          candidate_median: 73367552.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.017
          change_pct: -0.247
          ci95_low_pct: -1.346
          ci95_high_pct: 1.454
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
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 2124797020.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 98
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: probe mode index-second-report plus harness job; no engine serving change; no CLI flag
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -99.919
    reason: product second report 1.7ms versus default-tree 2078ms (1222x); determination kept; not a snapshot load
    commit: ee014340
---
## What was predicted

H117 confirmed that a second opened-root tree report on an unchanged Darwin tree is
about 1,630× a one-shot of the same request.
That cell timed the public `OpenedIndex` read.
H123 is the product follow-on: a second `query::report` on a retained detached `Index`,
the same path Python `Index.report()` and CLI `--watch` `Session.report()` use.

Named before measuring:

- Metric: second retained `query::report` plus text render versus `default-tree` process
  wall.
- Subject: `system-private-frameworks` (same tree as H117 / H122).
- Accept: product second report at least 3% faster; expected several-fold.
  One-shot stays a cold scan.
  Not a license to load a snapshot on `fdu PATH`.
- Instrument: new probe mode `index-second-report` (public `scan_into_index` +
  `query::report`, not a CLI flag).
  Discovery and the first report are setup; the component timer is only the second
  report.
- Same binary on both harness variants.
  `FDU_COUNTERS` unset.

Process wall of `index-second-report` includes the setup scan and is not the claim.
No small engine serving patch was ready: watch is feature-gated out of the probe, and
the command line invents nothing.

## What was measured

Subject: nominated `system-private-frameworks` (158,705 entries / 96,542 files / 55,256
directories, max depth 14). Digest unchanged from the H108 nomination (`0c863b0a…`). The
tree did not mutate.

Jobs: harness `default-tree` (one-shot `prepare_report` + text render) and
`index-second-report` (scan into `Index`, two tree reports, time the second).
3 warmups, 12 timed pairs, interleaved, same probe both variants (sha256 `b02fb0c9…`).
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 100% CPU busy.
The pair ran as **uncontrolled**. Initial busy 100.0%; final 95.43%. The 25% bar was not
lowered. No RAM disk.

0 invalid samples.

| Job | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| `default-tree` | 2,078.3 ms | 2,064.9 ms | 85.4 MiB |
| `index-second-report` | 3,131.5 ms | 1.7 ms | 70.3 MiB |

`default-tree` remains a cold scan (`prepare_report` does not load the metadata
snapshot). The product-report process wall is the setup scan plus two reports; only the
1.7 ms component is the second `query::report`.

## What the accept rule said

Second product report 1.7 ms versus one-shot wall 2,078.3 ms.
That is about 1,222× faster (99.92% down).
ACCEPT. The 3% bar is cleared by a wide margin, in the direction the registry named.

Same-binary `candidate_vs_control` is noise, as expected.

## Judgment

H123 is confirmed as a determination.
The product `Index.report()` path already has the cheaper retained read H117 named.
The probe mode is kept; no serving-policy change and no CLI flag.

Do not load a snapshot on `fdu PATH` from this result.
Do not raise the README 200K files/s from this cell.

H115 remains the standing *speed* best.
H117 remains the opened-root probe.
H123 is the standing product-retention determination.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
