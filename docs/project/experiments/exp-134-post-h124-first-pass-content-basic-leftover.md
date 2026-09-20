---
title: Post-H124 first-pass content-basic leftover
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-134
  title: Post-H124 first-pass content-basic leftover
  date: "2026-09-19"
  hypotheses:
    - H135
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
    control: HEAD release probe at 6a93fc12
    candidate: same probe (leftover profile)
    control_binary:
      name: control
      sha256: 8765aa6f198cacf94027411ea9151125b58af2ac4458c9f45b782ec1bfc71124
      size_bytes: 2553504
      args: []
    candidate_binary:
      name: candidate
      sha256: 8765aa6f198cacf94027411ea9151125b58af2ac4458c9f45b782ec1bfc71124
      size_bytes: 2553504
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-134-h135-post-h124-content-basic-leftover.json
  results:
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 10365916250.5
          candidate_median: 10058770625.5
          control_p95_over_median: 1.36
          candidate_p95_over_median: 1.31
          change_pct: -3.704
          ci95_low_pct: -8.25
          ci95_high_pct: 6.962
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 9363756020.5
          candidate_median: 9071863208.5
          control_p95_over_median: 1.408
          candidate_p95_over_median: 1.349
          change_pct: -2.67
          ci95_low_pct: -8.938
          ci95_high_pct: 7.095
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 24181783000.0
          candidate_median: 23226157500.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.126
          change_pct: -2.65
          ci95_low_pct: -11.874
          ci95_high_pct: 6.336
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 5701439500.0
          candidate_median: 5681484500.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.009
          change_pct: 0.029
          ci95_low_pct: -1.685
          ci95_high_pct: 0.413
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 18437735500.0
          candidate_median: 17615872500.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.164
          change_pct: -2.939
          ci95_low_pct: -15.881
          ci95_high_pct: 8.979
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 228220928.0
          candidate_median: 267935744.0
          control_p95_over_median: 1.174
          candidate_p95_over_median: 1.005
          change_pct: 3.523
          ci95_low_pct: -0.506
          ci95_high_pct: 18.043
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: content-basic
    primary_metric: wall_ns
    change_pct: -3.704
    reason: "first-pass leftover after H124 is still I/O (read 58.87%, open 16.09%); classify_with 2.02% of process; commit_record 0.59%; no new skippable 3% wall cut; no engine patch"
    commit: 6a93fc12
---
## What was predicted

H124 rejected a type/size gate and a larger read chunk on first-pass `content-basic`.
H118 rejected insert-then-rebuild.
H119 screened walk-overlap (`read` 59%, `__open` 17%). H134 closed the Darwin
`content-cache-hit` leftover hunt.

This cell is a leftover profile on a different component, not a cache-hit skip.

Named before measuring:

- Determination: after H124, apply-path classify / `apply_analysis` / candidate install
  is or is not a userspace stage ≥3% of `content-basic` wall that is not already
  rejected.
- If no skippable ≥3% mechanism appears, do not compile a cut in this cell.
- Attachment: 12-pair same-source `content-basic`, `FDU_COUNTERS` unset.
- Attribution: 20 s `/usr/bin/sample` on the profiling build plus counters-on hits.

Subject: frozen APFS clone of `metabrowser-clone`. Experiment id exp-134. H135. No
engine change.

Quiet start this tick refused at CPU busy 56.4% > 25.0%. Tried once; skipped.
Uncontrolled. Do not lower the 25% bar.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-basic` after metadata setup (scan is outside the component timer).
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Official quiet check 56.4% busy.
Pair initial 35.75%; final 82.48%. Thermal `normal`. The 25% bar was not lowered.
No RAM disk.

Same HEAD release probe both variants (`8765aa6f…` / 2,553,504 bytes).
0 invalid samples. Self-comparison only.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 10,365.9 ms | 9,363.8 ms | 217.6 MiB |
| candidate | 10,058.8 ms | 9,071.9 ms | 255.5 MiB |

Wall −3.70% [−8.25%, +6.96%]. Attachment only.
Interval includes zero.

Every timed sample applied 133,708 / analyzed 118,882 / binary 14,159. Content digest
`3be19a3e…`. Source `scan`.

## What first-pass `content-basic` spends time on

Three `FDU_COUNTERS=1` hits (scan is setup; sidecar timing stays 0):

| Hit | Component ms | File opens | Read calls | Bytes read | Walk µs |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 7,736 | 125,686 | 249,533 | 951,822,681 | 302,276 |
| 2 | 8,281 | 125,686 | 249,533 | 951,822,681 | 296,642 |
| 3 | 7,907 | 125,686 | 249,533 | 951,822,681 | 315,604 |

Opens and reads match H124. Walk stays ~0.3 s against an 8 s component.

A 20-second `/usr/bin/sample` on the profiling build (`--repeat 3`, counters and oracle
off). Process thread-root samples 123,034:

| Inclusive node | Samples | Share of process |
| --- | ---: | ---: |
| `read` (kernel) | 72,430 | 58.87% |
| `__open` (kernel) | 19,799 | 16.09% |
| `classify_with` | 2,483 | 2.02% |
| `commit_record` | 721 | 0.59% |
| `merge_ancestors` | 527 | 0.43% |
| `scan_into_index` (setup) | 584 | 0.47% |
| `path_of` (probe summary after the timer) | 96 | 0.08% |

Main-thread `content_analysis` 11,794 samples: `analyze_index` 89.00%, `commit_record`
6.11%, `classify_with` 1.41%. Apply is visible on the receive thread and hidden on wall,
which is the H118 leftover.

`classify_with` at 2.02% of process cannot clear 3% wall even if every classify were
skipped. Worker classify is content-family admission and is required for lines.
The apply-path self-check is a slice of that 2%.

## What the determination said

First-pass leftover after H124 is still file I/O (`read` 58.87%, `__open` 16.09%). No
new skippable userspace stage ≥3% of wall.
`classify_with` is 2.02% of process.
`commit_record` is 0.59% of process (H118 already rejected that apply cut).
Candidate install does not appear at ≥3%. No engine patch in this cell.

Do not retry H118. Do not retry H119 walk-overlap.
Do not retry H124 type/size or a larger read chunk.
Do not invent a first-pass classify skip.
Do not invent another cache-hit skip.

Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
