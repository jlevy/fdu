---
title: Cache-only restore omits classify on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-128
  title: Cache-only restore omits classify on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H129
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
    control: H125 release probe at be8d4d69
    candidate: restore-only candidates without classify; restore apply skips the classify self-check
    control_binary:
      name: control
      sha256: c86ad8cbeeec5a1cecc2b5b6f128a4913d3fec6e643ee8b569fb693001a3090f
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: a4de4b7280af4e47e5b654ffb009133e4a34f5569178201b35a16b32dd7105c7
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-128-h129-restore-omit-classify.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 986214770.5
          candidate_median: 855587646.0
          control_p95_over_median: 1.22
          candidate_p95_over_median: 1.011
          change_pct: -13.111
          ci95_low_pct: -20.221
          ci95_high_pct: -12.666
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 701913500.0
          candidate_median: 567068916.5
          control_p95_over_median: 1.297
          candidate_p95_over_median: 1.018
          change_pct: -19.192
          ci95_low_pct: -28.137
          ci95_high_pct: -18.461
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 978183000.0
          candidate_median: 847860500.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.005
          change_pct: -13.627
          ci95_low_pct: -15.008
          ci95_high_pct: -13.066
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 913623500.0
          candidate_median: 792595000.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.007
          change_pct: -13.465
          ci95_low_pct: -14.335
          ci95_high_pct: -12.866
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 65663000.0
          candidate_median: 53726000.0
          control_p95_over_median: 1.168
          candidate_p95_over_median: 1.089
          change_pct: -15.844
          ci95_low_pct: -23.883
          ci95_high_pct: -13.39
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        blocked_ns:
          control_median: 8049521.0
          candidate_median: 7800313.0
          control_p95_over_median: 24.017
          candidate_p95_over_median: 1.875
          change_pct: -9.646
          ci95_low_pct: -72.301
          ci95_high_pct: 33.386
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 351133696.0
          candidate_median: 308494336.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.009
          change_pct: -11.832
          ci95_low_pct: -13.745
          ci95_high_pct: -11.582
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
    lines_changed: 115
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: restore-only candidate walk and apply path; no dependency; no unsafe
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -13.111
    reason: "wall -13.11% [-20.22%, -12.67%] on frozen metabrowser-clone; digest identical; classify skip kept"
    commit: 6887a864
---
## What was predicted

Quiet start this tick refused at CPU busy 31.53% > 25.0%. Tried once; skipped.
Do not retry file-count.
Do not label uncontrolled as quiet.

H126 left the first `analysis_candidates` walk at 15.7% of `content_open` and called
that a slice, not a mechanism.
The exp-125 sample already split it:

| Inclusive node under `content_open` (15,296 samples) | Samples | Share |
| --- | ---: | ---: |
| `path_of` at `analysis_candidates` (index.rs:3484) | 1,096 | 7.17% |
| classify at `analysis_candidates` (index.rs:3489) | 909 | 5.94% |
| `root_path.join` (index.rs:3488) | 342 | 2.24% |
| classify guard at `apply_analysis_record` (index.rs:3563) | 1,125 | 7.36% |

Both classifies are a crate-private self-check.
Cache-only restore then commits the sidecar classification.
Combined classify is about 13.3% of `content_open`.

H129: restore walks file identities without classifying, and restore apply skips the
classify self-check.
The HashMap stays (not H116). `path_of` stays.
Completeness stays the H125 restore-count.
Incomplete sidecar still refused.

Named before measuring:

- Metric: `content-cache-hit` wall on frozen `metabrowser-clone`.
- Accept if the median is at least 3% faster and the 95% interval is entirely below
  zero; digest identical; incomplete-sidecar fail-closed still holds.
- Control = H125 release probe (`c86ad8cb…` / engine `be8d4d69`).
- Candidate = restore classify skip at `6887a864`.
- 12-pair, `FDU_COUNTERS` unset.
  Uncontrolled. Experiment id exp-128.

## What was measured

Official quiet check this tick: CPU busy **31.53% > 25.0%**. No quiet pair.

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Initial busy 33.76%; final 28.67%. Thermal `normal`. The 25%
bar was not lowered.
No RAM disk.

Control sha256 `c86ad8cb…`. Candidate sha256 `a4de4b72…`. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 986.2 ms | 701.9 ms | 334.9 MiB |
| candidate | 855.6 ms | 567.1 ms | 294.2 MiB |

Wall −13.11% [−20.22%, −12.67%]. **Accepted.** Median past 3%; interval entirely below
zero.
Component −19.19% [−28.14%, −18.46%]. User CPU −13.46% [−14.34%, −12.87%]. Peak RSS
−11.83% [−13.75%, −11.58%].

Every timed sample was `source=content-cache` with 133,708 cache hits and 0 applied.
Content digest `3be19a3e…` on both arms.

## What the counters said

`FDU_COUNTERS=1` hits after the pair (oracle on; attribution only).
Apply excludes decode.

| Arm | Hit | candidates µs | apply µs | parse µs |
| --- | ---: | ---: | ---: | ---: |
| control | 1 | 164,080 | 135,073 | 25,086 |
| control | 2 | 164,026 | 135,399 | 24,627 |
| control | 3 | 166,670 | 135,992 | 23,826 |
| candidate | 1 | 99,059 | 107,462 | 24,278 |
| candidate | 2 | 97,378 | 104,673 | 24,356 |
| candidate | 3 | 98,200 | 105,599 | 24,854 |

Candidates fell about 66 ms (classify gone from the walk; `path_of` and the HashMap
remain). Apply fell about 29 ms (classify guard gone).
Parse unchanged.

## What the prediction got right and wrong

Right about the component: both classifies were discarded work, and skipping them
cleared the wall bar.
Right that H116 was a different mechanism — keeping the HashMap did not repeat that
reject.

Wrong about the size of the leftover: the unpublished exp-125 split put classify at
13.3% of `content_open`; the claim-grade wall move was 13.11%, and component moved
19.19%. RSS also fell (no per-file `Classification` or `absolute_path` on restore).
That RSS move was not the accept metric.

`path_of` remains. That is still a slice.
Do not mint a threaded-path id unless a later profile names a ≥3% wall mechanism that is
not this skip.

Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
