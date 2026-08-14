---
title: Post-composable-CLI integration validation
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-033
  title: Post-composable-CLI integration validation
  date: "2026-08-12"
  hypotheses:
    - H3
    - H31
    - H53
    - H12
    - H9
  subject:
    tree_label: metabrowser-20260812
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: ce5a7430e152412a519ee9f9776c2fec73e59c58fa553aa3e9c2f8c085d26619
    tree_entries: 60067
    tree_directories: 7350
    tree_files: 52695
    tree_symlinks: 22
    tree_apparent_bytes: 1085083672
    tree_allocated_bytes: 1230073856
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
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: "origin/main dc56f77 after merged composable CLI PR #5"
    candidate: "PR #8 after merging origin/main, correctness review, and exact CLI-equivalence checks"
    control_binary:
      name: control
      sha256: 9d41606709d53bd13ea5311b4a33b796a43e39ba3f3c7fc50def2f80964091f3
      size_bytes: 552480
      args: []
    candidate_binary:
      name: candidate
      sha256: 3da6d0a6c284c6d89958204232a4647e5a9dced0c5316a4060439a2e23f2ff33
      size_bytes: 602192
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp033-post-cli-merge-full.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 329161895.5
          candidate_median: 315447458.5
          change_pct: -5.058
          ci95_low_pct: -7.314
          ci95_high_pct: -2.144
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 198382229.5
          candidate_median: 187626542.0
          change_pct: -6.951
          ci95_low_pct: -10.061
          ci95_high_pct: -2.559
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 1329757500.0
          candidate_median: 1278764500.0
          change_pct: -4.834
          ci95_low_pct: -7.415
          ci95_high_pct: -2.065
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 255957500.0
          candidate_median: 219235500.0
          change_pct: -14.035
          ci95_low_pct: -14.555
          ci95_high_pct: -13.433
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 1074266500.0
          candidate_median: 1058447000.0
          change_pct: -2.654
          ci95_low_pct: -5.556
          ci95_high_pct: 0.461
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 34037760.0
          candidate_median: 34160640.0
          change_pct: 0.265
          ci95_low_pct: -0.333
          ci95_high_pct: 1.135
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 486764875.0
          candidate_median: 440850145.5
          change_pct: -8.765
          ci95_low_pct: -12.074
          ci95_high_pct: -6.745
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 197807000.0
          candidate_median: 186254645.5
          change_pct: -6.072
          ci95_low_pct: -9.751
          ci95_high_pct: -0.669
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 2227355000.0
          candidate_median: 2004129500.0
          change_pct: -10.159
          ci95_low_pct: -12.798
          ci95_high_pct: -7.411
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 335817000.0
          candidate_median: 259143000.0
          change_pct: -22.679
          ci95_low_pct: -23.106
          ci95_high_pct: -21.753
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 1876517000.0
          candidate_median: 1736618000.0
          change_pct: -8.157
          ci95_low_pct: -10.482
          ci95_high_pct: -4.813
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 34422784.0
          candidate_median: 34955264.0
          change_pct: 0.959
          ci95_low_pct: 0.072
          ci95_high_pct: 2.267
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: cold-snapshot-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 370593292.0
          candidate_median: 352806646.0
          change_pct: -4.127
          ci95_low_pct: -5.541
          ci95_high_pct: -2.481
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 29895166.5
          candidate_median: 31226833.5
          change_pct: 0.786
          ci95_low_pct: -1.044
          ci95_high_pct: 5.347
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1399010000.0
          candidate_median: 1337562000.0
          change_pct: -3.199
          ci95_low_pct: -5.514
          ci95_high_pct: -2.183
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 288307500.0
          candidate_median: 254900500.0
          change_pct: -11.607
          ci95_low_pct: -14.571
          ci95_high_pct: -10.145
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 1113035500.0
          candidate_median: 1087800000.0
          change_pct: -1.516
          ci95_low_pct: -2.896
          ci95_high_pct: 0.134
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 42975232.0
          candidate_median: 43163648.0
          change_pct: 0.639
          ci95_low_pct: -0.563
          ci95_high_pct: 1.094
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 844736270.5
          candidate_median: 481882750.0
          change_pct: -42.261
          ci95_low_pct: -44.601
          ci95_high_pct: -40.997
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 594955937.5
          candidate_median: 229554437.5
          change_pct: -61.369
          ci95_low_pct: -62.756
          ci95_high_pct: -59.425
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 806263000.0
          candidate_median: 1077563000.0
          change_pct: 34.786
          ci95_low_pct: 30.66
          ci95_high_pct: 37.7
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 292764500.0
          candidate_median: 292306500.0
          change_pct: 0.104
          ci95_low_pct: -0.836
          ci95_high_pct: 0.699
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 513750000.0
          candidate_median: 782483000.0
          change_pct: 55.738
          ci95_low_pct: 49.211
          ci95_high_pct: 59.092
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        blocked_ns:
          control_median: 39256770.5
          candidate_median: 0.0
          change_pct: -100.0
          ci95_low_pct: -100.0
          ci95_high_pct: -100.0
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 33341440.0
          candidate_median: 34545664.0
          change_pct: 3.158
          ci95_low_pct: 2.518
          ci95_high_pct: 4.595
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: warm-snapshot-load
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 250593542.0
          candidate_median: 251164666.5
          change_pct: 0.181
          ci95_low_pct: -4.587
          ci95_high_pct: 2.588
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 117043458.0
          candidate_median: 114941229.5
          change_pct: -0.416
          ci95_low_pct: -4.875
          ci95_high_pct: 1.242
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 242341500.0
          candidate_median: 239212000.0
          change_pct: -1.126
          ci95_low_pct: -2.118
          ci95_high_pct: -0.187
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 233025000.0
          candidate_median: 230480000.0
          change_pct: -1.26
          ci95_low_pct: -1.844
          ci95_high_pct: -0.46
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 9140500.0
          candidate_median: 8986000.0
          change_pct: -2.435
          ci95_low_pct: -8.115
          ci95_high_pct: 6.871
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 8857021.0
          candidate_median: 12832104.5
          change_pct: 37.598
          ci95_low_pct: -43.978
          ci95_high_pct: 100.201
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 31531008.0
          candidate_median: 31088640.0
          change_pct: -1.274
          ci95_low_pct: -2.141
          ci95_high_pct: -0.395
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools:
    - name: dust
      wall_ns_median: 286559187.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 1514
    new_dependencies:
      - libc = 0.2.189 (macOS target only; already locked transitively)
    new_unsafe_blocks: 1
    new_failure_modes:
      - Malformed or unsupported bulk metadata must retry the complete directory through the portable reader
      - Adaptive reserve workers and reconciliation wave workers can panic; the scan reports partial rather than asserting complete truth
      - A reconciliation wave that exceeds its bounded deferred-op budget discards that wave and retries serially
    notes: Integration anchor includes previously reviewed experiments exp-015 through exp-030 plus correctness hardening and equivalence tests; individual experiment records own each optimization decision
  verdict:
    decision: accepted
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -42.261
    reason: "Against current origin/main, candidate improves cold index 5.06%, producer 8.77%, scan-plus-save 4.13%, and warm revalidation 42.26%; snapshot-only load is neutral at +0.18%, all 120 candidate/control oracle samples are valid, and the tree remained unchanged"
    commit: null
---
# Post-composable-CLI integration validation

## Question

Did the complete performance branch remain a strict, exact improvement after the
composable CLI landed on `origin/main`, or did the merge invalidate an earlier result?
This is an integration reproduction of H3, H31, H53, H12, and H9, not a new mechanism.

## Method

The merged-CLI `origin/main` binary and the rebased performance binary ran twelve
interleaved pairs after three warmups on a 60,067-entry APFS tree.
The harness exercised cold index construction, producer-only scanning, scan plus
snapshot save, compatible snapshot revalidation, and snapshot-only load.
Every sample had to match the independent engine digest and byte/count oracle, and the
tree was fingerprinted again afterward.

## Results

The candidate improved cold indexed wall time by 5.06% [2.14%, 7.31%], producer wall by
8.77% [6.75%, 12.07%], and scan-plus-save wall by 4.13% [2.48%, 5.54%]. Warm
revalidation improved 42.26% [41.00%, 44.60%]. Snapshot-only load was neutral at +0.18%
[-4.59%, +2.59%], so no load-path claim is made from this comparison.

The warm wall win spent more parallel CPU: total CPU rose 34.79%, system CPU rose
55.74%, and RSS rose 3.16%. That trade is explicit and bounded; it does not change
indexed data, query semantics, CLI rendering, or error handling.
All 120 timed control/candidate oracle samples were valid and the tree remained
unchanged.

## Verdict

**ACCEPTED** — Against current origin/main, candidate improves cold index 5.06%,
producer 8.77%, scan-plus-save 4.13%, and warm revalidation 42.26%; snapshot-only load
is neutral at +0.18%, all 120 candidate/control oracle samples are valid, and the tree
remained unchanged

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
