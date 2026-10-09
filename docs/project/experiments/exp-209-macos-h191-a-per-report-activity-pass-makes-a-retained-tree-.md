---
title: "macOS: H191 a per-report activity pass makes a retained tree report 6 and 20 times slower"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-209
  title: "macOS: H191 a per-report activity pass makes a retained tree report 6 and 20 times slower"
  date: "2026-10-09"
  hypotheses:
    - H191
  subject:
    tree_label: rustup
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: 4304d9d4071fd4478a0510edf80c4b78594f98b84969d90a1d7230eb0dc94d78
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains (root_id 36ce9b22). Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 77355
    tree_directories: 3427
    tree_files: 73928
    tree_symlinks: 0
    tree_apparent_bytes: 3750189949
    tree_allocated_bytes: 3978313728
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
    control: "148ef78e probe: main before the age column"
    candidate: "66652033 probe: the age column, each row aged by a per-report activity pass"
    control_binary:
      name: control
      sha256: d2ac70ff129f6c510100a0f58a27677015fec20af2f8d29b732f6e29e2182041
      size_bytes: 3363840
      args: []
    candidate_binary:
      name: candidate
      sha256: d9ef29846d45a4d0cd6c6f274186139c668c5a2e7385962fabe167cafc37579c
      size_bytes: 3380352
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-209/run.json.gz
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 130956083.0
          candidate_median: 130252687.5
          control_p95_over_median: 1.197
          candidate_p95_over_median: 1.057
          change_pct: -2.072
          ci95_low_pct: -12.159
          ci95_high_pct: 9.55
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 125087499.5
          candidate_median: 122474958.0
          control_p95_over_median: 1.205
          candidate_p95_over_median: 1.072
          change_pct: -0.756
          ci95_low_pct: -15.1
          ci95_high_pct: 12.362
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 530871500.0
          candidate_median: 567620000.0
          control_p95_over_median: 1.197
          candidate_p95_over_median: 1.171
          change_pct: -1.415
          ci95_low_pct: -7.793
          ci95_high_pct: 21.791
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 41623500.0
          candidate_median: 41779000.0
          control_p95_over_median: 1.069
          candidate_p95_over_median: 1.093
          change_pct: 4.035
          ci95_low_pct: -1.837
          ci95_high_pct: 9.782
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 492246000.0
          candidate_median: 526499000.0
          control_p95_over_median: 1.2
          candidate_p95_over_median: 1.181
          change_pct: -1.362
          ci95_low_pct: -8.428
          ci95_high_pct: 22.36
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 11821056.0
          candidate_median: 12959744.0
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.096
          change_pct: 10.414
          ci95_low_pct: 4.184
          ci95_high_pct: 14.03
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes straddles its +5% regression limit"
          - "minor_faults straddles its +10% regression limit"
          - voluntary_context_switches is missing a paired percent interval
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
          minor_faults: inconclusive
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: index-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 372760646.0
          candidate_median: 374051458.5
          control_p95_over_median: 1.213
          candidate_p95_over_median: 1.217
          change_pct: -2.268
          ci95_low_pct: -10.779
          ci95_high_pct: 5.707
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 127792.0
          candidate_median: 788749.5
          control_p95_over_median: 2.253
          candidate_p95_over_median: 4.45
          change_pct: 496.226
          ci95_low_pct: 451.678
          ci95_high_pct: 964.735
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 652581000.0
          candidate_median: 657870500.0
          control_p95_over_median: 1.23
          candidate_p95_over_median: 1.151
          change_pct: -0.362
          ci95_low_pct: -4.917
          ci95_high_pct: 6.746
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 231148500.0
          candidate_median: 232173500.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.028
          change_pct: 0.081
          ci95_low_pct: -2.986
          ci95_high_pct: 2.457
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 425368000.0
          candidate_median: 422984500.0
          control_p95_over_median: 1.354
          candidate_p95_over_median: 1.266
          change_pct: 0.051
          ci95_low_pct: -8.22
          ci95_high_pct: 9.781
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 28868608.0
          candidate_median: 29081600.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.06
          change_pct: 1.496
          ci95_low_pct: -4.507
          ci95_high_pct: 9.186
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
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: opened-second-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3257088875.0
          candidate_median: 2854748771.0
          control_p95_over_median: 1.424
          candidate_p95_over_median: 1.177
          change_pct: -18.177
          ci95_low_pct: -31.761
          ci95_high_pct: -6.385
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 251520.5
          candidate_median: 4920624.5
          control_p95_over_median: 8.627
          candidate_p95_over_median: 2.075
          change_pct: 1544.52
          ci95_low_pct: 251.981
          ci95_high_pct: 4579.652
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1486673000.0
          candidate_median: 1473265000.0
          control_p95_over_median: 1.06
          candidate_p95_over_median: 1.03
          change_pct: -0.748
          ci95_low_pct: -3.476
          ci95_high_pct: 1.133
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 1000846000.0
          candidate_median: 1011688500.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.021
          change_pct: 0.842
          ci95_low_pct: -1.795
          ci95_high_pct: 2.758
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 484426000.0
          candidate_median: 469673000.0
          control_p95_over_median: 1.167
          candidate_p95_over_median: 1.049
          change_pct: -2.268
          ci95_low_pct: -7.996
          ci95_high_pct: 0.358
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 117211136.0
          candidate_median: 118022144.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.008
          change_pct: 0.925
          ci95_low_pct: 0.411
          ci95_high_pct: 1.358
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
  reference_tools:
    - name: dust
      wall_ns_median: 347795688.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 932
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "fac3d744, the pass with its tests"
  verdict:
    decision: rejected
    primary_job: index-second-report
    primary_metric: component_ns
    change_pct: 496.226
    reason: "Second tree report over a retained Index 0.128 -> 0.789 ms, +496.23% [+451.68%, +964.74%], and over an opened root 0.252 -> 4.92 ms, +1,544.52% [+251.98%, +4,579.65%]: work per entry on every retained report; default-tree wall -2.07% [-12.16%, +9.55%] on a loaded host, peak RSS +10.41% from the per-slot table. Replaced by the maintained per-directory maximum (H192, exp-210)."
    commit: "66652033"
    kept: control
---
## What was predicted

H191, the age column’s first design: each tree row’s age, the newest modification among
every entry it counts, computed per report by one post-order pass over the index beside
`measure`, by entry id and depth with no paths.
The plan’s goal for the column was that the default report get no measurably slower in
either the one-shot or the retained regime, and the pass was expected to meet it: a
linear walk of the arena, small beside a walk of the filesystem and invisible beside a
retained report’s existing work.

## What was measured

One interleaved probe run on the `rustup` toolchain store (77,355 entries, 3,427
directories, depth 17): control the pre-age `main` (`148ef78e`), candidate the age
column with the per-report pass (`66652033`). 3 warmups and 12 timed trials per variant,
exploratory stage, on a heavily loaded host: the 1-minute load average was 91 at the
start and 127 at the end over 10 cores, with the CPU 98% to 100% busy.
No sample was invalid and the tree was unchanged.

- `index-second-report` component, the second tree report over a retained `Index`: 0.128
  ms to 0.789 ms, +496.23% [+451.68%, +964.74%]. Primary.
- `opened-second-report` component, the same over a settled opened root: 0.252 ms to
  4.92 ms, +1,544.52% [+251.98%, +4,579.65%].
- `default-tree` wall −2.07% [−12.16%, +9.55%], too wide to bound at +3% on this host;
  peak RSS +10.41% [+4.18%, +14.03%], the pass’s table with one slot per arena entry.

An opened root keeps each directory’s children in a name-keyed map whose order does not
follow the arena, so its pass touched scattered entries and cost two to three times the
detached index’s in isolation; the paired figure adds this host’s noise.
The opened root’s wall change (−18.18%) is discovery time on a loaded host, not the
report.

## Decision

Rejected. The pass adds work per entry to every retained report, 0.66 ms over a retained
`Index` and 4.7 ms over an opened root on a 77k-entry tree, so its cost grows with the
tree rather than with the rows shown.
A maximum the index maintains per directory replaced it wherever every subtree is
complete (H192,
[exp-210](exp-210-macos-h192-maintained-activity-leaves-the-age-column-per-row.md)). The
pass remains for an index that may hold an unlisted subtree, a partial walk, an opened
root mid-discovery, or a scan depth, where it also supplies the completeness the
partial-tree share proof needs.
