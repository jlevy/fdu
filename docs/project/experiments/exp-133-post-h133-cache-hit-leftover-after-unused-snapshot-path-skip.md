---
title: Post-H133 cache-hit leftover after unused snapshot path skip
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-133
  title: Post-H133 cache-hit leftover after unused snapshot path skip
  date: "2026-09-19"
  hypotheses:
    - H134
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
    control: H133 release probe at 143a1c73
    candidate: same-source rebuild of HEAD
    control_binary:
      name: control
      sha256: 15cb7078d2e54f83ec351628571d0b59e2a0c31f6bfaedc8e23905a01008e0af
      size_bytes: 2553504
      args: []
    candidate_binary:
      name: candidate
      sha256: 938d33fb507bd52f6bf94e6db6bd449dc91f9f873c9d8b7b91fed9a55aeaf9d4
      size_bytes: 2553504
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-133-h134-post-h133-leftover.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 775253187.5
          candidate_median: 774694292.0
          control_p95_over_median: 1.011
          candidate_p95_over_median: 1.005
          change_pct: -0.372
          ci95_low_pct: -0.818
          ci95_high_pct: 0.378
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 483275625.0
          candidate_median: 484428437.5
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.005
          change_pct: -0.094
          ci95_low_pct: -0.939
          ci95_high_pct: 0.945
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 764835500.0
          candidate_median: 766258500.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.007
          change_pct: -0.452
          ci95_low_pct: -0.79
          ci95_high_pct: 0.839
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 703703500.0
          candidate_median: 704211500.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.003
          change_pct: 0.023
          ci95_low_pct: -0.423
          ci95_high_pct: 0.367
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 63008000.0
          candidate_median: 60839000.0
          control_p95_over_median: 1.055
          candidate_p95_over_median: 1.068
          change_pct: -2.842
          ci95_low_pct: -8.236
          ci95_high_pct: 3.851
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 8031209.0
          candidate_median: 7238667.0
          control_p95_over_median: 2.385
          candidate_p95_over_median: 1.476
          change_pct: -10.747
          ci95_low_pct: -28.753
          ci95_high_pct: 5.809
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 311205888.0
          candidate_median: 312377344.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.006
          change_pct: 0.185
          ci95_low_pct: 0.034
          ci95_high_pct: 0.718
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: noninferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
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
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -0.372
    reason: snapshot path_of gone after H133; remaining leftover is already-rejected or already-landed restore and control stages; no engine patch
    commit: "f1e9ef9c"
---
## What was predicted

H133 skips `path_of` in `insert_loaded_child` when serving is off.
H132 had sampled that unused snapshot `path_of` at 9.89% of `content_open`.

This cell is a leftover profile, not a cut.

Named before measuring:

- Determination: snapshot `path_of` is under 3% of `content_open` (or absent), and the
  named remaining leftover is or is not a userspace stage ≥3% that is not already
  rejected.
- If snapshot `path_of` is gone and no skippable ≥3% mechanism appears, do not compile a
  cut in this cell.
- Attachment: 12-pair same-source `content-cache-hit`, `FDU_COUNTERS` unset.
- Attribution: 20 s `/usr/bin/sample` on the profiling build plus counters-on hits.

Subject: frozen APFS clone of `metabrowser-clone`. Experiment id exp-133. H134. No
engine change.

Quiet start this tick refused at CPU busy 28.07% > 25.0%. Tried once; skipped.
Uncontrolled. Do not lower the 25% bar.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Pair initial 29.36%; final 40.04%. Thermal `normal`. The 25%
bar was not lowered.
No RAM disk.

Control is the H133 release probe (`15cb7078…` / engine `143a1c73`). Candidate is a
same-source rebuild of HEAD (`938d33fb…`). 0 invalid samples.
Self-comparison only.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 775.3 ms | 483.3 ms | 296.8 MiB |
| candidate | 774.7 ms | 484.4 ms | 297.9 MiB |

Wall −0.37% [−0.82%, +0.38%]. Attachment only.

Every timed sample was `source=content-cache` with 133,708 cache hits and 0 applied.
Content digest `3be19a3e…`.

## What restore and `content_open` spend time on

Three `FDU_COUNTERS=1` hits after H133 (apply excludes decode):

| Phase | Hit 1 µs | Hit 2 µs | Hit 3 µs |
| --- | ---: | ---: | ---: |
| read | 19,289 | 5,915 | 6,352 |
| parse | 27,623 | 25,137 | 24,342 |
| candidates | 65,587 | 60,910 | 60,716 |
| apply | 127,765 | 115,459 | 109,255 |

Candidates stayed ~61 ms (H132 ~60 ms).
Apply stayed ~110–128 ms.

A 20-second `/usr/bin/sample` on the profiling build (`--repeat 25`, counters and oracle
off). Main-thread `content_open` 9,919 samples:

| Inclusive node under `content_open` | Samples | Share of `content_open` |
| --- | ---: | ---: |
| `load_content` (`open_for_report` lib.rs:595) | 6,156 | 62.06% |
| snapshot load (`open_for_report` lib.rs:562) | 3,763 | 37.94% |
| completeness compare | 0 | 0% |
| restore-walk `path_of` (under `load_content`) | 0 | 0% |
| snapshot `path_of` (`insert_loaded_child`) | 0 | 0% |
| `path_of` (all) | 0 | 0% |
| `Index::classify` / `classify_with` | 0 | 0% |
| `rebuild_content_rollups` | 1,857 | 18.72% |
| `insert_loaded_child` | 1,620 | 16.33% |
| `install_controls` | 1,450 | 14.62% |
| `reclassify_controlled_subtrees` | 1,197 | 12.07% |
| `commit_record` | 1,191 | 12.01% |
| `Path::__join` under `load_content` | 715 | 7.21% |
| HashMap under `load_content` | 463 | 4.67% |

Snapshot `path_of` is gone.
`insert_loaded_child` remaining cost is eager `merge_upward`, `insert_child`,
`intern_ext`, and alloc.
The function already records that deferring `merge_upward` would buy little and would
leave the index structurally complete but numerically wrong mid-load.
`Path::__join` under `load_content` is the H131 leftover construction, not a skip.
Joining a parent path on snapshot insert would add work the serving skip already
removed.

`commit_record` and `rebuild_content_rollups` remain on the restore path (H116 / H103
shapes; H115 already landed the rebuild).
`reclassify_controlled_subtrees` sits under snapshot `install_controls` (H109), not this
restore increment.

## What the determination said

Snapshot `path_of` is gone.
The remaining leftover on `content_open` is already-landed restore work (H115 rebuild,
H131 join construction, snapshot insert) and already-rejected stages (H116 HashMap, H109
reclassify, H103-shaped `commit_record`). No engine patch in this cell.

Do not retry H116. Do not retry H131. Do not retry H133. Do not mint a snapshot-parse
cut. Do not retry an H109 Path rewrite.
Do not retry file-count.
Do not defer snapshot `merge_upward`.

Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
