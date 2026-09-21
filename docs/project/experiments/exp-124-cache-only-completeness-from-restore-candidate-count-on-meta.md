---
title: Cache-only completeness from restore candidate count on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-124
  title: Cache-only completeness from restore candidate count on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H125
  subject:
    tree_label: metabrowser-clone
    tree_root_id: 3b5427f76be06cb475a2ea5c609bcd70f5d5f5b8af1280375c6a77b558513f50
    tree_engine_digest: dc0df2630acc6f604f7b76495f8214220d75fc600ac534d8c056f0210185ac5f
    tree_provenance: "An APFS copy-on-write clone of this host's github.com/jlevy/metabrowser checkout, taken 2026-09-19 after concurrent writers mutated the live path. Same shape as the live workspace at copy time. Not reconstructible."
    tree_reconstructible: false
    tree_entries: 146047
    tree_directories: 11517
    tree_files: 133708
    tree_symlinks: 822
    tree_apparent_bytes: 1726062376
    tree_allocated_bytes: 2060058624
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
    control: HEAD at af306146 with H115 and H120
    candidate: restore candidate count instead of a second analysis_candidates walk
    control_binary:
      name: control
      sha256: fd10aa87a8cd9701b91c2e21c469bf3c4ea066298cece44ba1230fe29495f54b
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: c86ad8cbeeec5a1cecc2b5b6f128a4913d3fec6e643ee8b569fb693001a3090f
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-124-h125-restore-candidate-count.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1063128708.5
          candidate_median: 975089584.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.016
          change_pct: -8.031
          ci95_low_pct: -10.791
          ci95_high_pct: -7.785
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 783411812.5
          candidate_median: 688428500.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.03
          change_pct: -11.481
          ci95_low_pct: -14.768
          ci95_high_pct: -11.035
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 1056972000.0
          candidate_median: 967098500.0
          control_p95_over_median: 1.034
          candidate_p95_over_median: 1.007
          change_pct: -8.41
          ci95_low_pct: -10.739
          ci95_high_pct: -7.837
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 993481500.0
          candidate_median: 897512000.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.008
          change_pct: -9.182
          ci95_low_pct: -10.807
          ci95_high_pct: -9.012
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 64663000.0
          candidate_median: 66688000.0
          control_p95_over_median: 1.171
          candidate_p95_over_median: 1.125
          change_pct: 3.283
          ci95_low_pct: -7.253
          ci95_high_pct: 12.577
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 7332458.0
          candidate_median: 6988437.5
          control_p95_over_median: 2.329
          candidate_p95_over_median: 2.916
          change_pct: -9.975
          ci95_low_pct: -44.245
          ci95_high_pct: 68.644
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 348930048.0
          candidate_median: 349560832.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.024
          change_pct: 0.129
          ci95_low_pct: -0.62
          ci95_high_pct: 0.603
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
  reference_tools: []
  complexity:
    lines_changed: 16
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: crate-private candidates count on ContentCacheLoad; shipped denominator is files visited, not HashMap length; incomplete-sidecar fail-closed kept; not the file-count heuristic
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -8.031
    reason: "wall -8.03 percent [-10.79%, -7.79%]; restore-count completeness kept; H113 superseded"
    commit: be8d4d69
---
## What was predicted

H113’s file-count completeness shortcut stays unshipped: this tick’s quiet start refused
at CPU busy **45.48% > 25.0%** (load 1.599/core).
exp-113 unused.

exp-123 left the second `analysis_candidates` walk in `open_for_report` at 16.0% of
`content_open` (~12% of H121 wall).
Restore already builds that set into a HashMap (H121: 48% of restore).
Completeness can compare `hits` to the count that walk already paid for.

Not H113’s file-count heuristic (`rollup.files` / `analysis_candidate_count`). Not H116
(the first candidates walk and HashMap stay).

Named before measuring:

- Metric: `content-cache-hit` wall on deciding-scale `metabrowser-clone`.
- Direction: down.
- Accept: median at least 3% faster and the 95% paired interval entirely below zero;
  content digest identical; incomplete sidecar still refused.
- Control: this branch HEAD at `af306146` with H115 and H120 in.
  Claim-grade pair with `FDU_COUNTERS` unset.
- Smallest fdu-core change: `ContentCacheLoad.candidates` stores `HashMap` len after the
  restore walk; cache-only completeness compares `hits` to that.

H125. Experiment id exp-124. exp-113 remains reserved.

## What was measured

H113 quiet start (this tick) refused at **45.48%**. No file-count pair.

A first H125 pair attempted `PERF_HOST_REGIME=quiet` after a later 23.21% busy check.
The start snapshot was 24.23%. The cell did not hold: 13 invalid samples (control n=6,
candidate n=5). That incomplete quiet run is not a verdict and was not topped up.
Saved as `run-exp-124-h125-quiet-incomplete.json`.

The claim-grade pair is **uncontrolled**. Initial busy 43.98% / thermal `normal`; final
26.55%. The 25% bar was not lowered.
No RAM disk.

Subject: frozen APFS clone of `metabrowser-clone` (146,047 entries / 133,708 files /
11,517 directories, max depth 19). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant into isolated
scratch. 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Control is the release probe at `af306146`, sha256 `fd10aa87…`. Candidate is that probe
plus restore-count completeness, sha256 `c86ad8cb…`. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,063.1 ms | 783.4 ms | 332.8 MiB |
| candidate | 975.1 ms | 688.4 ms | 333.4 MiB |

Every timed sample was `source=content-cache` with 133,708 cache hits and 0 applied.
Content digest `3be19a3e8a2330041bfc692dbc69cbb6f15482182a8a6d9d6276186ed54b1681`, the
same digest exp-120 through exp-123 recorded.

## What the accept rule said

Wall −8.03% [−10.79%, −7.79%]. ACCEPT. The median is past 3% and the interval is
entirely below zero.

Component −11.48% [−14.77%, −11.04%] and user CPU −9.18% [−10.81%, −9.01%] both exclude
zero. Those match a deleted second walk, and they are not a post-hoc metric switch.

Peak RSS +0.13% [−0.62%, +0.60%] non-inferior.

Adaptive qualification is inconclusive (`voluntary_context_switches` has no paired
percent interval). That is the same missing-interval note as exp-110 and does not
override the wall rule.

The incomplete-sidecar test still fails closed: a metadata-only pass that widens the
snapshot without rewriting the sidecar is refused under cache-only analysis.

## Judgment

H125 is accepted on the pre-registered wall rule.
The restore-count completeness check is kept.

H113’s leftover is gone.
The file-count shortcut is not compiled and is not retried.
exp-113 unused. H113 is superseded.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

## Errata (2026-09-21)

The recorded candidate described completeness as comparing hits to the HashMap length,
with `new_failure_modes: []`. The measured cell still had the R3 fail-open (map-length
denominator). Shipped code counts visited files.
Timing claims are unchanged.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
