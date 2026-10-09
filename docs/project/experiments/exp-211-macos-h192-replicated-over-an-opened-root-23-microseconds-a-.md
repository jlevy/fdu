---
title: "macOS: H192 replicated over an opened root, 23 microseconds a report"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-211
  title: "macOS: H192 replicated over an opened root, 23 microseconds a report"
  date: "2026-10-09"
  hypotheses:
    - H192
  subject:
    tree_label: rustup
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: 4304d9d4071fd4478a0510edf80c4b78594f98b84969d90a1d7230eb0dc94d78
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains (root_id 36ce9b22). Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 77355
    tree_directories: 3427
    tree_files: 73928
    tree_symlinks: 0
    tree_apparent_bytes: 3750189949
    tree_allocated_bytes: 3978313728
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
    control: "148ef78e probe: main before the age column"
    candidate: "the exp-210 probe, built from the working tree that became 10ae731f before that commit existed, so only the binary's sha256 ties it to source (review C4 on #191); it predates 1b3ac793 and e25f12e3. The age column, each directory keeping its newest activity beside its roll-up"
    control_binary:
      name: control
      sha256: d2ac70ff129f6c510100a0f58a27677015fec20af2f8d29b732f6e29e2182041
      size_bytes: 3363840
      args: []
    candidate_binary:
      name: candidate
      sha256: b6f433efcc9a5d89a4159adb6ad1bea3b95c5318be77098e00f375ecb6f463f7
      size_bytes: 3380352
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-211/run.json.gz
  results:
    - job: opened-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1130107104.5
          candidate_median: 1121397208.5
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.312
          change_pct: -0.804
          ci95_low_pct: -3.834
          ci95_high_pct: 2.539
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 131729.5
          candidate_median: 153666.5
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.035
          change_pct: 17.785
          ci95_low_pct: 15.061
          ci95_high_pct: 21.383
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1175039500.0
          candidate_median: 1176229500.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.185
          change_pct: -0.663
          ci95_low_pct: -2.243
          ci95_high_pct: 1.482
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 837809500.0
          candidate_median: 839435500.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.027
          change_pct: -0.187
          ci95_low_pct: -0.645
          ci95_high_pct: 0.756
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 336889500.0
          candidate_median: 335825500.0
          control_p95_over_median: 1.131
          candidate_p95_over_median: 1.582
          change_pct: -1.857
          ci95_low_pct: -6.35
          ci95_high_pct: 3.37
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 115384320.0
          candidate_median: 115081216.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.012
          change_pct: -0.22
          ci95_low_pct: -0.669
          ci95_high_pct: 0.347
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
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 242120187.0
          candidate_median: 241061188.0
          control_p95_over_median: 1.318
          candidate_p95_over_median: 1.42
          change_pct: 0.346
          ci95_low_pct: -2.488
          ci95_high_pct: 4.99
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 65335146.0
          candidate_median: 68799979.0
          control_p95_over_median: 1.477
          candidate_p95_over_median: 1.325
          change_pct: 1.732
          ci95_low_pct: -5.252
          ci95_high_pct: 13.994
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 236163000.0
          candidate_median: 233179000.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.066
          change_pct: -1.411
          ci95_low_pct: -2.445
          ci95_high_pct: 0.279
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 228293000.0
          candidate_median: 226281000.0
          control_p95_over_median: 1.059
          candidate_p95_over_median: 1.068
          change_pct: -0.808
          ci95_low_pct: -2.276
          ci95_high_pct: 0.169
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 8191000.0
          candidate_median: 7685500.0
          control_p95_over_median: 1.133
          candidate_p95_over_median: 1.072
          change_pct: -6.394
          ci95_low_pct: -14.864
          ci95_high_pct: 8.685
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 4599750.0
          candidate_median: 6953188.0
          control_p95_over_median: 14.92
          candidate_p95_over_median: 11.459
          change_pct: 17.565
          ci95_low_pct: -32.439
          ci95_high_pct: 130.424
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 31735808.0
          candidate_median: 31694848.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.002
          change_pct: -0.077
          ci95_low_pct: -0.567
          ci95_high_pct: 0.233
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
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 187055187.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 208
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: the same change as exp-210
  verdict:
    decision: rejected
    primary_job: opened-second-report
    primary_metric: component_ns
    change_pct: 17.785
    reason: "not a speed decision: replicates exp-210 on the same binaries as the cost record of the age column, which ships regardless, held to the per-row bar set after exp-210's run: second tree report over an opened root +17.79% [+15.06%, +21.38%], 23 us; wall -0.80% [-3.83%, +2.54%], so exp-210's +1.56% opened wall did not reproduce; warm-snapshot-load component +1.73% [-5.25%, +13.99%] and wall +0.35% too wide to bound on a loaded host, so exp-210 remains the snapshot-load evidence"
    commit: 10ae731f
    kept: candidate
---
## What was predicted

A replication, on the same two binaries, of the two components
[exp-210](exp-210-macos-h192-maintained-activity-leaves-the-age-column-per-row.md)
measured closest to a bar: the per-report cost over an opened root, and the snapshot
load’s added maintenance.
H192’s bar applies: per-row work a report, and a snapshot load within +3%.

## What was measured

One interleaved probe run on the `rustup` toolchain store, control `148ef78e`, candidate
`10ae731f`, 3 warmups and 12 timed trials per variant, exploratory stage; the 1-minute
load average was 25 at the start and 49 at the end over 10 cores, the CPU 63% to 67%
busy. No sample was invalid and the tree was unchanged.

- `opened-second-report` component: 0.132 ms to 0.154 ms, +17.79% [+15.06%, +21.38%], a
  median 23 µs. Primary.
  Wall −0.80% [−3.83%, +2.54%].
- `warm-snapshot-load` component +1.73% [−5.25%, +13.99%] and wall +0.35%
  [−2.49%, +4.99%], both too wide to bound at +3% on this host.

## Decision

Not a speed decision, as exp-210 is not (review C3 on #191): a replication of its cost
record under the per-row bar, which was set after exp-210 ran, recorded `rejected` as a
speed claim with the candidate kept, because the age column ships regardless.
It puts the opened root’s price at 23 µs a report, the same order as exp-210’s 18.5 µs
and two orders below the pass’s 4.7 ms.
It does not bound the snapshot load’s maintenance more tightly; exp-210’s +2.21%
[+0.53%, +2.71%] remains the evidence for that.
exp-210’s opened-root wall signal, +1.56% [+1.06%, +3.75%], did not reproduce here:
−0.80% [−3.83%, +2.54%]. The binary is exp-210’s, which predates `1b3ac793` and
`e25f12e3`;
[exp-212](exp-212-macos-the-age-column-re-measured-at-the-shipped-head-per-row.md)
re-measures the shipped head with each binary tied to its commit.
