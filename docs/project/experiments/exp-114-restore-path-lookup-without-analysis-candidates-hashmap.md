---
title: Restore path lookup without analysis_candidates HashMap
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-114
  title: Restore path lookup without analysis_candidates HashMap
  date: "2026-09-19"
  hypotheses:
    - H116
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
    control: HEAD at 7f289d5f with H115 in
    candidate: Index lookup plus restore-only classify skip
    control_binary:
      name: control
      sha256: 33716fe454af4444b90254e98423b4f49277d381a3c2797501406b86e3f6e247
      size_bytes: 2520464
      args: []
    candidate_binary:
      name: candidate
      sha256: 23fe6dcc626489af71bcd469756af76631ff8c8061728065aa2a21a69c54c825
      size_bytes: 2520464
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-114-h116-restore-path-lookup.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1475404333.5
          candidate_median: 1573737416.5
          control_p95_over_median: 4.251
          candidate_p95_over_median: 7.727
          change_pct: 8.701
          ci95_low_pct: -19.333
          ci95_high_pct: 63.899
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 1122875708.0
          candidate_median: 978591833.5
          control_p95_over_median: 3.796
          candidate_p95_over_median: 8.905
          change_pct: -16.201
          ci95_low_pct: -25.298
          ci95_high_pct: 43.914
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1217711000.0
          candidate_median: 1029316500.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.083
          change_pct: -15.053
          ci95_low_pct: -17.038
          ci95_high_pct: -13.297
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 1073448500.0
          candidate_median: 904259000.0
          control_p95_over_median: 1.034
          candidate_p95_over_median: 1.059
          change_pct: -15.646
          ci95_low_pct: -16.116
          ci95_high_pct: -14.321
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 144639500.0
          candidate_median: 123803000.0
          control_p95_over_median: 1.107
          candidate_p95_over_median: 1.271
          change_pct: -12.679
          ci95_low_pct: -25.191
          ci95_high_pct: -5.79
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        blocked_ns:
          control_median: 246666625.5
          candidate_median: 541226416.5
          control_p95_over_median: 20.3
          candidate_p95_over_median: 20.408
          change_pct: 71.222
          ci95_low_pct: -33.533
          ci95_high_pct: 247.217
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 395657216.0
          candidate_median: 351592448.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.006
          change_pct: -11.267
          ci95_low_pct: -11.651
          ci95_high_pct: -10.981
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
  reference_tools: []
  complexity:
    lines_changed: 50
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - wall interval includes zero under host contention
    notes: ""
  verdict:
    decision: rejected
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: 8.701
    reason: "wall +8.70 percent [-19.33%, +63.90%]; user CPU moved; engine reverted"
    commit: 7f289d5f
---
## What was predicted

H115 took the named apply/install cut on restore.
exp-109 left candidate install at 25.4% of restore: a full `analysis_candidates` walk
that classifies every file, then a `HashMap` drained by sidecar path.

H116 is not H113. H113 only skipped the second completeness `len()` walk after restore.
H116 deletes that Vec+HashMap inside `load_content_cache` and matches each sidecar
record with `Index::lookup`. Restore apply skips the classify staleness check because
the sidecar identity already pins the type rules.

Named before measuring:

- Metric: `content-cache-hit` wall on deciding-scale `metabrowser-clone`.
- Direction: down.
- Accept: median at least 3% faster and the 95% paired interval entirely below zero;
  content digest identical; incomplete sidecar still refused.
- Control: this branch HEAD at `7f289d5f` with H115 in.
  Claim-grade pair with `FDU_COUNTERS` unset.

exp-113 remains reserved for an H113 quiet confirmatory.

## What was measured

Subject: nominated `metabrowser-clone` (live checkout, 145,931 entries / 133,597 files /
11,512 directories, max depth 19). Same shape and engine digest as exp-108 through
exp-112 (`3fbfed48…`). The tree did not mutate during the pair.

Job: harness `content-cache-hit` after one `content-seed` per variant into an isolated
scratch snapshot. 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 85.6% CPU busy.
The pair ran as **uncontrolled**. Initial busy 76.31%; final 72.15%. Load-1m reached
152\. The 25% bar was not lowered.
No RAM disk.

Control is the release probe at `7f289d5f`, sha256 `33716fe4…`. Candidate is that probe
plus path lookup and restore-only classify skip, sha256 `23fe6dcc…`. 0 invalid samples.
The engine change was never committed.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,475.4 ms | 1,122.9 ms | 377.3 MiB |
| candidate | 1,573.7 ms | 978.6 ms | 335.3 MiB |

Every timed sample was `source=content-cache` with 133,597 cache hits / 0 applied.
Content digest `3b8cfa7183353657e70de585bc1262bbbf29161a9c9422a4a390526e94346ef1`, the
same digest exp-108 through exp-112 recorded.
Incomplete-sidecar fail-closed tests still pass.

Early pairs sat in the multi-second range (control 6.3 s / 7.8 s, candidate 13.8 s /
12.2 s). Blocked time +71.22% [−33.53%, +247.22%]. That is host contention, not the
lookup.

## What the accept rule said

Wall +8.70% [−19.33%, +63.90%]. REJECT. The median is the wrong direction and the
interval includes zero.

User CPU −15.65% [−16.12%, −14.32%] and peak RSS −11.27% [−11.65%, −10.98%] both clear
their intervals. Component −16.20% [−25.30%, +43.91%] includes zero.
The predicted classify walk is visible in CPU and faults; it is not a wall accept on
this cell.

## Judgment

H116 is rejected on the wall rule.
The engine change is reverted.

Do not retry H116 on another uncontrolled cell.
User CPU and RSS are not a substitute for the pre-registered wall metric.
A quiet confirmatory could still test whether the CPU cut becomes wall; that is not
tonight’s next item.

H115 remains the standing speed best.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
