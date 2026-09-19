---
title: Cache-only completeness by file count on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-110
  title: Cache-only completeness by file count on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H113
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
    control: current HEAD with H112 timers at 4998ee73
    candidate: file-count completeness instead of walking analysis_candidates
    control_binary:
      name: control
      sha256: bfa8cd518c6649dbfeb4129b00d597434f9ba71f022b61543ea335bd2e33a4de
      size_bytes: 2503920
      args: []
    candidate_binary:
      name: candidate
      sha256: 0256297c239a765bf7e6f751edc26badcfff1eb812d3c22b3efe14ac39ab98dd
      size_bytes: 2503920
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-110-cache-only-completeness-count.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1246300833.5
          candidate_median: 1156766000.0
          control_p95_over_median: 1.074
          candidate_p95_over_median: 1.198
          change_pct: -7.594
          ci95_low_pct: -10.756
          ci95_high_pct: 2.239
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 937580166.5
          candidate_median: 836919854.5
          control_p95_over_median: 1.084
          candidate_p95_over_median: 1.054
          change_pct: -13.068
          ci95_low_pct: -14.3
          ci95_high_pct: -7.893
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 1222107500.0
          candidate_median: 1114001000.0
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.047
          change_pct: -9.858
          ci95_low_pct: -11.251
          ci95_high_pct: -6.754
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 1108760000.0
          candidate_median: 1002324000.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.034
          change_pct: -9.791
          ci95_low_pct: -10.192
          ci95_high_pct: -8.04
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 113366000.0
          candidate_median: 110005000.0
          control_p95_over_median: 1.202
          candidate_p95_over_median: 1.08
          change_pct: -12.468
          ci95_low_pct: -16.893
          ci95_high_pct: -0.643
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        blocked_ns:
          control_median: 23990750.0
          candidate_median: 46140083.0
          control_p95_over_median: 2.899
          candidate_p95_over_median: 5.591
          change_pct: 77.812
          ci95_low_pct: -3.598
          ci95_high_pct: 285.774
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 398090240.0
          candidate_median: 394190848.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.002
          change_pct: -1.062
          ci95_low_pct: -2.81
          ci95_high_pct: -0.575
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
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
    notes: file-count completeness shortcut measured and reverted; incomplete-sidecar fail-closed test kept
  verdict:
    decision: rejected
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -7.594
    reason: wall -7.59 percent but interval includes zero; shortcut reverted
    commit: null
---
## What was predicted

H113: after a cache-only sidecar restore, `open_for_report` walks `analysis_candidates`
again only to compare `hits` to `len()`. exp-109 sampled that walk at 12.6% of
`content_open` (about 9% of wall).
Completeness can use a count already known from restore, or the index file count when
every regular file is a candidate, and still refuse an incomplete sidecar.

Named before measuring:

- Metric: `content-cache-hit` wall on deciding-scale `metabrowser-clone`.
- Direction: down.
- Accept: median at least 3% faster and the 95% paired interval entirely below zero;
  content digest identical; incomplete sidecar still refused.
- Control: this branch HEAD, including the H112 off-by-default restore phase timers.
  Claim-grade pair with `FDU_COUNTERS` unset.
- The smallest fdu-core change: `Index::analysis_candidate_count` returns the root
  regular-file total, which is what `analysis_candidates` already uses as its capacity,
  and the cache-only check compares `hits` to that instead of walking.

H81 is the precedent if the median clears 3% and the interval includes zero: revert.

## What was measured

Subject: nominated `metabrowser-clone` (live checkout, 145,931 entries / 133,597 files /
11,512 directories, max depth 19). Same shape and engine digest as exp-109
(`3fbfed48…`). The tree did not mutate during the pair.

Job: harness `content-cache-hit` after one `content-seed` per variant into an isolated
scratch snapshot. 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 27.8% CPU busy.
The pair ran as **uncontrolled**. Initial busy 40.0%; final 63.65%. The 25% bar was not
lowered. No RAM disk.

Control is the release probe at `4998ee73`, sha256 `bfa8cd51…`. Candidate is that probe
plus the file-count completeness check, sha256 `0256297c…`. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,246.3 ms | 937.6 ms | 379.6 MiB |
| candidate | 1,156.8 ms | 836.9 ms | 375.9 MiB |

Every timed sample was `source=content-cache` with 133,597 cache hits and 0 applied.
Content digest `3b8cfa7183353657e70de585bc1262bbbf29161a9c9422a4a390526e94346ef1`, the
same digest exp-108 and exp-109 recorded.

Pairs 6–8 spiked (control 1,991 ms, candidate 1,741 ms).
That is host noise on an uncontrolled cell, not a second mechanism.

## What the accept rule said

Wall −7.59% [−10.76%, +2.24%]. REJECT. The median is past 3% and near the 9% ceiling,
but the interval includes zero.

Component −13.07% [−14.30%, −7.89%] and user CPU −9.79% [−10.19%, −8.04%] both exclude
zero. Those are mechanism, not the pre-registered accept metric.
A post-hoc switch to component is never an accept.

The incomplete-sidecar test added for the contract still fails closed after the revert:
a metadata-only pass that widens the snapshot without rewriting the sidecar is refused
under cache-only analysis.

## Judgment

H113 is rejected on the wall rule.
The second `analysis_candidates` walk is real work, and the file-count shortcut removes
it, but this cell cannot claim the wall.
The shortcut is reverted.
Do not retry it on another uncontrolled cell.
A quiet confirmatory cell would be a new experiment, not a top-up of this one.

H83 remains open, scoped to apply / commit / `merge_ancestors`. That is next.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
