---
title: Share content metric resolution across views
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-159
  title: Share content metric resolution across views
  date: "2026-09-27"
  hypotheses:
    - H153
  subject:
    tree_label: metabrowser-clone
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 0eed491edf9b68dce5e8660d49692971e3f11bc5710312fab4806433ddacbe42
    tree_provenance: "Live checkout of https://github.com/jlevy/metabrowser.git at 091d404347823792a3d132c0100114f39d5a72b0 with local generated, installed, and untracked state; exact shape is fingerprinted but not reconstructible"
    tree_reconstructible: false
    tree_entries: 137085
    tree_directories: 9936
    tree_files: 127104
    tree_symlinks: 45
    tree_apparent_bytes: 1643250814
    tree_allocated_bytes: 1954811904
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
    control: release probe at 1ba06b19 with independent metric resolution
    candidate: release probe at d0902cfd with one-pass shared metric resolution
    control_binary:
      name: control
      sha256: 069e3423c0b6ea6a45f87d5da6338e2ec53080c10351ad29468624f3ef63d316
      size_bytes: 2900928
      args: []
    candidate_binary:
      name: candidate
      sha256: 98ec79d8f22078fa784c1a4889480fe1f106c6154ea1e50439adb53f6d3658d0
      size_bytes: 2900928
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /Volumes/spud-ext1/agent-scratch/fdu-perf-6h-20260927/runs/run-exp-159-h153-shared-metric-resolution-one-pass.json
  results:
    - job: content-query
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 38629306250.0
          candidate_median: 20636392917.0
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.063
          change_pct: -47.005
          ci95_low_pct: -47.49
          ci95_high_pct: -45.23
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 29907984395.5
          candidate_median: 12009100667.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.043
          change_pct: -59.941
          ci95_low_pct: -60.803
          ci95_high_pct: -58.594
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 53785726500.0
          candidate_median: 36263288000.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.059
          change_pct: -32.122
          ci95_low_pct: -33.597
          ci95_high_pct: -30.708
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 34902132500.0
          candidate_median: 17199856000.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.016
          change_pct: -50.747
          ci95_low_pct: -51.098
          ci95_high_pct: -50.451
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 18964767000.0
          candidate_median: 19134280000.0
          control_p95_over_median: 1.052
          candidate_p95_over_median: 1.094
          change_pct: 2.37
          ci95_low_pct: -1.738
          ci95_high_pct: 9.262
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 1320525824.0
          candidate_median: 1318068224.0
          control_p95_over_median: 1.001
          candidate_p95_over_median: 1.002
          change_pct: -0.169
          ci95_low_pct: -0.536
          ci95_high_pct: 0.183
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
    lines_changed: 396
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - multi-view metric summaries could be returned out of request order; the combined-versus-independent oracle and tests guard it
    notes: "266 insertions and 130 deletions in query_report.rs, including focused tests; no dependencies, unsafe code, public API, or persistent identity"
  verdict:
    decision: accepted
    primary_job: content-query
    primary_metric: wall_ns
    change_pct: -47.005
    reason: "one-pass shared metric resolution cut the 100-report probe wall 47.01% [45.23%, 47.49%] with exact report identity and non-inferior RSS/minor faults; keep the platform-neutral algorithmic cut"
    commit: d0902cfd
    kept: candidate
---
## What was predicted

H152 and the Astra review identified one bounded cross-platform follow-up to H138. H138
shares the `FileRow` walk across unfiltered Types, Families, Languages, and Documents
sections, but each section still called `ContentIndex::file` and `Index::classify` for
every regular file. H153 predicted that resolving those two path-keyed answers once per
file and feeding every requested metric accumulator would cut `content-query` wall by at
least 3% on deciding-scale `metabrowser-clone`.

The mechanism was constrained before implementation:

- Keep current path classification.
  Cached content detection is not the same answer.
- Apply only when two or more unfiltered metric views are requested.
  The filtered and single-view paths stay on their existing traversal.
- Add no public abstraction, persistent identity, dependency, or unsafe code.
- Require the exact multi-view report oracle, an unchanged subject fingerprint, a paired
  wall interval below zero, and non-inferior resources.

H152 added the missing report oracle in `1ba06b19`. It compares the combined report,
outside the timer, with independently constructed single-view reports, including section
order, rows, totals, and projection.
The current profiling-build component was 31.125 s for 100 reports on this tree.
Its whole-process 8 s sample was dominated by the untimed analysis setup (`read` plus
`open` 80.19%), so that profile is attribution evidence only, not a query-path
percentage. Direct inspection established the repeated four-way resolution that H153
removes.

## What was tried

The first candidate retained one resolved classification and content reference per file,
then replayed that vector into each metric section.
A fixed-N quiet attempt had only five usable observations per arm because the host
repeatedly crossed the 25% CPU boundary, but it showed wall −44.69% [−45.55%, −44.18%]
and component −57.78%. It also increased minor faults by 29.58% [29.18%, 29.86%], beyond
the +10% resource gate.
That shape was rejected and never committed.

The kept candidate performs one streaming pass instead.
For each regular file it resolves the current classification and admitted content
record, immediately applies them to the requested metric accumulators, and drops the
classification before advancing.
The finished summaries are then placed back in request order.
This preserves the resolution cut without retaining 127,104 owned classification
strings.

The existing combined-versus-independent view corpus and a focused one-pass test compare
every shared summary with the old independent path.
The full no-default-features core suite passed: 775 unit tests, one ignored manual test,
every integration group, and all doctests.

## What was measured

Subject: live `metabrowser-clone` at commit `091d4043`, 137,085 entries, 127,104 files,
9,936 directories, 1.53 GiB apparent, maximum depth 19. The checkout includes local
generated and installed state and is not reconstructible.
The harness verified its exact fingerprint before and after the run.

Job: `content-query`, which performs one fresh scan and line-analysis setup, then
constructs the unfiltered Types, Families, Languages, and Documents views 100 times.
It does not run every analyzer.
Three warmups, 12 timed pairs, interleaved, warm-steady cache.
Every sample ran the exact report oracle.
The cell was predeclared `uncontrolled`; instantaneous CPU busy was 17.58% initially and
17.11% finally, thermal pressure was normal, and no sample was invalid.
No RAM disk.

Component time covers the 100-report construction loop.
Whole-probe wall also includes the one-time scan and line-analysis setup, correctness
work outside the component timer, and teardown.
Neither number is a single end-to-end `--analyze all` measurement.

| Arm | Wall median | Component | User CPU | Peak RSS |
| --- | ---: | ---: | ---: | ---: |
| control `1ba06b19` | 38,629.3 ms | 29,908.0 ms | 34,902.1 ms | 1,259.4 MiB |
| candidate `d0902cfd` | 20,636.4 ms | 12,009.1 ms | 17,199.9 ms | 1,257.0 MiB |

Paired wall changed −47.01% [−47.49%, −45.23%]. Component changed −59.94%
[−60.80%, −58.59%], total CPU −32.12%, and user CPU −50.75%. Peak RSS changed −0.17%
[−0.54%, +0.18%], and minor faults −0.05% [−0.10%, +0.69%]: both are non-inferior.
System CPU was inconclusive at +2.37% [−1.74%, +9.26%], inside its +75% resource limit.

The adaptive qualification is inconclusive only on major faults.
Several runs recorded zero and several recorded about 100 in both arms, so the paired
bootstrap cannot establish the zero-delta non-regression rule.
No ordinary memory, CPU, answer, or subject-integrity gate regressed.

## What the determination said

**Accepted.** H153 clears the 3% wall gate by a wide margin and removes duplicated
algorithmic work rather than tuning a macOS constant.
The code is platform-neutral Rust and should transfer to Linux because it reduces the
same four content-map lookups and classifications per file there.
That transfer is a prediction, not evidence; a Linux replication remains useful, just as
H141 replicated H138’s shared-row win.

Do not apply the shared path to filtered reports: they already share the selection walk.
Do not replace current classification with cached detection.
Do not restore the rejected retained-resolution vector.
Single metric views keep their original one-pass path.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
