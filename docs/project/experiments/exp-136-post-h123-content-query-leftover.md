---
title: Post-H123 content-query leftover
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-136
  title: Post-H123 content-query leftover
  date: "2026-09-19"
  hypotheses:
    - H137
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
    control: HEAD release probe at afd0c919
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
    run_artifact: /tmp/fdu-realtree/results/run-exp-136-h137-content-query-leftover.json
  results:
    - job: content-query
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 40340480875.0
          candidate_median: 38623467583.0
          control_p95_over_median: 1.125
          candidate_p95_over_median: 1.14
          change_pct: -3.267
          ci95_low_pct: -14.417
          ci95_high_pct: 4.409
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 26781197833.0
          candidate_median: 27325757333.0
          control_p95_over_median: 1.238
          candidate_p95_over_median: 1.15
          change_pct: 2.268
          ci95_low_pct: -10.651
          ci95_high_pct: 6.809
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 49300953000.0
          candidate_median: 48949831500.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.075
          change_pct: -0.771
          ci95_low_pct: -2.076
          ci95_high_pct: 0.755
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 28030242500.0
          candidate_median: 28021942500.0
          control_p95_over_median: 1.025
          candidate_p95_over_median: 1.049
          change_pct: 0.325
          ci95_low_pct: -1.107
          ci95_high_pct: 1.316
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 21222213000.0
          candidate_median: 20906416000.0
          control_p95_over_median: 1.069
          candidate_p95_over_median: 1.052
          change_pct: -1.637
          ci95_low_pct: -4.669
          ci95_high_pct: 4.413
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 705937408.0
          candidate_median: 724680704.0
          control_p95_over_median: 1.049
          candidate_p95_over_median: 1.035
          change_pct: 2.131
          ci95_low_pct: -2.525
          ci95_high_pct: 7.474
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
    primary_job: content-query
    primary_metric: wall_ns
    change_pct: -3.267
    reason: content-query leftover is every_entry path-join FileRow walks (~278ms per four-view report); unfiltered Types/Families/Languages/Documents each walk independently; no engine patch
    commit: afd0c919
---
## What was predicted

H123 confirmed a metadata `query::report` is 1.7 ms on frameworks.
H134 exhausted Darwin `content-cache-hit` leftover.
H135 closed first-pass analyze (still file I/O). H136 closed first-run
`default-tree-first` (walk still the job).

This cell is a leftover profile on deciding-scale `content-query` (four content views,
100 iterations). Not a cache-hit skip.

The three named cache-hit / opened slices were inspected before measuring and did not
yield a discardable ≥3% mechanism:

- `insert_loaded_child` after H133: remaining work is `alloc` / `insert_child` /
  `intern_ext` / eager `merge_upward`. None of that is discardable on one-shot
  (`serving=None`) without breaking opened-root insert or totals.
- `rebuild_content_rollups`: already H115’s one O(files + dirs) bottom-up pass.
  A cheaper rebuild would be the same mechanism, not a new one.
  H118 rejected the same shape on first-pass.
- Opened-root journal clone / `read_dir`+`fstatat`: H127 already said no smallest
  fdu-core cut. `retain_commit` clones because the caller still holds the `Commit`. Not a
  `macos_bulk` port.

Named before measuring:

- Determination: content-view aggregation is or is not a userspace stage ≥3% of this
  job, and whether that stage is a skippable product-path cut.
- If no skippable ≥3% mechanism appears, do not compile a cut in this cell.
- Attachment: 12-pair same-source `content-query`, `FDU_COUNTERS` unset.
- Attribution: three counters-on hits plus a 20 s `/usr/bin/sample` on the profiling
  build.

Subject: frozen APFS clone of `metabrowser-clone`. Experiment id exp-136. H137. No
engine change.

Quiet start this tick refused at CPU busy 76.8% > 25.0%. Tried once; skipped.
Uncontrolled. Do not lower the 25% bar.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. Content digest `3be19a3e…`. The clone did not mutate.

Job: harness `content-query` (scan + analyze as setup; timer is 100 `query::report`
calls with Types, Families, Languages, Documents).
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Official quiet check 76.8% busy.
Pair initial 64.19%; final 54.35%. Thermal `normal`. The 25% bar was not lowered.
No RAM disk.

Same copied HEAD release probe both variants (`8765aa6f…` / 2,553,504 bytes).
0 invalid samples. Self-comparison only.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 40,340.5 ms | 26,781.2 ms | 673.2 MiB |
| candidate | 38,623.5 ms | 27,325.8 ms | 691.1 MiB |

Wall −3.27% [−14.42%, +4.41%]. Attachment only.
Interval includes zero.

## What `content-query` spends time on

Three `FDU_COUNTERS=1` hits (scan and analyze are setup):

| Hit | Component ms | Queries | ms / query | Analyzed |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 27,759 | 100 | 277.6 | 118,882 |
| 2 | 28,072 | 100 | 280.7 | 118,882 |
| 3 | 27,956 | 100 | 279.6 | 118,882 |

One four-view content report is about 280 ms on this tree.
H123’s metadata report was 1.7 ms on a different tree and a different view.

A 20 s `/usr/bin/sample` on the profiling build (13,968 main-thread samples) started
with the process. `report_in` is 7,810 samples (56% of process; the rest is analyze
setup). Under `build_section`, `every_entry` reconstructs every path and grows a
`FileRow` `Vec`.

`metric_summary` for unfiltered Types / Families / Languages / Documents calls
`every_entry` independently.
Four views are four full walks.
Filtered views already share one traversal (`walked` in `report_in`).

## What the determination said

Content-query leftover is `every_entry` on every unfiltered metric view, about 280 ms
per four-view report.
That is a named ≥3% userspace stage of this job.

It is not compiled in this cell.
Sharing one walk across unfiltered metric views is the leftover cut (the filtered path
already does that). Using content roll-ups without paths is a larger shape change.
Do not compile either here.

Do not invent a cache-hit skip.
Do not persist ignored bits.
Do not load a snapshot on `fdu PATH`. Do not retry H109 / H116 / H118 / H124. Do not
raise the README 200K files/s or 4M cached lines/s.
