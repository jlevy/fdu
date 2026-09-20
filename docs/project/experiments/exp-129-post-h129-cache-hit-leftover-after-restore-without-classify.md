---
title: Post-H129 cache-hit leftover after restore-without-classify
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-129
  title: Post-H129 cache-hit leftover after restore-without-classify
  date: "2026-09-19"
  hypotheses:
    - H130
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
    candidate: same-source rebuild of HEAD
    control_binary:
      name: control
      sha256: a4de4b7280af4e47e5b654ffb009133e4a34f5569178201b35a16b32dd7105c7
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: e8098aff17a06d4423309ac7b123bafbc0ddf4ca7d8bda6494cbe939d4b0ddf3
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-129-h130-post-h129-leftover.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 862226375.5
          candidate_median: 859697750.0
          control_p95_over_median: 1.308
          candidate_p95_over_median: 1.31
          change_pct: -0.209
          ci95_low_pct: -1.766
          ci95_high_pct: 21.324
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 571484229.5
          candidate_median: 570640916.5
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.355
          change_pct: -0.191
          ci95_low_pct: -1.395
          ci95_high_pct: 13.995
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 853446000.0
          candidate_median: 850101000.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.029
          change_pct: -0.184
          ci95_low_pct: -1.774
          ci95_high_pct: 1.514
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 792830500.0
          candidate_median: 789776500.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.02
          change_pct: -0.4
          ci95_low_pct: -0.948
          ci95_high_pct: 1.022
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 62024000.0
          candidate_median: 61804000.0
          control_p95_over_median: 1.174
          candidate_p95_over_median: 1.114
          change_pct: -1.938
          ci95_low_pct: -11.504
          ci95_high_pct: 7.796
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 10791250.5
          candidate_median: 10555458.0
          control_p95_over_median: 17.882
          candidate_p95_over_median: 23.615
          change_pct: 29.041
          ci95_low_pct: -31.608
          ci95_high_pct: 1917.933
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 310648832.0
          candidate_median: 313057280.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.001
          change_pct: 0.721
          ci95_low_pct: 0.163
          ci95_high_pct: 1.139
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: inconclusive
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
    change_pct: -0.209
    reason: restore classify gone after H129; path_of 11.85 percent of content_open; no engine patch
    commit: 6e101ace
---
## What was predicted

H129 omitted classify on cache-only restore (walk and apply self-check).
H126 left the first candidates walk at 15.7% of `content_open`; classify was about half
of that.

This cell is a leftover profile, not a cut.

Named before measuring:

- Determination: `Index::classify` / `classify_with` on the restore path is under 3% of
  `content_open` (or absent), and the named remaining leftover is or is not a userspace
  stage ≥3% that is not already rejected.
- If classify is gone and no skippable ≥3% mechanism appears, do not compile a cut in
  this cell. `path_of` is a slice until a skip that keeps completeness can be named.
- Attachment: 12-pair same-source `content-cache-hit`, `FDU_COUNTERS` unset.
- Attribution: exp-128 candidate counters-on hits and a 20 s `/usr/bin/sample`.

Subject: frozen APFS clone of `metabrowser-clone`. Experiment id exp-129. H130. No
engine change.

Quiet this tick refused at 34.97%. Tried once; skip.
Uncontrolled. Do not lower the 25% bar.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Quiet start gate 34.97%; pair initial 66.8% / final 66.6%. The
25% bar was not lowered.
No RAM disk.

Control is the H129 release probe (`a4de4b72…`). Candidate is a same-source rebuild of
HEAD (`e8098aff…`). 0 invalid samples.
Self-comparison only.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 862.2 ms | 571.5 ms | 296.3 MiB |
| candidate | 859.7 ms | 570.6 ms | 298.6 MiB |

Wall −0.21% [−1.77%, +21.32%]. Attachment only.

Every timed sample was `source=content-cache` with 133,708 cache hits and 0 applied.
Content digest `3be19a3e…`.

## What restore and `content_open` spend time on

Three `FDU_COUNTERS=1` hits from the H129 candidate (apply excludes decode):

| Phase | Hit 1 µs | Hit 2 µs | Hit 3 µs |
| --- | ---: | ---: | ---: |
| read | 5,610 | 5,347 | 5,588 |
| parse | 24,278 | 24,356 | 24,854 |
| candidates | 99,059 | 97,378 | 98,200 |
| apply | 107,462 | 104,673 | 105,599 |

Candidates dropped from ~164 ms (H125/H126) to ~98 ms.
Apply dropped from ~135 ms to ~106 ms.
Parse unchanged.

A 20-second `/usr/bin/sample` on the profiling build (`--repeat 25`, counters and oracle
off). Main-thread `content_open` 12,030 samples:

| Inclusive node under `content_open` | Samples | Share of `content_open` |
| --- | ---: | ---: |
| `load_content` (`open_for_report` lib.rs:595) | 6,824 | 56.7% |
| snapshot load (`open_for_report` lib.rs:562) | 5,206 | 43.3% |
| completeness compare (`open_for_report` lib.rs:594) | 0 | 0% |
| `Index::classify` / `classify_with` | 0 | 0% |
| `path_of` (direct child of `load_content`) | 1,425 | 11.85% |
| `commit_record` (direct child of `load_content`) | 1,325 | 11.01% |
| `rebuild_content_rollups` | 518 | 4.31% |

exp-125 sampled the first `analysis_candidates` walk at 15.7% and classify at about
13.3% of `content_open` (walk 5.94% + apply guard 7.36%). Classify is gone.
`path_of` rose as a share because classify left.

`classify.rs` appears at 0.90% (`derive_ext` during snapshot insert), not restore
classify. `reclassify_controlled_subtrees` / `control.rs` sit under snapshot load
(H107/H109), not this restore increment.

## What the determination said

Restore classify is gone.
The remaining leftover on the restore walk is `path_of` (ancestor walk per file, 11.85%
of `content_open`) plus HashMap insert (H116, rejected on wall) plus apply
`commit_record`. Snapshot load rose as a share because restore got cheaper (H78/H92; not
a snapshot load on `fdu PATH`). No engine patch in this cell.

`path_of` is still a slice until a later cell names a skip that keeps completeness.
Threading the parent path already in the DFS (join child name, instead of walking
ancestors per file) is a candidate mechanism; it is not compiled here.

Do not retry H116. Do not mint a snapshot-parse cut.
Do not retry file-count.

Do not raise the README 200K files/s or 4M cached lines/s.
