---
title: "macOS: H196 several roots pay about 2.4 ms a root, 625 small roots seven times one walk of their parent"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-216
  title: "macOS: H196 several roots pay about 2.4 ms a root, 625 small roots seven times one walk of their parent"
  date: "2026-10-09"
  hypotheses:
    - H196
  subject:
    tree_label: cargo-registry-src
    tree_root_id: 0d6ac3b56b7696752b6af951b3802fd843b8d1235fa49cad9f2a2214cd8e403b
    tree_engine_digest: 0305e6b87619111cf6c1a9102364e938c24264702aacc52cbf246f9081dba33c
    tree_provenance: "The cargo registry source store (index.crates.io) of this machine (root_id 0d6ac3b5): 625 crate directories, whichever crates this machine's builds downloaded, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 36526
    tree_directories: 5451
    tree_files: 31075
    tree_symlinks: 0
    tree_apparent_bytes: 428929970
    tree_allocated_bytes: 508485632
    tree_max_depth: 10
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
    control: "3f195a2b probe, the store as one root (sha256 2495b561)"
    candidate: "3f195a2b probe, each of the store 625 crate directories as a root (--child-roots, same binary)"
    control_binary:
      name: control
      sha256: 2495b5612a3fb8d61145d2eb455a91d7af2b4647d17e8eb507e7735dda638bb4
      size_bytes: 3644944
      args: []
    candidate_binary:
      name: candidate
      sha256: 2495b5612a3fb8d61145d2eb455a91d7af2b4647d17e8eb507e7735dda638bb4
      size_bytes: 3644944
      args:
        - "--child-roots"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-216/run.json.gz
  results:
    - job: roots-default-tree
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 234484312.5
          candidate_median: 1719502166.5
          control_p95_over_median: 1.167
          candidate_p95_over_median: 1.355
          change_pct: 570.249
          ci95_low_pct: 506.615
          ci95_high_pct: 732.482
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        component_ns:
          control_median: 216608479.0
          candidate_median: 1703340250.0
          control_p95_over_median: 1.225
          candidate_p95_over_median: 1.346
          change_pct: 620.884
          ci95_low_pct: 552.887
          ci95_high_pct: 760.178
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 557769000.0
          candidate_median: 764594000.0
          control_p95_over_median: 1.146
          candidate_p95_over_median: 1.086
          change_pct: 44.098
          ci95_low_pct: 24.579
          ci95_high_pct: 47.457
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        user_cpu_ns:
          control_median: 40473500.0
          candidate_median: 126907500.0
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.083
          change_pct: 209.16
          ci95_low_pct: 203.779
          ci95_high_pct: 241.149
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        system_cpu_ns:
          control_median: 516208000.0
          candidate_median: 630687000.0
          control_p95_over_median: 1.158
          candidate_p95_over_median: 1.091
          change_pct: 29.54
          ci95_low_pct: 11.722
          ci95_high_pct: 32.323
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        peak_rss_bytes:
          control_median: 9576448.0
          candidate_median: 21798912.0
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.008
          change_pct: 127.459
          ci95_low_pct: 124.225
          ci95_high_pct: 131.572
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes exceeds its +5% regression limit"
          - "minor_faults exceeds its +10% regression limit"
          - voluntary_context_switches is missing a paired percent interval
          - "involuntary_context_switches exceeds its +50% regression limit"
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
          involuntary_context_switches: rejected
          major_faults: inconclusive
          minor_faults: rejected
          peak_rss_bytes: rejected
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: roots-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 201176354.0
          candidate_median: 762396146.5
          control_p95_over_median: 1.425
          candidate_p95_over_median: 2.595
          change_pct: 313.111
          ci95_low_pct: 137.007
          ci95_high_pct: 391.035
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        component_ns:
          control_median: 188457729.5
          candidate_median: 752878458.0
          control_p95_over_median: 1.443
          candidate_p95_over_median: 2.605
          change_pct: 321.843
          ci95_low_pct: 140.401
          ci95_high_pct: 407.231
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 630938000.0
          candidate_median: 854482000.0
          control_p95_over_median: 1.317
          candidate_p95_over_median: 1.185
          change_pct: 32.76
          ci95_low_pct: 17.521
          ci95_high_pct: 47.25
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        user_cpu_ns:
          control_median: 38867500.0
          candidate_median: 108437000.0
          control_p95_over_median: 1.099
          candidate_p95_over_median: 1.131
          change_pct: 187.539
          ci95_low_pct: 171.323
          ci95_high_pct: 211.513
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        system_cpu_ns:
          control_median: 591173000.0
          candidate_median: 746924000.0
          control_p95_over_median: 1.333
          candidate_p95_over_median: 1.191
          change_pct: 21.985
          ci95_low_pct: 8.533
          ci95_high_pct: 36.77
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        peak_rss_bytes:
          control_median: 7733248.0
          candidate_median: 11493376.0
          control_p95_over_median: 1.121
          candidate_p95_over_median: 1.042
          change_pct: 49.957
          ci95_low_pct: 34.228
          ci95_high_pct: 67.579
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "peak_rss_bytes exceeds its +5% regression limit"
          - "minor_faults exceeds its +10% regression limit"
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: rejected
          major_faults: within-limit
          minor_faults: rejected
          peak_rss_bytes: rejected
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 409453646.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: roots-default-tree
    primary_metric: wall_ns
    change_pct: 570.249
    reason: "not a speed decision: it measures what several roots cost; roots-default-tree wall 234.5 to 1719.5 ms, +570% [+507%, +732%], about 2.4 ms a root, and roots-summary +313% [+137%, +391%], about 0.9 ms a root, against 190 ms predicted; far past the 10% bar, so one walker pool across roots (fdu-ich9) earns its hypothesis; user CPU and minor faults point at per-root fixed costs, unattributed"
    commit: 3f195a2b
    kept: neither
---
## What was predicted

H196, registered before this run (`afc890e8`): review C2 on
[#192](https://github.com/jlevy/fdu/pull/192). A report over several roots runs each
root’s plan one after another, so each root pays a plan’s fixed costs before it reads an
entry. The review estimated a few hundred microseconds a root on macOS. Predicted: over
the 625 crate directories of the cargo registry store, about 625 × 0.3 ms ≈ 190 ms more
wall than the same report over their parent.
If the added wall exceeded 10% of the parent’s on `roots-default-tree`, one walker pool
across roots (`fdu-ich9`) would earn a hypothesis of its own.

## What was measured

One interleaved run on the `cargo-registry-src` store (36,526 entries, 5,451
directories, depth 10, 625 crate directories and nothing else at its top), 3 warmups and
12 timed trials per configuration, exploratory stage, on an uncontrolled host: the
1-minute load average was 35.7 at the start and 33.2 at the end over 10 cores, the CPU
98% and 100% busy. No sample was invalid and the tree was unchanged.

One binary, the probe built from `3f195a2b` with a clean tree (sha256 `2495b561`), in
two configurations bound by the variant notes: the store as one root (control) and each
of its 625 crate directories as a root (`--child-roots`, candidate).
Both pass the same tallies oracle, since the probe adds the children back to the
directory count.

- `roots-default-tree`: wall 234.5 ms to 1,719.5 ms, +570% [+507%, +732%]; component
  216.6 to 1,703.3 ms; user CPU +209%, minor faults +993%, peak RSS 9.1 to 20.8 MB.
  About 2.4 ms a root.
- `roots-summary`: wall 201.2 ms to 762.4 ms, +313% [+137%, +391%]; user CPU +188%,
  minor faults +1,098%, peak RSS 7.4 to 11.0 MB. About 0.9 ms a root.

## Decision

Not a speed decision: it measures what several roots cost, which ships regardless.
The prediction was low by about eight times on the tree and three on the summary; the
added wall is several times the parent’s on both, far past the 10% bar, so `fdu */` over
many small directories is several times slower than `fdu .` over their parent.
The growth in user CPU and minor faults points at fixed costs paid per root, such as a
walker pool spawned and calibrated for every root, rather than at the entries.
Where the time goes is not attributed here; a profile is the next step.
One walker pool across roots (`fdu-ich9`) earns its hypothesis.
Recorded `rejected` as a speed claim; neither configuration is a change that ships.
