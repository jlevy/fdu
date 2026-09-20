---
title: Restore DFS joins parent path on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-130
  title: Restore DFS joins parent path on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H131
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
    control: H129 release probe at 6887a864
    candidate: analysis-file DFS joins parent path instead of path_of
    control_binary:
      name: control
      sha256: a4de4b7280af4e47e5b654ffb009133e4a34f5569178201b35a16b32dd7105c7
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: 84618edba302e624b423a8afa9b1f2f6d103fefda2d4d7e121360e5435054edf
      size_bytes: 2553504
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-130-h131-restore-parent-path-join.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 857800395.5
          candidate_median: 824309229.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.008
          change_pct: -4.07
          ci95_low_pct: -4.538
          ci95_high_pct: -3.28
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 568813771.0
          candidate_median: 535061000.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.019
          change_pct: -6.054
          ci95_low_pct: -6.466
          ci95_high_pct: -4.651
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 851265500.0
          candidate_median: 816613500.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.004
          change_pct: -4.068
          ci95_low_pct: -4.484
          ci95_high_pct: -3.846
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 791013000.0
          candidate_median: 756398000.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.004
          change_pct: -4.296
          ci95_low_pct: -4.693
          ci95_high_pct: -4.083
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 59540500.0
          candidate_median: 58901500.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.055
          change_pct: -0.997
          ci95_low_pct: -2.354
          ci95_high_pct: 2.871
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        blocked_ns:
          control_median: 6939145.5
          candidate_median: 8403500.0
          control_p95_over_median: 1.306
          candidate_p95_over_median: 1.395
          change_pct: -0.432
          ci95_low_pct: -12.112
          ci95_high_pct: 62.077
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 310132736.0
          candidate_median: 311222272.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.006
          change_pct: 0.379
          ci95_low_pct: -0.431
          ci95_high_pct: 1.001
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
    lines_changed: 20
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: private DFS join; public path_of unchanged; no dependency; no unsafe
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -4.07
    reason: "wall -4.07% [-4.54%, -3.28%] on frozen metabrowser-clone; digest identical; parent-path join kept"
    commit: 7840ce9b
---
## What was predicted

Quiet start this tick refused at CPU busy 27.23% > 25.0%. Tried once; skipped.
Do not retry file-count.
Do not label uncontrolled as quiet.

H130 left `path_of` at 11.85% of `content_open` after restore-without-classify.
The exp-129 sample split that function between ancestor walks (`index.rs:3359`) and
`PathBuf` collect (`index.rs:3364`).

H131: the analysis-file DFS already holds the parent, so each file path is
`parent.join(name)` instead of `path_of`. The public `path_of` stays.
The `HashMap` stays (not H116). Completeness stays the H125 restore-count.
Sidecar keys must match `path_of` byte-for-byte.

Named before measuring:

- Metric: `content-cache-hit` wall on frozen `metabrowser-clone`.
- Accept if the median is at least 3% faster and the 95% interval is entirely below
  zero; digest identical; incomplete-sidecar fail-closed still holds.
- Control = H129 release probe (`a4de4b72…` / engine `6887a864`).
- Candidate = DFS parent-path join.
- 12-pair, `FDU_COUNTERS` unset.
  Uncontrolled. Experiment id exp-130.

## What was measured

Official quiet check this tick: CPU busy **27.23% > 25.0%**. No quiet pair.

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Initial busy 25.84%; final 25.73%. Thermal `normal`. The 25%
bar was not lowered.
No RAM disk.

Control sha256 `a4de4b72…`. Candidate sha256 `84618edb…`. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 857.8 ms | 568.8 ms | 295.8 MiB |
| candidate | 824.3 ms | 535.1 ms | 296.8 MiB |

Wall −4.07% [−4.54%, −3.28%]. **Accepted.** Median past 3%; interval entirely below
zero. Component −6.05% [−6.47%, −4.65%]. User CPU −4.30% [−4.69%, −4.08%]. Peak RSS
+0.38% [−0.43%, +1.00%] non-inferior.

Every timed sample was `source=content-cache` with 133,708 cache hits and 0 applied.
Content digest `3be19a3e…` on both arms.

## What the prediction got right and wrong

Right about the mechanism: the ancestor walk was discarded work, join kept path
identity, and the wall bar cleared.
Right that this is not H116 — the `HashMap` stayed.

Wrong about treating 11.85% of `content_open` as the wall ceiling.
Component moved 6.05% and wall 4.07%, which matches the sample split: collect remains as
`PathBuf::join`. `path_of` as a restore-walk node should now be gone; join is the
leftover construction.

Do not retry H116. Do not mint a snapshot-parse cut.
Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
