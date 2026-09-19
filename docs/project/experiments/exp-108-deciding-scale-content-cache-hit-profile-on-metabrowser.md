---
title: Deciding-scale content-cache-hit profile on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-108
  title: Deciding-scale content-cache-hit profile on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H109
  subject:
    tree_label: metabrowser-clone
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: aaf1e17d63831a1fa0490ab426621144cf00916154bb137f662ba34092b7b05d
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
    control: same probe at a35a4cee
    candidate: same probe self-comparison
    control_binary:
      name: control
      sha256: ac298b3a0cf7d583c5e0f44deb5b2be3b69210100449bb2499c39f99b664af60
      size_bytes: 2487408
      args: []
    candidate_binary:
      name: candidate
      sha256: ac298b3a0cf7d583c5e0f44deb5b2be3b69210100449bb2499c39f99b664af60
      size_bytes: 2487408
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-108-h109-content-cache-hit-profile.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1218002208.5
          candidate_median: 1219939812.5
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.122
          change_pct: -0.294
          ci95_low_pct: -1.091
          ci95_high_pct: 1.054
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 916672521.0
          candidate_median: 906409458.5
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.174
          change_pct: -0.864
          ci95_low_pct: -1.942
          ci95_high_pct: 0.136
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1205580000.0
          candidate_median: 1201624500.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.022
          change_pct: -0.669
          ci95_low_pct: -1.211
          ci95_high_pct: 0.446
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 1100596500.0
          candidate_median: 1100834000.0
          control_p95_over_median: 1.016
          candidate_p95_over_median: 1.014
          change_pct: -0.239
          ci95_low_pct: -0.774
          ci95_high_pct: 0.191
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 103702000.0
          candidate_median: 101655000.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.101
          change_pct: 0.248
          ci95_low_pct: -12.565
          ci95_high_pct: 14.454
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 15086479.5
          candidate_median: 16285124.5
          control_p95_over_median: 1.498
          candidate_p95_over_median: 8.645
          change_pct: 35.926
          ci95_low_pct: -24.693
          ci95_high_pct: 55.583
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 396558336.0
          candidate_median: 398073856.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.02
          change_pct: 0.259
          ci95_low_pct: -1.145
          ci95_high_pct: 2.157
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
    notes: ""
  verdict:
    decision: baseline
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -0.294
    reason: "install_controls is 7.2 percent of the profile on 146k entries, down from 19 percent on 3k; Path rewrite is not justified"
    commit: a35a4cee
---
## What was predicted

H109 is a profile first, not a Path rewrite.

The screening figure (exp-104, cargo-registry, 3,077 entries) put
`Index::install_controls` → `reclassify_controlled_subtrees` at 19.43% of the
`content-cache-hit` profile and 25.5% of the engine, almost all
`ControlMatcher::is_ignored` → `std::path::compare_components`. A 3k tree overstated
this tier about 2× for H103. exp-104 also showed that an instruction-only Path change
does not move wall.

Named before measuring:

- Attribution: where a deciding-scale `content-cache-hit` spends time, especially that
  `install_controls` share versus 19%/26%.
- Attachment: wall and peak RSS from a 12-pair self-comparison of the same release
  probe, so the profile is not orphaned.
- The Path rewrite is justified only if that share stays large enough that a
  memory-access rewrite like H102 could move wall by at least 3%. Skip it if the share
  collapses.
- Do not land an instruction-only trim.

This is a baseline profile.
It does not accept or reject a code change.

## What was measured

Subject: nominated `metabrowser-clone` (live checkout, 145,931 entries / 133,597 files /
11,512 directories, max depth 19). Controls-bearing (`.gitignore` present; 28 control
sources shared on the instrumented hit).
Same shape as exp-106; the engine digest moved (`41a1e845…` → `aaf1e17d…`) and was
re-observed into the nominated-subjects document.
The tree did not mutate during the pair.

Job: harness `content-cache-hit` (`CachePolicy::Only`, basic content) after one
`content-seed` per variant into an isolated scratch snapshot.
Sequential processes; the harness interleaved the two same-binary arms.
3 warmups, 12 timed pairs.
`FDU_COUNTERS` unset on the claim-grade pair.

Quiet was attempted.
Instantaneous CPU busy was 29.0%, then 15.72% (gate would have passed), then 85% after
the smoke seed, then 26.61% and 39.4%. The pair ran as **uncontrolled**. Start busy
40.28%; final 34.53%. The 25% bar was not lowered.
No RAM disk.

Both arms are the same `perf_probe` at `a35a4cee`, sha256 `ac298b3a0cf7…`.
Self-comparison wall −0.29% [−1.09%, +1.05%]. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,218.0 ms | 916.7 ms | 378.2 MiB |
| candidate | 1,219.9 ms | 906.4 ms | 379.6 MiB |

Every timed sample was `source=content-cache` with 133,597 cache hits and 0 applied.
The smoke seed analysed 118,850 files in 9.10 s and wrote a 10 MiB snapshot plus a 34
MiB sidecar. Content digest
`3b8cfa7183353657e70de585bc1262bbbf29161a9c9422a4a390526e94346ef1`.

A later `FDU_COUNTERS=1` hit is attribution only.
Counters distorted the component (1,422 ms versus 917 ms claim-grade), as the playbook
says they can. The sampling profile used `--counters disabled --oracle disabled`.

## What the hit path spends time on

Instrumented hit (not the verdict): 0 directory opens, 0 stats, 0 file opens, 0 file
reads, 0 control files read, 28 control sources shared, 0 same-parent path comparisons
(that counter is mutation preflight, not `is_ignored`), 1,121,963 roll-up merges,
5,611,160 allocations / 3,661,542 reallocations, 800 MiB allocated, 1,394 syscalls.

Harness self-time (9,498 samples, 10 s): allocator 24.47%; path 11.70%; collections
12.76%; `fdu::content` 6.48%; `fdu::index` 7.77%; `fdu::snapshot` 1.83%; probe/oracle
0.33%. Named symbols include `load_content` 5.42%, `Path::hash` 4.43%,
`ContentIndex::merge_ancestors` 2.43%, `ControlMatcher::is_ignored` 1.09%.

Caller tree (`/usr/bin/sample`, 7,880 thread samples; `content_open` 7,618):

| Inclusive node | Samples | Share of profile | Share of engine |
| --- | ---: | ---: | ---: |
| `content_open` | 7,618 | 96.7% | 100% |
| `open_for_report` → `load_content` (lib.rs:595) | 4,612 | 58.5% | 60.5% |
| `open_for_report` → snapshot load (lib.rs:562) | 1,952 | 24.8% | 25.6% |
| `install_controls` (under snapshot parse) | 569 | 7.2% | 7.5% |

`compare_components` under `is_ignored` is about 1% of the profile (78 samples in the
largest `is_ignored` node).
The rest of the path layer is content-map `Path::hash` / SipHash and `Components`
iteration on sidecar restore, the H103 shape already refuted on wall.

## Judgment

The screening share collapsed: 19.43% / 25.5% on 3k entries became 7.2% / 7.5% on 146k.
Deleting all of `install_controls` would ceiling around 5% of wall (7.5% of a component
that is 75% of wall).
A Path-comparison rewrite of the `is_ignored` slice cannot clear 3% after exp-104.

H109 code is not justified.
No engine patch.

The hit path is sidecar restore: `load_content` → `apply_analysis` →
`ContentIndex::commit` / `merge_ancestors`, plus snapshot parse.
That is H83 / `fdu-jxhk` (layout usable without rebuilding per-record state), not a
control matcher rewrite.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
