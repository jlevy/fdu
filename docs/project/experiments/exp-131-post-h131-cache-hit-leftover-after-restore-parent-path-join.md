---
title: Post-H131 cache-hit leftover after restore parent-path join
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-131
  title: Post-H131 cache-hit leftover after restore parent-path join
  date: "2026-09-19"
  hypotheses:
    - H132
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
    candidate: same-source rebuild of HEAD
    control_binary:
      name: control
      sha256: 84618edba302e624b423a8afa9b1f2f6d103fefda2d4d7e121360e5435054edf
      size_bytes: 2553504
      args: []
    candidate_binary:
      name: candidate
      sha256: 7136e576a6a2369a0fb153dc1c6657971c65f23faab4e9840c7c0375ec24ffee
      size_bytes: 2553504
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-131-h132-post-h131-leftover.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 825221667.0
          candidate_median: 825255770.5
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.025
          change_pct: -0.021
          ci95_low_pct: -0.668
          ci95_high_pct: 0.7
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 535807062.0
          candidate_median: 535572250.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.02
          change_pct: 0.047
          ci95_low_pct: -0.751
          ci95_high_pct: 1.458
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 818329000.0
          candidate_median: 815735500.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.023
          change_pct: -0.003
          ci95_low_pct: -0.772
          ci95_high_pct: 0.784
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 757159500.0
          candidate_median: 757818000.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.004
          change_pct: -0.013
          ci95_low_pct: -0.377
          ci95_high_pct: 0.407
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 59087500.0
          candidate_median: 59572000.0
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.214
          change_pct: -1.75
          ci95_low_pct: -5.693
          ci95_high_pct: 7.85
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 7170833.0
          candidate_median: 7933646.0
          control_p95_over_median: 1.214
          candidate_p95_over_median: 3.399
          change_pct: 14.175
          ci95_low_pct: -1.723
          ci95_high_pct: 37.22
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 312049664.0
          candidate_median: 311386112.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.005
          change_pct: -0.126
          ci95_low_pct: -0.521
          ci95_high_pct: 0.337
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -0.021
    reason: restore-walk path_of gone after H131; leftover snapshot path_of 9.89 percent of content_open discarded on one-shot serving=None; no engine patch
    commit: "69704206"
---
## What was predicted

H131 joined the parent path the restore DFS already holds, instead of `path_of` per
file. H130 had sampled `path_of` at 11.85% of `content_open`.

This cell is a leftover profile, not a cut.

Named before measuring:

- Determination: restore-walk `path_of` is under 3% of `content_open` (or absent), and
  the named remaining leftover is or is not a userspace stage ≥3% that is not already
  rejected.
- If restore-walk `path_of` is gone and no skippable ≥3% mechanism appears, do not
  compile a cut in this cell.
- Attachment: 12-pair same-source `content-cache-hit`, `FDU_COUNTERS` unset.
- Attribution: 20 s `/usr/bin/sample` on the profiling build plus counters-on hits.

Subject: frozen APFS clone of `metabrowser-clone`. Experiment id exp-131. H132. No
engine change.

A pre-pair busy check read 24.38%. `PERF_HOST_REGIME=quiet` then refused.
Uncontrolled. Do not lower the 25% bar.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Pair initial 26.27%; final 26.67%. Thermal `normal`. The 25%
bar was not lowered.
No RAM disk.

Control is the H131 release probe (`84618edb…` / engine `7840ce9b`). Candidate is a
same-source rebuild of HEAD (`7136e576…`). 0 invalid samples.
Self-comparison only.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 825.2 ms | 535.8 ms | 297.6 MiB |
| candidate | 825.3 ms | 535.6 ms | 297.0 MiB |

Wall −0.02% [−0.67%, +0.70%]. Attachment only.

Every timed sample was `source=content-cache` with 133,708 cache hits and 0 applied.
Content digest `3be19a3e…`.

## What restore and `content_open` spend time on

Three `FDU_COUNTERS=1` hits after H131 (apply excludes decode):

| Phase | Hit 1 µs | Hit 2 µs | Hit 3 µs |
| --- | ---: | ---: | ---: |
| read | 5,981 | 5,957 | 5,792 |
| parse | 25,717 | 24,637 | 24,182 |
| candidates | 59,091 | 61,198 | 60,155 |
| apply | 110,163 | 110,702 | 105,744 |

Candidates dropped from ~98 ms (H130) to ~60 ms.
Apply stayed ~110 ms.

A 20-second `/usr/bin/sample` on the profiling build (`--repeat 25`, counters and oracle
off). Main-thread `content_open` 11,343 samples:

| Inclusive node under `content_open` | Samples | Share of `content_open` |
| --- | ---: | ---: |
| `load_content` (`open_for_report` lib.rs:595) | 6,490 | 57.22% |
| snapshot load (`open_for_report` lib.rs:562) | 4,852 | 42.78% |
| completeness compare | 0 | 0% |
| restore-walk `path_of` (under `load_content`) | 0 | 0% |
| snapshot `path_of` (`insert_loaded_child`) | 1,122 | 9.89% |
| `Path::__join` under `load_content` | 701 | 6.18% |
| `commit_record` | 1,434 | 12.64% |
| `rebuild_content_rollups` | 1,801 | 15.88% |
| `reclassify_controlled_subtrees` | 1,210 | 10.67% |

Restore-walk `path_of` is gone.
The remaining `path_of` is entirely under snapshot `insert_loaded_child`
(`index.rs:4935`). One-shot snapshot load constructs the index with `serving = None`, so
`insert_serving_entry` returns without using that path.
`Path::__join` under `load_content` is the H131 leftover construction, not a skip.

`commit_record` and `rebuild_content_rollups` remain on the restore path (H116 / H103
shapes). `reclassify_controlled_subtrees` sits under snapshot `install_controls` (H109),
not this restore increment.

## What the determination said

Restore-walk `path_of` is gone.
The remaining leftover on `content_open` is snapshot `path_of` (9.89%), discarded
because serving indexes are off on one-shot load, plus already-rejected restore stages
and H109 reclassify.
No engine patch in this cell.

Skipping `path_of` when `serving` is `None` is a candidate mechanism; it is not compiled
here. Joining a parent path on snapshot insert would keep the same discarded work.

Do not retry H116. Do not retry H131. Do not mint a snapshot-parse cut.
Do not retry an H109 Path rewrite.
Do not retry file-count.

Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
