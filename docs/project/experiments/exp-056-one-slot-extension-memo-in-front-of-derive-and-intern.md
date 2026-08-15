---
title: One-slot extension memo in front of derive-and-intern
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-056
  title: One-slot extension memo in front of derive-and-intern
  date: "2026-08-15"
  hypotheses:
    - H89
  subject:
    tree_label: vm450k
    tree_root_id: 9311b5fa18bc84e62d74e610f48c354dc72b352a0e0ebb6a5dc6847091a61ce0
    tree_engine_digest: 45022e107978f96adc9624cfa30f54271d024961832dcf0192188eabcacf7ea5
    tree_entries: 450463
    tree_directories: 28630
    tree_files: 421690
    tree_symlinks: 143
    tree_apparent_bytes: 3000524491
    tree_allocated_bytes: 747966464
    tree_max_depth: 20
    tree_mutated_during_run: false
    host_cpu: Linux
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 0
    host_system: Linux 6.18.5-fc-v20
    filesystem: ""
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: "branch head 3bda5c8: derive_ext + BTreeMap intern per file"
    candidate: "raw-suffix one-slot memo answering repeat extensions without derive, validate, or search"
    control_binary:
      name: control
      sha256: 2b574c6ecfdbbed12907144b6325f627564cf44afc2bbb66d876a41382827de2
      size_bytes: 1870760
      args: []
    candidate_binary:
      name: candidate
      sha256: c6ec0e6c5132b204996975fee8ad029db74f21a353b8dd6e40a89aca45070113
      size_bytes: 1872488
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: /tmp/fdu-realtree/results/run-exp056-ext-memo.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1767498512.5
          candidate_median: 1788354672.5
          change_pct: 1.589
          ci95_low_pct: -1.618
          ci95_high_pct: 6.082
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 700302368.5
          candidate_median: 726193323.5
          change_pct: 4.728
          ci95_low_pct: -3.451
          ci95_high_pct: 15.331
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 2687413000.0
          candidate_median: 2734637000.0
          change_pct: 1.124
          ci95_low_pct: -0.581
          ci95_high_pct: 5.66
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 1849040000.0
          candidate_median: 1863043500.0
          change_pct: 1.422
          ci95_low_pct: 0.392
          ci95_high_pct: 3.985
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 866657000.0
          candidate_median: 879231000.0
          change_pct: -0.162
          ci95_low_pct: -2.678
          ci95_high_pct: 11.224
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 278556672.0
          candidate_median: 275318784.0
          change_pct: 1.428
          ci95_low_pct: -3.807
          ci95_high_pct: 3.356
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
          control_median: 1736320958.0
          candidate_median: 1726158739.0
          change_pct: -0.26
          ci95_low_pct: -1.704
          ci95_high_pct: 1.004
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 290628037.0
          candidate_median: 297432712.5
          change_pct: 1.085
          ci95_low_pct: -2.285
          ci95_high_pct: 3.513
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 2537734500.0
          candidate_median: 2542352500.0
          change_pct: 0.585
          ci95_low_pct: -1.593
          ci95_high_pct: 1.335
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 1692314000.0
          candidate_median: 1733277000.0
          change_pct: 0.625
          ci95_low_pct: -1.581
          ci95_high_pct: 4.568
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 830479500.0
          candidate_median: 833889000.0
          change_pct: 0.543
          ci95_low_pct: -5.76
          ci95_high_pct: 3.407
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 199897088.0
          candidate_median: 199659520.0
          change_pct: -0.106
          ci95_low_pct: -0.147
          ci95_high_pct: -0.094
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools:
    - name: dust
      wall_ns_median: 649493381.0
      argv:
        - "{binary}"
        - "-s"
        - "-k"
        - "{root}"
  complexity:
    lines_changed: 130
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 1.589
    reason: "Wall +1.59% [-1.62%, +6.08%] and user CPU regressed +1.42% [+0.39%, +3.98%]: the per-file suffix extraction and byte compare cost as much as the small-Vec derive and intern they replaced, the H51/H62 pattern on a new site"
    commit: 3bda5c8
---
The callgrind profile put `derive_ext`'s `from_utf8` at ~3% of engine instructions and
dhat showed ~0.9 allocations per file, so a one-slot memo keyed on the raw extension
suffix looked like a clean cut: equal raw bytes derive equal extensions, a hit bumps
the same refcount `intern_ext` would, and every `release_ext` clears the memo so a
reissued id can never be answered from stale bytes.

The mechanism worked - an agreement test pinned the selection to `derive_ext_units`
across dotfiles, `.tar` widening, and trailing dots, and a churn test held the
refcounts exact - but the measurement refused it: the memo's own extraction and
compare per file costs what the derive-and-intern path saved. glibc's same-thread
small-allocation fast path is again cheaper than modeled, which is the third time
this campaign has measured that shape (H51 move-not-clone, H62 worker-local
reduction).

The residual truth accrues to H86: the win is not shaving this path but deleting it -
arena entries intern from a batch context where the run-length structure is explicit.
Reverted in full, including the `raw_ext_suffix` helper and both tests.
