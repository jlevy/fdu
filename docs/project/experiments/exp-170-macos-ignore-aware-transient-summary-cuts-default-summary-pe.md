---
title: "macOS ignore-aware transient summary cuts default summary peak RSS 69% on a source checkout"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-170
  title: "macOS ignore-aware transient summary cuts default summary peak RSS 69% on a source checkout"
  date: "2026-09-28"
  hypotheses:
    - H161
  subject:
    tree_label: metabrowser-clone
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 0eed491edf9b68dce5e8660d49692971e3f11bc5710312fab4806433ddacbe42
    tree_provenance: "A clone of github.com/jlevy/metabrowser used as this host's nominated source-checkout subject (root_id a319238d), with the workspace state its use left on top of it. The clone is reproducible; the workspace state is not, so the shape is not."
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
    run_artifact: docs/project/experiments/evidence/exp-170/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 293712604.5
          candidate_median: 270792166.5
          control_p95_over_median: 1.155
          candidate_p95_over_median: 1.141
          change_pct: -4.278
          ci95_low_pct: -11.333
          ci95_high_pct: -0.214
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 288190750.0
          candidate_median: 265704188.0
          control_p95_over_median: 1.159
          candidate_p95_over_median: 1.143
          change_pct: -4.079
          ci95_low_pct: -11.219
          ci95_high_pct: -0.019
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 1686726000.0
          candidate_median: 1612795500.0
          control_p95_over_median: 1.124
          candidate_p95_over_median: 1.093
          change_pct: -4.26
          ci95_low_pct: -7.333
          ci95_high_pct: 0.087
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 133119000.0
          candidate_median: 123177500.0
          control_p95_over_median: 1.21
          candidate_p95_over_median: 1.229
          change_pct: -5.751
          ci95_low_pct: -10.411
          ci95_high_pct: 0.4
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1553882000.0
          candidate_median: 1490813500.0
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.089
          change_pct: -4.05
          ci95_low_pct: -6.782
          ci95_high_pct: 0.214
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 37625856.0
          candidate_median: 11575296.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.026
          change_pct: -69.119
          ci95_low_pct: -71.094
          ci95_high_pct: -68.71
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: superior
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
    change_pct: -69.119
    reason: "Pre-registered primary peak RSS -69.12% [-71.09%, -68.71%], past the 50% bar; wall -4.28% [-11.33%, -0.21%], non-inferior; placebo --no-controls on both arms includes zero on wall and RSS. Uncontrolled host; the Linux wall cell is pending."
    commit: 060bbfe6
    kept: candidate
---
## What was predicted

H161: the default `fdu --view summary` observes `.gitignore`, and until this change the
transient summary reducer kept no control table, so the planner fell closed to a full
retained index only to report the ignored share (`fdu-elnn`). The candidate classifies
each entry on the reducer’s one consumer exactly as the detached index builder does,
keeping the control table and the heads of ignored subtrees and nothing per entry, so
the default summary should cost what the `--no-gitignore` summary costs.

Registered in the hypothesis table before any timing ran: on macOS the primary is peak
RSS of `aggregate-summary` (bare, so controls on), down at least 50% with the interval
below zero; wall must be non-inferior, the upper bound of its interval under +3%; the
placebo is the same job with `--no-controls` on both arms, which takes the transient
tier on both builds and must include zero.
Wall is decided on Linux, where the index is a larger share of the run; that cell is
pending.

## What was measured

One interleaved probe run on `metabrowser-clone`, the nominated source checkout (137,085
entries, 59 `.gitignore` files), with four variants: the control is the release probe at
`a5c0ab46` (the engine this branch is stacked on), the candidate is the release probe at
`060bbfe6`, and each also runs with `--no-controls` as the placebo pair.
3 warmups and 12 timed trials per variant, round-robin by ordinal, warm-steady.

A quiet attempt on the first cut passed the start gate and then invalidated 16 samples
for CPU busy above 25%, leaving 7 pairs, so this run was declared `uncontrolled` before
it started. The 1-minute load average was 3.7 at its start and 4.1 at its end, no sample
was invalid, and the tree was unchanged.

- Peak RSS: −69.12% [−71.09%, −68.71%] (35.8 MiB to 11.0 MiB). Primary; passes.
- Wall: −4.28% [−11.33%, −0.21%]; non-inferior, in fact below zero on this host.
- User CPU −5.75% [−10.41%, +0.40%]; system CPU −4.05% [−6.78%, +0.21%].
- Placebo, both arms `--no-controls`: wall −1.49% [−5.91%, +2.98%], peak RSS +2.71%
  [−6.51%, +13.63%]; both include zero.

The candidate’s peak RSS is about 2 MiB above either build without controls (9.0 and 8.8
MiB) and 25 MiB below the index: classification costs the rules and the ignored subtree
heads, not an index.

A first cut of the candidate (`cbeb9e57`) held each listing whole until it ended and
measured −64.81% here; on the package cache it kept only −17.07%, recorded as
[exp-172](exp-172-macos-whole-listing-hold-keeps-only-17-rss-saving-on-wide-di.md).

## Decision

Accepted on macOS on the pre-registered primary, peak RSS, with wall non-inferior and
both placebos including zero.
The host was uncontrolled, so this is exploratory evidence, and the wall claim waits for
the Linux cell: `aggregate-summary` bare, quiet, 12 pairs, `linux-v6.12` deciding and
balanced-1M screening, against `a5c0ab46`; wall −3% with the interval below zero and
peak RSS down at least 50%, with `--no-controls` and `default-tree` placebos.

Exactness is not a measurement: the differential test in `execution.rs` compares the
whole transient report with the indexed one, and the golden and parity corpora are
unchanged.
