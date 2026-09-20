---
title: Stream sidecar parse-into-apply on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-117
  title: Stream sidecar parse-into-apply on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H120
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
    control: HEAD at 984e4618 with H115 in
    candidate: stream sidecar records into apply without a decoded Vec
    control_binary:
      name: control
      sha256: 33716fe454af4444b90254e98423b4f49277d381a3c2797501406b86e3f6e247
      size_bytes: 2520464
      args: []
    candidate_binary:
      name: candidate
      sha256: 216ff35e16a7841bcad559e7aa3963f8cc395b07719d0fbbad4662d149766a63
      size_bytes: 2537008
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-117-h120-stream-sidecar-parse-into-apply.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1111019834.0
          candidate_median: 1103236583.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.03
          change_pct: -0.594
          ci95_low_pct: -1.628
          ci95_high_pct: 0.511
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 822507375.0
          candidate_median: 812471083.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.029
          change_pct: -0.896
          ci95_low_pct: -2.343
          ci95_high_pct: 0.494
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1102302000.0
          candidate_median: 1094277500.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.018
          change_pct: -0.656
          ci95_low_pct: -1.271
          ci95_high_pct: 0.425
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 1008576500.0
          candidate_median: 1007051500.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.006
          change_pct: 0.056
          ci95_low_pct: -0.518
          ci95_high_pct: 0.501
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 94008500.0
          candidate_median: 87856000.0
          control_p95_over_median: 1.114
          candidate_p95_over_median: 1.104
          change_pct: -8.132
          ci95_low_pct: -16.774
          ci95_high_pct: -1.72
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        blocked_ns:
          control_median: 10792292.0
          candidate_median: 9355229.0
          control_p95_over_median: 1.649
          candidate_p95_over_median: 2.379
          change_pct: 2.007
          ci95_low_pct: -27.631
          ci95_high_pct: 54.975
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 395886592.0
          candidate_median: 355868672.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.003
          change_pct: -10.127
          ci95_low_pct: -10.492
          ci95_high_pct: -10.025
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
  reference_tools: []
  complexity:
    lines_changed: 80
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: parse-into-apply; fail-closed clear_content on a bad record; no unsafe
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: peak_rss_bytes
    change_pct: -10.127
    reason: "peak RSS -10.13 percent [-10.49%, -10.03%]; wall non-inferior; streaming restore kept"
    commit: "bbfd7d1c"
---
## What was predicted

Cache-only restore holds the decoded sidecar records `Vec` beside the files map.
Streaming parse-into-apply drops that transient copy.
H116 already failed to remove the candidate HashMap; this is the Vec only.

Named before measuring:

- Metric: `content-cache-hit` peak RSS on deciding-scale `metabrowser-clone`.
- Accept: peak RSS down at least 10%; wall non-inferior (interval upper bound below
  +3%); content digest identical.
- Control: this branch HEAD at `984e4618` with H115 in, H116/H118 reverted.
  Claim-grade pair with `FDU_COUNTERS` unset.

A corrupt record after a prefix of applies is a miss: `clear_content` then return the
default load.

## What was measured

Subject: nominated `metabrowser-clone` (145,931 entries / 133,597 files / 11,512
directories, max depth 19). Same shape and engine digest as exp-108 through exp-115
(`3fbfed48…`). The tree did not mutate during the pair.

Job: harness `content-cache-hit` after one `content-seed` per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 28.1% CPU busy.
The pair ran as **uncontrolled**. Initial busy 19.82%; final 90.64%. The 25% bar was not
lowered. No RAM disk.

Control is the release probe at `7f289d5f` / `984e4618` (engine unchanged until this
patch), sha256 `33716fe4…`. Candidate streams sidecar records into apply, sha256
`216ff35e…`. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,111.0 ms | 822.5 ms | 377.5 MiB |
| candidate | 1,103.2 ms | 812.5 ms | 339.4 MiB |

Every timed sample was `source=content-cache` with 0 applied (cache hits).
Content digest `3b8cfa7183353657e70de585bc1262bbbf29161a9c9422a4a390526e94346ef1`, the
same digest exp-108 through exp-115 recorded.

## What the accept rule said

Peak RSS −10.13% [−10.49%, −10.03%]. ACCEPT. The median meets the 10% bar and the
interval is entirely below −10%.

Wall −0.59% [−1.63%, +0.51%] is non-inferior (upper bound below +3%).

## Judgment

H120 is accepted on the pre-registered RSS rule.
The streaming restore is kept.

H115 remains the standing wall-speed best.
This is the standing content-hit RSS best.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
