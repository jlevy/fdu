---
title: "macOS ignore-aware transient summary cuts peak RSS 58% on a tree with no .gitignore"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-171
  title: "macOS ignore-aware transient summary cuts peak RSS 58% on a tree with no .gitignore"
  date: "2026-09-28"
  hypotheses:
    - H161
  subject:
    tree_label: rustup-toolchains
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: b2e3920a70095ca9ea73098ff48319125083a027b1dd8921458931a508f28ec7
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains (root_id 36ce9b22). Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 77159
    tree_directories: 3420
    tree_files: 73739
    tree_symlinks: 0
    tree_apparent_bytes: 3212541127
    tree_allocated_bytes: 3440263168
    tree_max_depth: 17
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
    control: "a5c0ab46 probe: the default summary falls closed to the full index"
    candidate: "060bbfe6 probe: the transient summary classifies entries against .gitignore"
    control_binary:
      name: control
      sha256: 02e8f3805362af40ee4b2fc83cb1518d039a1dac947280dfb254f6bfac7bf9c3
      size_bytes: 3148912
      args: []
    candidate_binary:
      name: candidate
      sha256: ce786fdbf93dd8135b30b74c420f8ab9fe3ff31a704502678ac2fb3d8ee0ee02
      size_bytes: 3165440
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-171/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 129686604.5
          candidate_median: 126022708.0
          control_p95_over_median: 1.173
          candidate_p95_over_median: 1.062
          change_pct: -3.87
          ci95_low_pct: -10.856
          ci95_high_pct: 0.013
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 123877354.5
          candidate_median: 120342791.5
          control_p95_over_median: 1.187
          candidate_p95_over_median: 1.071
          change_pct: -3.562
          ci95_low_pct: -10.954
          ci95_high_pct: 0.219
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 642986000.0
          candidate_median: 607661000.0
          control_p95_over_median: 1.037
          candidate_p95_over_median: 1.051
          change_pct: -5.79
          ci95_low_pct: -6.566
          ci95_high_pct: 2.988
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 60734000.0
          candidate_median: 40964000.0
          control_p95_over_median: 1.037
          candidate_p95_over_median: 1.008
          change_pct: -32.249
          ci95_low_pct: -33.762
          ci95_high_pct: -29.018
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 583154500.0
          candidate_median: 567125000.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.054
          change_pct: -2.555
          ci95_low_pct: -4.561
          ci95_high_pct: 6.476
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 26345472.0
          candidate_median: 11395072.0
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.139
          change_pct: -57.86
          ci95_low_pct: -58.978
          ci95_high_pct: -53.963
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    lines_changed: 999
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - a listing that fills a batch before its .gitignore is listed takes one extra metadata probe
    notes: "Two engine files carry the change: scan.rs groups a classifying fold's controls ahead of their entries on the streaming path only (the detached builder is untouched), and execution.rs adds the SummaryFold reducer; query_report.rs shares its notes with report_in. lines_changed counts crates/fdu-core/src including about 450 test lines."
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: peak_rss_bytes
    change_pct: -57.86
    reason: "Pre-registered primary peak RSS -57.86% [-58.98%, -53.96%], past the 50% bar; wall -3.87% [-10.86%, +0.01%], non-inferior; user CPU -32.25%; placebo --no-controls on both arms includes zero. Uncontrolled host; the Linux wall cell is pending."
    commit: 060bbfe6
    kept: candidate
---
## What was predicted

H161, on the second pre-registered subject: `rustup-toolchains` holds no `.gitignore` at
all, the regime of the million-entry Linux tree where the fall-closed index cost 0.32 s
and 309 MiB. With no rule to apply, the classifying transient summary should cost what
the `--no-gitignore` summary costs.

The same registration as
[exp-170](exp-170-macos-ignore-aware-transient-summary-cuts-default-summary-pe.md):
primary peak RSS of `aggregate-summary` (bare) down at least 50% with the interval below
zero; wall non-inferior at +3%; placebo `--no-controls` on both arms includes zero.

## What was measured

One interleaved probe run on the nominated package cache (77,159 entries, 3,420
directories, listings of up to about 10,000 entries), four variants as in exp-170:
`a5c0ab46` and `060bbfe6`, each bare and with `--no-controls`. 3 warmups and 12 timed
trials per variant, declared `uncontrolled` after a quiet attempt on the first cut
invalidated 10 samples for CPU busy above 25%. CPU busy was 36% at the start and 28% at
the end; no sample was invalid and the tree was unchanged.

- Peak RSS: −57.86% [−58.98%, −53.96%] (25.1 MiB to 10.8 MiB). Primary; passes.
- Wall: −3.87% [−10.86%, +0.01%]; non-inferior.
- User CPU: −32.25% [−33.76%, −29.02%], the index build that no longer happens.
- Placebo, both arms `--no-controls`: wall −1.17% [−14.92%, +5.68%], peak RSS +1.78%
  [−4.53%, +5.63%]; both include zero.

With no control file in the tree the candidate’s peak RSS equals the no-controls tier’s
(10.5 and 10.7 MiB), which is what the fast path in the fold predicts: no rule and no
ignored subtree means no per-entry work beyond the tallies.

## Decision

Accepted on macOS on the pre-registered primary, with wall non-inferior and the placebo
including zero, on an uncontrolled host.
The first cut’s result on this subject is the reason the shipped emission sends at
`batch_size`: see
[exp-172](exp-172-macos-whole-listing-hold-keeps-only-17-rss-saving-on-wide-di.md).
The Linux wall cell is pending, with the protocol stated in exp-170.
