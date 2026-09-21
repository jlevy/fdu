---
title: Share one every_entry across unfiltered metric views
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-137
  title: Share one every_entry across unfiltered metric views
  date: "2026-09-19"
  hypotheses:
    - H138
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
    control: HEAD release probe at c382568c
    candidate: share one every_entry for unfiltered entry-row views
    control_binary:
      name: control
      sha256: 8765aa6f198cacf94027411ea9151125b58af2ac4458c9f45b782ec1bfc71124
      size_bytes: 2553504
      args: []
    candidate_binary:
      name: candidate
      sha256: 79668087f5212490d7917e3e0ec5bfb1641bfdd91f2466c4a1ef943185e2331c
      size_bytes: 2553504
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-137-h138-share-every-entry.json
  results:
    - job: content-query
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 38234226104.0
          candidate_median: 31475241520.5
          control_p95_over_median: 1.299
          candidate_p95_over_median: 1.178
          change_pct: -18.764
          ci95_low_pct: -22.865
          ci95_high_pct: -13.688
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 28291062208.0
          candidate_median: 20420605208.5
          control_p95_over_median: 1.309
          candidate_p95_over_median: 1.214
          change_pct: -24.606
          ci95_low_pct: -29.815
          ci95_high_pct: -20.558
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 44137499500.0
          candidate_median: 39095040000.0
          control_p95_over_median: 1.129
          candidate_p95_over_median: 1.062
          change_pct: -10.792
          ci95_low_pct: -15.906
          ci95_high_pct: -7.039
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 27929140500.0
          candidate_median: 23077906000.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.056
          change_pct: -17.173
          ci95_low_pct: -19.103
          ci95_high_pct: -16.576
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 14761619000.0
          candidate_median: 16227943000.0
          control_p95_over_median: 1.357
          candidate_p95_over_median: 1.146
          change_pct: -1.997
          ci95_low_pct: -9.37
          ci95_high_pct: 16.492
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 786964480.0
          candidate_median: 795860992.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.01
          change_pct: 1.031
          ci95_low_pct: 0.418
          ci95_high_pct: 1.608
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
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 68
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: share one every_entry walk; Summary/Tree/Extensions keep unfiltered roll-ups
  verdict:
    decision: accepted
    primary_job: content-query
    primary_metric: wall_ns
    change_pct: -18.764
    reason: "content-query wall -18.76% [-22.86%, -13.69%] on frozen metabrowser-clone; component -24.61%; engine kept; report identity is from crate tests, not the benchmark digest"
    commit: a5c98d59
---
## What was predicted

H137 leftover named the cut: unfiltered Types / Families / Languages / Documents each
call `every_entry` independently.
Filtered views already share one traversal.

H138 is that share. Smallest patch: one `FileRow` walk for every unfiltered view that
needs entry rows. Summary / Tree / Extensions keep roll-ups and must not see a partial
`Walked`.

Predicted: `content-query` wall down at least 3% with the interval below zero on frozen
`metabrowser-clone`. Report identity unchanged.
Content digest unchanged (read-only).

Quiet start this tick refused at CPU busy 93.2% > 25.0%. Tried once; skipped.
Uncontrolled. Do not lower the 25% bar.

## What was changed

`report_in` computes `every_entry` once when the selection is unfiltered and more than
one view needs entry rows.
A single row-consuming view owns its walk.
`metric_summary` and `file_rows` take that shared slice when it exists.

No cache change. No format change.
No snapshot load on `fdu PATH`.

The first draft of this section said “any view” and predated the R1 single-view
ownership fix. The measured candidate already shared on two or more consumers.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-query`. 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Labeled **uncontrolled**. Official quiet check 93.2% busy.
Pair initial 69.09%; final 70.06%. Thermal `normal`. The 25% bar was not lowered.
No RAM disk.

Control: leftover HEAD probe (`8765aa6f…` / 2,553,504 bytes).
Candidate: H138 probe (`79668087…` / 2,553,504 bytes).
0 invalid samples. The benchmark oracle checks that the retained index and content facts
remain unchanged; it does not digest the reports produced inside the timed loop.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 38,234.2 ms | 28,291.1 ms | 750.5 MiB |
| candidate | 31,475.2 ms | 20,420.6 ms | 759.0 MiB |

Wall −18.76% [−22.86%, −13.69%]. Component −24.61% [−29.82%, −20.56%]. User CPU −17.17%
[−19.10%, −16.58%]. Peak RSS +1.03% [+0.42%, +1.61%] non-inferior.
Minor faults −63.01%. Interval on wall excludes zero and is entirely below −3%.

## What the determination said

Sharing one unfiltered `every_entry` walk is a real ≥3% wall cut on `content-query`.
Focused [report tests](../../../crates/fdu-core/src/query/query_report.rs) establish the
output semantics with independent expected totals and rows, then compare combined and
individual views. The benchmark digest establishes only that retained facts are
unchanged. A general report-digest harness remains tracked on `fdu-2moo`. Engine kept.

Do not invent a cache-hit skip.
Do not persist ignored bits.
Do not load a snapshot on `fdu PATH`. Do not raise the README 200K files/s or 4M cached
lines/s.

## Errata (2026-09-21)

The verdict reason said “report identity unchanged”.
The benchmark digested retained index and content facts, not the reports produced inside
the timed loop. Report identity is established by crate tests.
Timing claims are unchanged.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
