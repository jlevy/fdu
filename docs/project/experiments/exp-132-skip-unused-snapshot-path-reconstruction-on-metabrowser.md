---
title: Skip unused snapshot path reconstruction on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-132
  title: Skip unused snapshot path reconstruction on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H133
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
    control: H131 release probe at 7840ce9b
    candidate: insert_loaded_child skips path_of when serving is None
    control_binary:
      name: control
      sha256: 84618edba302e624b423a8afa9b1f2f6d103fefda2d4d7e121360e5435054edf
      size_bytes: 2553504
      args: []
    candidate_binary:
      name: candidate
      sha256: 15cb7078d2e54f83ec351628571d0b59e2a0c31f6bfaedc8e23905a01008e0af
      size_bytes: 2553504
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-132-h133-skip-unused-snapshot-path-of.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 829510958.5
          candidate_median: 777956250.5
          control_p95_over_median: 1.314
          candidate_p95_over_median: 1.004
          change_pct: -6.366
          ci95_low_pct: -18.225
          ci95_high_pct: -5.656
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 541214292.0
          candidate_median: 487438125.0
          control_p95_over_median: 1.478
          candidate_p95_over_median: 1.007
          change_pct: -10.144
          ci95_low_pct: -24.219
          ci95_high_pct: -9.568
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 823880500.0
          candidate_median: 770697500.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.006
          change_pct: -6.501
          ci95_low_pct: -7.715
          ci95_high_pct: -5.87
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 762570000.0
          candidate_median: 711537500.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.006
          change_pct: -6.754
          ci95_low_pct: -7.697
          ci95_high_pct: -6.237
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 61689500.0
          candidate_median: 59786000.0
          control_p95_over_median: 1.122
          candidate_p95_over_median: 1.08
          change_pct: -3.555
          ci95_low_pct: -8.347
          ci95_high_pct: 0.725
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        blocked_ns:
          control_median: 7569854.5
          candidate_median: 7267479.5
          control_p95_over_median: 31.229
          candidate_p95_over_median: 1.378
          change_pct: -2.938
          ci95_low_pct: -86.721
          ci95_high_pct: 29.029
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 311771136.0
          candidate_median: 311877632.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.001
          change_pct: -0.058
          ci95_low_pct: -0.444
          ci95_high_pct: 0.189
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
    lines_changed: 47
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -6.366
    reason: "wall -6.37% [-18.23%, -5.66%] on frozen metabrowser-clone; digest identical; unused snapshot path skip kept"
    commit: 143a1c73
---
## What was predicted

Quiet start this tick refused at CPU busy 27.87% > 25.0%. Tried once; skipped.
Do not retry file-count.
Do not label uncontrolled as quiet.

H132 left restore-walk `path_of` gone and remaining `path_of` at 9.89% of
`content_open`, all under snapshot `insert_loaded_child`. One-shot snapshot load
constructs the index with `serving = None`, so `insert_serving_entry` discarded that
path.

H133: skip `path_of` in `insert_loaded_child` when serving is off.
Opened-root insert still reconstructs the path and fills serving indexes.
Public `path_of` stays.
Not H131 (restore DFS join stays).
Not H109 (no Path rewrite).
Not a snapshot-parse cut.
Not a snapshot load on `fdu PATH`.

Named before measuring:

- Metric: `content-cache-hit` wall on frozen `metabrowser-clone`.
- Accept if the median is at least 3% faster and the 95% interval is entirely below
  zero; digest identical; incomplete-sidecar fail-closed still holds.
- Control = H131 release probe (`84618edb…` / engine `7840ce9b`).
- Candidate = skip unused snapshot path reconstruction.
- 12-pair, `FDU_COUNTERS` unset.
  Uncontrolled. Experiment id exp-132.

## What was measured

Official quiet check this tick: CPU busy **27.87% > 25.0%**. No quiet pair.

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Initial busy 33.96%; final 29.48%. Thermal `normal`. The 25%
bar was not lowered.
No RAM disk.

Control sha256 `84618edb…`. Candidate sha256 `15cb7078…`. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 829.5 ms | 541.2 ms | 297.3 MiB |
| candidate | 778.0 ms | 487.4 ms | 297.4 MiB |

Wall −6.37% [−18.23%, −5.66%]. **Accepted.** Median past 3%; interval entirely below
zero. Component −10.14% [−24.22%, −9.57%]. User CPU −6.75% [−7.70%, −6.24%]. Peak RSS
−0.06% [−0.44%, +0.19%] non-inferior.

Every timed sample was `source=content-cache` with 133,708 cache hits and 0 applied.
Content digest `3be19a3e…` on both arms.

## What the prediction got right and wrong

Right about the mechanism: the reconstructed path was discarded work on one-shot load,
skipping it kept path identity and digest, and the wall bar cleared.
Right that this is not H131 — the restore DFS join stayed.

Wrong about treating 9.89% of `content_open` as the wall ceiling.
Component moved 10.14% and wall 6.37%. The interval is wide because control tails
reached 1.31× median; the candidate tail did not (1.00×).

Do not retry H116. Do not retry H131. Do not retry H109. Do not mint a snapshot-parse
cut. Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
