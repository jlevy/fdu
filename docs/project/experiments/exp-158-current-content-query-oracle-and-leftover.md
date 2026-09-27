---
title: Current content-query oracle and leftover
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-158
  title: Current content-query oracle and leftover
  date: "2026-09-27"
  hypotheses:
    - H152
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
    control: release probe at 1ba06b19
    candidate: byte-identical release probe at 1ba06b19
    control_binary:
      name: control
      sha256: 069e3423c0b6ea6a45f87d5da6338e2ec53080c10351ad29468624f3ef63d316
      size_bytes: 2900928
      args: []
    candidate_binary:
      name: candidate
      sha256: 069e3423c0b6ea6a45f87d5da6338e2ec53080c10351ad29468624f3ef63d316
      size_bytes: 2900928
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /Volumes/spud-ext1/agent-scratch/fdu-perf-6h-20260927/runs/run-exp-158-h152-current-content-query.json
  results:
    - job: content-query
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 37903863792.0
          candidate_median: 38337176562.5
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.569
          change_pct: 1.022
          ci95_low_pct: 0.049
          ci95_high_pct: 15.625
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 29518429854.5
          candidate_median: 29615735062.5
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.59
          change_pct: 1.027
          ci95_low_pct: 0.028
          ci95_high_pct: 15.543
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 56169647000.0
          candidate_median: 56371856500.0
          control_p95_over_median: 1.202
          candidate_p95_over_median: 1.23
          change_pct: 1.158
          ci95_low_pct: -5.411
          ci95_high_pct: 3.203
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 34979544000.0
          candidate_median: 35102900000.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.041
          change_pct: 0.593
          ci95_low_pct: 0.027
          ci95_high_pct: 1.063
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 21123854500.0
          candidate_median: 20036394000.0
          control_p95_over_median: 1.54
          candidate_p95_over_median: 1.704
          change_pct: -3.004
          ci95_low_pct: -14.654
          ci95_high_pct: 9.551
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 1318510592.0
          candidate_median: 1317208064.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.003
          change_pct: -0.066
          ci95_low_pct: -0.503
          ci95_high_pct: 0.255
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
    lines_changed: 80
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - the outside-timer differential oracle depends on independent single-view report construction and adds wall work outside the component timer
    notes: "78 insertions and 2 deletions in perf_probe.rs, including a mutation test; no engine behavior, dependency, unsafe code, or public API change"
  verdict:
    decision: accepted
    primary_job: content-query
    primary_metric: wall_ns
    change_pct: 1.022
    reason: "same-binary attachment +1.02% [0.05%, 15.62%] is uncontrolled host noise; exact report oracle landed and current code inspection named four repeated per-file metric resolutions for H153"
    commit: 1ba06b19
    kept: neither
---
## What was predicted

H152 was the instrumentation and attribution gate before another `content-query` change.
The existing probe timed 100 four-view reports but discarded their answers, then hashed
only retained index and content facts.
Zero invalid samples therefore could not prove that report rows, totals, ordering, or
projection were correct.

The cell had two outputs, neither a speed claim:

- Add an outside-timer exact report oracle before accepting another report-path change.
- Profile the current path and name a bounded repeated operation before compiling a cut.

The current strategy and Astra review prioritized cross-platform algorithmic work.
The candidate hypothesis was repeated per-file resolution across Types, Families,
Languages, and Documents, but only if current evidence supported it.

## What was changed

Commit `1ba06b19` adds the exact oracle.
It constructs one combined four-view report and compares every section, in request
order, with an independently constructed single-view report.
The comparison is outside the component timer.
A mutation test swaps sections and proves the oracle fails.
All 21 probe tests and the full no-default-features core suite passed.

The oracle deliberately uses the product’s independent report path rather than a second
hand-written model. It catches omissions, ordering changes, rows, totals, and projection
differences while avoiding a duplicate implementation of report semantics.

## What was measured

Subject: live `metabrowser-clone` at commit `091d4043`, 137,085 entries, 127,104 files,
9,936 directories, 1.53 GiB apparent, maximum depth 19. Local generated, installed, and
untracked state makes the checkout non-reconstructible.
The exact fingerprint stayed unchanged across the run.

Both variants were byte-identical copies of the `1ba06b19` release probe, with the exact
oracle enabled. Three warmups, 12 timed pairs, interleaved, warm-steady cache.
The cell was declared `uncontrolled`; no sample was invalid and no oracle failed.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 37,903.9 ms | 29,518.4 ms | 1,257.4 MiB |
| same-binary candidate | 38,337.2 ms | 29,615.7 ms | 1,256.2 MiB |

The same-binary paired wall statistic is +1.02% [0.05%, 15.62%]. It is attachment noise,
not a regression: the candidate arm alone received 84.0 s and 60.1 s host outliers,
while the later pairs converged around 35.7–38.5 s on both arms.
One four-view report costs about 295 ms at the component medians.

The profiling build was calibrated separately.
The default 40 repeats, then 12 and 3, outlived the profiler’s fixed completion bound.
One repeat with an 8 s sample completed and produced 55,342 stacks.
Whole-process sampling began during untimed content-analysis setup, so `read` (62.97%)
and `open` (17.22%) dominated.
The recorded component was 31.125 s for 100 reports.
This profile cannot honestly assign a current percentage to report functions; it is
retained as evidence of that instrumentation boundary.

Direct inspection after H138 supplied the actionable fact: the shared `FileRow` walk is
already one pass, but `metric_summary` still performed `ContentIndex::file` and
`Index::classify` independently for each of the four views.
The operations and semantics were explicit enough to pre-register H153 without inventing
a broader cache.

## What the determination said

**Accepted as an instrumentation and hypothesis-selection milestone.** The report oracle
is required for future report-path verdicts.
The same-binary pair establishes the current absolute scale and the host’s noise, not a
speed delta. The setup-skewed profile is not a query-path percentage.

Advance only the bounded H153 mechanism: share current per-file content lookup and path
classification across multiple unfiltered metric views.
Keep filtered and single-view paths unchanged.
Do not substitute cached detection, add persistent identity, or claim Linux performance
from this Darwin measurement.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
