---
title: Sidecar restore stage split on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-109
  title: Sidecar restore stage split on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H112
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
    control: current best at 98da0c83
    candidate: off-by-default sidecar restore phase timers
    control_binary:
      name: control
      sha256: da1608ec56131c15d4cccedbbb78fea1bf689c9c582893b03a0682c820a36f06
      size_bytes: 2487408
      args: []
    candidate_binary:
      name: candidate
      sha256: 2ca2e82d3ccb52c658b4e1cd4b9454f3723f2d5f93c25d0d17e31328021c6b48
      size_bytes: 2503920
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-109-sidecar-restore-stage-split.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1195452437.5
          candidate_median: 1210132958.5
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.029
          change_pct: 0.309
          ci95_low_pct: -0.814
          ci95_high_pct: 1.626
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 891200979.5
          candidate_median: 903382667.0
          control_p95_over_median: 1.024
          candidate_p95_over_median: 1.035
          change_pct: 0.957
          ci95_low_pct: -0.973
          ci95_high_pct: 2.616
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1183323000.0
          candidate_median: 1197105500.0
          control_p95_over_median: 1.03
          candidate_p95_over_median: 1.02
          change_pct: 0.182
          ci95_low_pct: -0.76
          ci95_high_pct: 1.536
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 1088649500.0
          candidate_median: 1094170500.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.007
          change_pct: 0.188
          ci95_low_pct: -0.275
          ci95_high_pct: 0.838
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 95265000.0
          candidate_median: 104014500.0
          control_p95_over_median: 1.3
          candidate_p95_over_median: 1.184
          change_pct: 0.191
          ci95_low_pct: -5.878
          ci95_high_pct: 9.247
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 11579042.0
          candidate_median: 9787791.5
          control_p95_over_median: 1.468
          candidate_p95_over_median: 2.228
          change_pct: -3.358
          ci95_low_pct: -30.283
          ci95_high_pct: 66.915
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 406421504.0
          candidate_median: 405250048.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.007
          change_pct: -0.083
          ci95_low_pct: -0.494
          ci95_high_pct: 0.079
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
    lines_changed: 89
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: off-by-default phase timers; no unsafe; kept after counters-off wall stayed non-inferior
  verdict:
    decision: baseline
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: 0.309
    reason: "apply dominates restore (timers 63 percent, sample 54 percent of load_content); parse is about 10 percent; wall non-inferior so timers stay"
    commit: 0ec489e7af94f2a647d1ed94ac55f214fac5e477
---
## What was predicted

H83 is not refuted for this architecture.
The cheap increments are dead: H102 landed the file-map order, H103 refuted Path::hash /
SipHash on wall, and exp-108 screened out an H109 Path rewrite.
A format rewrite (H78) and `fdu-jxhk` (EntryId roll-ups, one bottom-up pass) are not the
smallest next step.

H112 asks whether sidecar restore is one cost.
On deciding-scale `content-cache-hit`, one named stage among read, parse (integrity +
decode), candidate install (`analysis_candidates` + HashMap), and apply
(`apply_analysis` / `commit` / `merge_ancestors`) accounts for a majority of restore
time and a wall ceiling of at least 3%.

Named before measuring:

- Determination: a stage dominates if it is at least 50% of restore phase time and at
  least 3% of claim-grade wall.
- Attachment: 12-pair `content-cache-hit` of the current best versus off-by-default
  phase timers, `FDU_COUNTERS` unset.
- If none dominates, H83 as “sidecar restore is the win” is too coarse for one
  increment.
- The timers are not a speed claim.
  Keep them if counters-off wall is non-inferior; revert if they regress wall or fail
  tests.

## What was measured

Subject: nominated `metabrowser-clone` (live checkout, 145,931 entries / 133,597 files /
11,512 directories, max depth 19). Same shape as exp-108; the engine digest moved
(`aaf1e17d…` → `3fbfed48…`) and was re-observed into the nominated-subjects document.
The tree did not mutate during the pair.

Job: harness `content-cache-hit` after one `content-seed` per variant into an isolated
scratch snapshot. 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the claim-grade pair.

Quiet was attempted.
The start gate refused at 31.7% CPU busy.
The pair ran as **uncontrolled**. Initial busy 38.12%; final 88.35%. The 25% bar was not
lowered. No RAM disk.

Control is the release probe at `98da0c83`, sha256 `da1608ec…`. Candidate is the same
probe plus four off-by-default restore phase timers
(`content_sidecar_{read,parse,candidates,apply}_us`). Self-comparison wall +0.31%
[−0.81%, +1.63%]. 0 invalid samples.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,195.5 ms | 891.2 ms | 387.6 MiB |
| candidate | 1,210.1 ms | 903.4 ms | 386.5 MiB |

Every timed sample was `source=content-cache` with 133,597 cache hits and 0 applied.
Content digest `3b8cfa7183353657e70de585bc1262bbbf29161a9c9422a4a390526e94346ef1`, the
same digest exp-108 recorded.

A later `FDU_COUNTERS=1` hit is attribution only.
Counters distorted the component (1,076–1,206 ms versus 891 ms claim-grade), as the
playbook says they can.
The sampling profile used the profiling probe with counters off and `--no-oracle`.

## What restore spends time on

Three counters-on hits, median of the four phase timers:

| Phase | Median µs | Share of restore |
| --- | ---: | ---: |
| read | 6,118 | 0.8% |
| parse | 62,617 | 8.5% |
| candidates | 186,370 | 25.4% |
| apply | 464,240 | 63.3% |

Counters-off `/usr/bin/sample` of a 10 s `--repeat` hit, inclusive children of
`load_content` (4,707 samples under `open_for_report` at lib.rs:595):

| Inclusive node | Samples | Share of `load_content` |
| --- | ---: | ---: |
| `apply_analysis` | 2,521 | 53.6% |
| `analysis_candidates` | 950 | 20.2% |
| candidate-map hash | 404 | 8.6% |
| leftover (inlined parse / read / free) | 832 | 17.7% |

The two instruments agree on order: apply, then candidate install, then parse.
Apply is at least 50% of restore either way.
Its wall ceiling is well above 3% (about a quarter of claim-grade wall if the whole
apply node disappeared).
Parse is about 10% of restore.
A format-speed or CRC tweak cannot clear the bar from here.

`open_for_report` at lib.rs:601 then walks `analysis_candidates` a second time to
compare `hits` to `len()`. That node is 971 samples (12.6% of `content_open`) and is not
inside `load_content`. It is the smallest leftover the split named, registered as H113.

## Judgment

H112 is confirmed: apply dominates sidecar restore.
H83 remains open, scoped to rebuilding per-record state on apply / commit / merge, not
to parse. The phase timers stay.
They did not move counters-off wall.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
