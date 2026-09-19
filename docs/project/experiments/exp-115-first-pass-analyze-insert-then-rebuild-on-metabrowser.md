---
title: First-pass analyze insert-then-rebuild on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-115
  title: First-pass analyze insert-then-rebuild on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H118
  subject:
    tree_label: metabrowser-clone
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 3fbfed48354ed91f6933c70a8e798f21bbe7233d939926a8b768f7857428c541
    tree_provenance: "A clone of github.com/jlevy/metabrowser used as this host's source-checkout subject. The 2026-08 nominated path (fdu/benchmarks/corpus/realtree/metabrowser at 433fb6e plus workspace state) is gone from disk; this live checkout replaces it. The clone is reproducible; workspace state on top of it is not."
    tree_reconstructible: false
    tree_entries: 145931
    tree_directories: 11512
    tree_files: 133597
    tree_symlinks: 822
    tree_apparent_bytes: 1724995969
    tree_allocated_bytes: 2058641408
    tree_max_depth: 19
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
    control: "HEAD at 55261e6c with H115 in, H116 reverted"
    candidate: analyze_index apply_restored_analysis plus one rebuild
    control_binary:
      name: control
      sha256: 33716fe454af4444b90254e98423b4f49277d381a3c2797501406b86e3f6e247
      size_bytes: 2520464
      args: []
    candidate_binary:
      name: candidate
      sha256: d6e9169b8fa9a5cfd62d313e679c1d5007ad3262bd2b9a53c8866371ec10f976
      size_bytes: 2536976
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-115-h118-first-pass-insert-rebuild.json
  results:
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 10020589416.5
          candidate_median: 10022077833.0
          control_p95_over_median: 1.224
          candidate_p95_over_median: 1.555
          change_pct: -5.012
          ci95_low_pct: -13.281
          ci95_high_pct: 23.394
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 9232523562.5
          candidate_median: 9151214312.5
          control_p95_over_median: 1.198
          candidate_p95_over_median: 1.57
          change_pct: -2.604
          ci95_low_pct: -12.004
          ci95_high_pct: 23.865
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 20755239000.0
          candidate_median: 20309616000.0
          control_p95_over_median: 1.121
          candidate_p95_over_median: 1.046
          change_pct: -4.768
          ci95_low_pct: -8.029
          ci95_high_pct: -1.592
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 5814434000.0
          candidate_median: 5545743000.0
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.022
          change_pct: -4.53
          ci95_low_pct: -5.205
          ci95_high_pct: -3.305
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 14887299500.0
          candidate_median: 14777074500.0
          control_p95_over_median: 1.166
          candidate_p95_over_median: 1.056
          change_pct: -5.024
          ci95_low_pct: -9.767
          ci95_high_pct: -0.644
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 265445376.0
          candidate_median: 266518528.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.011
          change_pct: 1.387
          ci95_low_pct: -0.7
          ci95_high_pct: 7.95
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
          - "peak_rss_bytes straddles its +5% regression limit"
          - "minor_faults straddles its +10% regression limit"
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
          minor_faults: inconclusive
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 15
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - file I/O hides per-file ancestor walk on first-pass analyze
    notes: first-pass insert-then-rebuild measured and reverted; incremental apply_analysis kept
  verdict:
    decision: rejected
    primary_job: content-basic
    primary_metric: component_ns
    change_pct: -2.604
    reason: "component -2.60 percent [-12.00%, +23.86%]; file I/O hid ancestor walk; engine reverted"
    commit: 55261e6c
---
## What was predicted

H115 took the named apply/install cut on restore only.
First-pass `analyze_index` still `commit`s with `merge_ancestors` per file.
The receive loop can insert with `commit_without_rollup` and rebuild roll-ups once after
the pool joins.

Named before measuring:

- Metric: `content-basic` component (analyze only) on deciding-scale
  `metabrowser-clone`.
- Direction: down.
- Accept: median at least 3% faster and the 95% paired interval entirely below zero;
  content digest identical.
- Control: this branch HEAD at `55261e6c` (H115 in, H116 reverted).
  Claim-grade pair with `FDU_COUNTERS` unset.

The registry row said to refute if file I/O hides the ancestor walk.

## What was measured

Subject: nominated `metabrowser-clone` (live checkout, 145,931 entries / 133,597 files /
11,512 directories, max depth 19). Same shape and engine digest as exp-108 through
exp-114 (`3fbfed48…`). The tree did not mutate during the pair.

Job: harness `content-basic` (cold first-pass analyze after metadata setup).
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 39.7% CPU busy.
The pair ran as **uncontrolled**. Initial busy 82.08%; final 98.8%. The 25% bar was not
lowered. No RAM disk.

Control is the release probe at `7f289d5f` / `55261e6c` (engine unchanged), sha256
`33716fe4…`. Candidate is that probe plus insert-then-rebuild on the receive loop,
sha256 `d6e9169b…`. 0 invalid samples.
The engine change was never committed.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 10,020.6 ms | 9,232.5 ms | 253.1 MiB |
| candidate | 10,022.1 ms | 9,151.2 ms | 254.2 MiB |

Every timed sample was `source=scan` with 133,597 applied.
Content digest `3b8cfa7183353657e70de585bc1262bbbf29161a9c9422a4a390526e94346ef1`, the
same digest exp-108 through exp-114 recorded.

## What the accept rule said

Component −2.60% [−12.00%, +23.86%]. REJECT. The median misses the 3% bar and the
interval includes zero.

Wall −5.01% [−13.28%, +23.39%] also includes zero.
User CPU −4.53% [−5.21%, −3.31%] moved; that is not the predicted metric.

## Judgment

H118 is rejected on the pre-registered component rule.
File I/O hid the ancestor walk, which is the refute the registry named.
The engine change is reverted.

Do not retry H118 on another uncontrolled cell.
User CPU is not a substitute for the component metric.

H115 remains the standing speed best.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
