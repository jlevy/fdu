---
title: Post-0.1.0 uncontrolled baseline on the rustup store
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-105
  title: Post-0.1.0 uncontrolled baseline on the rustup store
  date: "2026-09-19"
  hypotheses: []
  subject:
    tree_label: rustup-toolchains
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: b839b65efbc7a3d0ff92161eb94e4236766ae5bd3dd3d6dcc5f057c7e7950772
    tree_provenance: "The rustup toolchain store for this machine installed toolchains. Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 77132
    tree_directories: 3418
    tree_files: 73714
    tree_symlinks: 0
    tree_apparent_bytes: 3099092158
    tree_allocated_bytes: 3326771200
    tree_max_depth: 17
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
    control: same probe at 285a41d2
    candidate: same probe self-comparison
    control_binary:
      name: control
      sha256: d19b1c4e30c7c2296a3d33ec6ab2daa0e2475f1a91e5a6e8656b9d30f0e900e7
      size_bytes: 2487408
      args: []
    candidate_binary:
      name: candidate
      sha256: d19b1c4e30c7c2296a3d33ec6ab2daa0e2475f1a91e5a6e8656b9d30f0e900e7
      size_bytes: 2487408
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-105-post-010-uncontrolled-baseline.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 297064437.5
          candidate_median: 302370896.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.061
          change_pct: 0.713
          ci95_low_pct: -0.247
          ci95_high_pct: 3.986
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 129733188.0
          candidate_median: 132958937.5
          control_p95_over_median: 1.085
          candidate_p95_over_median: 1.124
          change_pct: 1.24
          ci95_low_pct: -1.882
          ci95_high_pct: 7.492
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 828245000.0
          candidate_median: 835470500.0
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.122
          change_pct: -0.707
          ci95_low_pct: -1.886
          ci95_high_pct: 5.216
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 209075000.0
          candidate_median: 210904000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.021
          change_pct: 0.961
          ci95_low_pct: -0.164
          ci95_high_pct: 1.711
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 617567500.0
          candidate_median: 625703500.0
          control_p95_over_median: 1.084
          candidate_p95_over_median: 1.16
          change_pct: -1.184
          ci95_low_pct: -3.145
          ci95_high_pct: 6.669
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 25919488.0
          candidate_median: 25927680.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.012
          change_pct: 0.569
          ci95_low_pct: -3.438
          ci95_high_pct: 2.801
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
          control_median: 149786729.0
          candidate_median: 148675791.5
          control_p95_over_median: 1.104
          candidate_p95_over_median: 1.306
          change_pct: 2.459
          ci95_low_pct: -5.589
          ci95_high_pct: 18.059
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 143861333.5
          candidate_median: 141959458.5
          control_p95_over_median: 1.1
          candidate_p95_over_median: 1.328
          change_pct: 0.642
          ci95_low_pct: -5.731
          ci95_high_pct: 18.851
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 675889500.0
          candidate_median: 671305500.0
          control_p95_over_median: 1.076
          candidate_p95_over_median: 1.087
          change_pct: 0.346
          ci95_low_pct: -2.464
          ci95_high_pct: 3.511
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 61295500.0
          candidate_median: 61954500.0
          control_p95_over_median: 1.072
          candidate_p95_over_median: 1.16
          change_pct: 1.851
          ci95_low_pct: -1.864
          ci95_high_pct: 7.275
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 614498500.0
          candidate_median: 606574000.0
          control_p95_over_median: 1.077
          candidate_p95_over_median: 1.104
          change_pct: 0.543
          ci95_low_pct: -3.738
          ci95_high_pct: 3.979
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 34619392.0
          candidate_median: 34144256.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.03
          change_pct: -1.549
          ci95_low_pct: -2.437
          ci95_high_pct: 1.043
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
  reference_tools:
    - name: dust
      wall_ns_median: 175672687.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
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
    change_pct: 2.459
    reason: default-tree 149.8 ms and cold-scan-index 297.1 ms on 77132 entries; quiet cell could not hold
    commit: 285a41d2
---
## What was measured

A self-comparison of the 0.1.0 engine on this host after the request-model and
`.gitignore`-default-on refactor.
Both arms are the same `perf_probe` binary built from `285a41d2`
(`fdu 0.1.0-dev+g285a41d25`). The run establishes the current `default-tree` and
`cold-scan-index` numbers on the nominated rustup store, which has shrunk from the
175,191 entries recorded in exp-066 to 77,132.

The host is Darwin 25.5.0, Apple M1 Pro, bare metal, APFS, `os_cache: warm-steady`.
`PERF_HOST_REGIME=quiet` was attempted first.
A 12-trial quiet cell started, then `cold-scan-index` invalidated every sample (CPU busy
over 25% before and after).
`default-tree` kept 7–8 valid samples near 132 ms.
Desktop load (Cursor, ChatGPT, Arc) plus the scan itself cannot hold the quiet cell on
this machine tonight.
This artifact is the uncontrolled 12-pair run that followed, all samples valid.

## Numbers

On 77,132 entries (73,714 files), 12 interleaved pairs, uncontrolled:

| job | wall median | peak RSS |
| --- | ---: | ---: |
| `default-tree` | 149.8 ms | 33.0 MiB |
| `cold-scan-index` | 297.1 ms | 24.7 MiB |

Self-comparison wall was +2.46% `default-tree` and +0.71% `cold-scan-index`, both
intervals including zero, as required of identical binaries.

That is about 515k entries/s and 492k files/s on the default-tree probe job, and 260k
entries/s on `cold-scan-index`. These are probe numbers, not the installed CLI, and the
host was not quiet. They sit well above the README ballpark of 200K files/s, so that
figure is not an overclaim on this tree.
They are not a reason to raise it: the 2026-09-18 installed-CLI QA on a mutating fdu
checkout with nested worktrees saw 34,145 files in 0.43 s (~79k files/s), and a quiet
CLI cell was not obtained.

Compared with exp-066 on the same `root_id` at 175k entries, `default-tree` has moved
from about 1.3–1.5 s (that larger tree) to 150 ms on a store that is now 44% the size.
Do not divide those two walls into a speedup.

## What this does not settle

H108 (metadata one-shot stays `cold scan`) needs a quiet immutable CLI run.
H107 is the next recorded experiment, on the metabrowser checkout.
