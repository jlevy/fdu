---
title: "Linux fc-v49 baseline: default tree 182 ms, 2.4x pdu, with a false A/A accept on the summary"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-175
  title: "Linux fc-v49 baseline: default tree 182 ms, 2.4x pdu, with a false A/A accept on the summary"
  date: "2026-09-29"
  hypotheses: []
  subject:
    tree_label: linux-v6.12
    tree_root_id: 4ffe9d749fe638cd6c746668f7ae66d88e50225fa6903eabd8c4b9405b1aa245
    tree_engine_digest: 55a09ea65f48d38d88d7e2eacde95fa81d0a9970baabfeeba6f9e9418f478704
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293209
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v49
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: "e5a71c8a probe (0.2.1 engine + docs), copy A"
    candidate: "e5a71c8a probe, copy B (A/A)"
    control_binary:
      name: control
      sha256: 3ed8fdbfa1af82713b2b828ec0398cfaed3f3eabd14f55b6b738914849bd0d25
      size_bytes: 3758784
      args: []
    candidate_binary:
      name: candidate
      sha256: 3ed8fdbfa1af82713b2b828ec0398cfaed3f3eabd14f55b6b738914849bd0d25
      size_bytes: 3758784
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-175/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 149467501.5
          candidate_median: 144864626.5
          control_p95_over_median: 1.269
          candidate_p95_over_median: 1.035
          change_pct: -3.742
          ci95_low_pct: -12.334
          ci95_high_pct: -0.591
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 145247590.5
          candidate_median: 141048142.0
          control_p95_over_median: 1.275
          candidate_p95_over_median: 1.038
          change_pct: -3.434
          ci95_low_pct: -11.955
          ci95_high_pct: -0.467
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 419982500.0
          candidate_median: 422365000.0
          control_p95_over_median: 1.132
          candidate_p95_over_median: 1.044
          change_pct: -0.344
          ci95_low_pct: -5.762
          ci95_high_pct: 3.467
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 218741500.0
          candidate_median: 210109000.0
          control_p95_over_median: 1.096
          candidate_p95_over_median: 1.03
          change_pct: -5.358
          ci95_low_pct: -13.723
          ci95_high_pct: 0.852
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 209674000.0
          candidate_median: 211767000.0
          control_p95_over_median: 1.126
          candidate_p95_over_median: 1.154
          change_pct: -1.609
          ci95_low_pct: -6.43
          ci95_high_pct: 9.44
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
          - peak_rss_bytes is missing a paired percent interval
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
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 181868759.0
          candidate_median: 180696863.5
          control_p95_over_median: 1.055
          candidate_p95_over_median: 1.03
          change_pct: -1.12
          ci95_low_pct: -3.785
          ci95_high_pct: 3.122
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 176093702.0
          candidate_median: 174110000.5
          control_p95_over_median: 1.053
          candidate_p95_over_median: 1.037
          change_pct: -1.117
          ci95_low_pct: -3.9
          ci95_high_pct: 2.395
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 427825000.0
          candidate_median: 429263000.0
          control_p95_over_median: 1.048
          candidate_p95_over_median: 1.021
          change_pct: 0.996
          ci95_low_pct: -2.844
          ci95_high_pct: 2.468
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 199718500.0
          candidate_median: 198568500.0
          control_p95_over_median: 1.147
          candidate_p95_over_median: 1.093
          change_pct: 1.067
          ci95_low_pct: -3.68
          ci95_high_pct: 7.628
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 229759500.0
          candidate_median: 222676000.0
          control_p95_over_median: 1.08
          candidate_p95_over_median: 1.111
          change_pct: -0.55
          ci95_low_pct: -9.783
          ci95_high_pct: 5.615
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 35627008.0
          candidate_median: 35434496.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.018
          change_pct: -0.628
          ci95_low_pct: -1.451
          ci95_high_pct: 0.459
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
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -1.12
    reason: "quiet four-arm A/A of e5a71c8a on linux-v6.12: default-tree 181.9 ms (blind 89.9), aggregate-summary 149.5 ms (blind 77.8); default-tree A/A -1.12% [-3.79%, +3.12%] includes zero, aggregate-summary A/A -3.74% [-12.33%, -0.59%] would read as an accept"
    commit: null
    kept: control
---
## What was predicted

The first quiet cell of the 2026-09-29 overnight loop
([plan](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md), Q0) on a
new Firecracker kernel build, 6.18.44-fc-v49: no earlier absolute number transfers to it
(the registry review found 20–40% session-to-session drift on these hosts).

Four arms of one engine, `e5a71c8a` (the 0.2.1 engine plus documentation): `control` and
`candidate` are two copies of the same `perf_probe`, and each also runs with
`--no-controls`. Every comparison between copies of the same binary
(`candidate_vs_control`, `candidate-blind_vs_control-blind`) is an A/A and should
include zero. The width of those intervals is this host’s noise at 12 pairs.

## What was measured

Quiet regime (25% CPU-busy gate before and after every sample), 3 warmups, 12 pairs, no
invalid samples.

| Job | control | candidate (same binary) | `--no-controls` |
| --- | ---: | ---: | ---: |
| `default-tree` | 181.9 ms | 180.7 ms | 89.9 ms |
| `aggregate-summary` | 149.5 ms | 144.9 ms | 77.8 ms |

- `default-tree` A/A: −1.12% [−3.79%, +3.12%]; blind A/A +0.21% [−5.14%, +8.08%].
- `aggregate-summary` A/A: **−3.74% [−12.33%, −0.59%]**, which the accept arithmetic
  reads as ACCEPT although both arms are the same binary.
  Blind A/A −0.44% [−3.50%, +10.72%].
- `.gitignore` on against off: `default-tree` +102% (the blind arm is −50.71%
  [−51.79%, −49.84%]), `aggregate-summary` +92%.

The same night’s quiet tool cell (`make perf-compare-tools`, `fdu-default-tree`
contract, anchor `fdu` CLI at `e5a71c8a`, 12 pairs, no invalid samples, no semantic or
oracle mismatch) on this subject:

| Tool | Wall | Change against the adjacent fdu run |
| --- | ---: | --- |
| `fdu --color never PATH` | 0.19 s | baseline |
| pdu 0.24.0, default (`--silent-errors PATH`, new `pdu-default` contract) | 0.079 s | −58% |
| pdu 0.24.0, `--max-depth 2` | 0.074 s | −59% |
| diskus 0.9.0 | 0.085 s | −55% |

An uncontrolled A/A earlier the same session (load 0.7) read `default-tree` +8.26%
[−0.02%, +14.10%].

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-q0-tools-linux.json.gz`, the Q0 tool standing on `linux-v6.12` (fdu,
pdu, diskus, dut).

## Decision

Baseline. Two consequences are adopted for the rest of the night and stated here before
any candidate is measured:

1. At 12 pairs on this host an A/A can clear the −3% rule on `aggregate-summary`. A
   candidate predicted below 10% is measured at 20 pairs or treated as a screen, and an
   accept below 5% needs its placebo arms to include zero in the same cell.
2. A placebo that excludes zero by at most 3% is noted and does not block; one beyond 3%
   blocks the verdict.

The goal is stated against pdu’s default invocation: fdu’s default command is 2.4 times
pdu’s on this subject, and nearly all of it is `.gitignore` classification on one
thread.
