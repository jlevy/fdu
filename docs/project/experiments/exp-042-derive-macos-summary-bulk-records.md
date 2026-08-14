---
title: Derive macOS summary bulk records
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-042
  title: Derive macOS summary bulk records
  date: "2026-08-13"
  hypotheses:
    - H63
  subject:
    tree_label: h63-self-contained-901k
    tree_root_id: c95b1edda5762c399d4eaaf8494b1e1866f5554814d9db5c3fe353a5a13bc7a0
    tree_engine_digest: e7ed1ac6334eb80379d3a8b259188115462014f247c147b5560682cbb27d1fca
    tree_entries: 901963
    tree_directories: 110369
    tree_files: 791261
    tree_symlinks: 333
    tree_apparent_bytes: 16537459815
    tree_allocated_bytes: 18714202112
    tree_max_depth: 23
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
    trials: 16
    warmups: 3
    interleaved: true
    control: H59 transient summary reduced from generic observation batches
    candidate: H62 worker-local reduction plus a requirement-derived macOS bulk record
    control_binary:
      name: h59
      sha256: 0a02839ffe9c0221c96fbca2d20e2d6f97636b1891860389fee468986a677f73
      size_bytes: 1299840
      args:
        - --cache
        - "off"
        - --view
        - summary
        - --format
        - json
        - --color
        - never
    candidate_binary:
      name: h63
      sha256: d6c22fff47023051cd167175da7ede91b1fcb4e9cb167826d298af63f1dd5239
      size_bytes: 1332912
      args:
        - --cache
        - "off"
        - --view
        - summary
        - --format
        - json
        - --color
        - never
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp042-derived-summary-bulk.json
  results:
    - job: rich-summary-report
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3199502916.5
          candidate_median: 3251150708.5
          change_pct: 1.857
          ci95_low_pct: -1.959
          ci95_high_pct: 4.558
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 16
        cpu_ns:
          control_median: 11245731500.0
          candidate_median: 10860363000.0
          change_pct: -2.936
          ci95_low_pct: -4.031
          ci95_high_pct: -0.988
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        user_cpu_ns:
          control_median: 499198500.0
          candidate_median: 246836000.0
          change_pct: -50.958
          ci95_low_pct: -51.721
          ci95_high_pct: -49.844
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        system_cpu_ns:
          control_median: 10741372500.0
          candidate_median: 10613907500.0
          change_pct: -0.626
          ci95_low_pct: -1.859
          ci95_high_pct: 1.361
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 16
        peak_rss_bytes:
          control_median: 14557184.0
          candidate_median: 8847360.0
          change_pct: -39.697
          ci95_low_pct: -40.461
          ci95_high_pct: -38.623
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        minor_faults:
          control_median: 1136.5
          candidate_median: 768.5
          change_pct: -32.318
          ci95_low_pct: -33.794
          ci95_high_pct: -31.725
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 16
        involuntary_context_switches:
          control_median: 28758.0
          candidate_median: 27427.0
          change_pct: -9.279
          ci95_low_pct: -11.58
          ci95_high_pct: 0.639
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 16
        voluntary_context_switches:
          control_median: 73589.5
          candidate_median: 73580.5
          change_pct: -0.012
          ci95_low_pct: -0.035
          ci95_high_pct: 0.037
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 16
  reference_tools: []
  complexity:
    lines_changed: 547
    new_dependencies: []
    new_unsafe_blocks: 1
    new_failure_modes:
      - A second strict kernel-record parser could drift from the full metadata parser
      - A reduction-specific walker could drift from the generic scan contract
    notes: The complete H62 plus H63 composition was screened against committed H59 and is reverted because its primary wall metric missed the production bar
  verdict:
    decision: rejected
    primary_job: rich-summary-report
    primary_metric: wall_ns
    change_pct: 1.857
    reason: "The composition changes paired wall by +1.86% with a 95% interval spanning -1.96% to +4.56%; lower user CPU and RSS do not justify a second walker and macOS record parser without a user-visible speedup"
    commit: null
---
# Derive macOS Summary Bulk Records

## Hypothesis

H63: an exact transient summary does not need index-only inode and ctime fields, file
paths, or directory byte sizes.
A separate strict `getattrlistbulk` request could therefore return narrower records,
validate file names without allocating them, and reduce file tallies before data leaves
each scan worker.

The candidate composed H63 with the rejected H62 worker-local reducer because the narrow
record cannot be consumed by H59’s generic observation path.
The production verdict was preregistered against committed H59, not against H62, so two
small layers could be retained only if their complete composition cleared the 3% wall
gate.

## Method

The immutable H59 binary at `0916a40` and the H62 plus H63 prototype ran sixteen
adjacent pairs after three warmups on the mutation-free self-contained 901,963-entry
APFS tree. Both commands used the same `fdu-transient-summary` contract and every sample
produced the same stable semantic digest.
There were no invalid samples, semantic mismatches, baseline drift, or tree mutation.

The candidate kept fdu’s directory queue, scope rules, adaptive pool, partial-error
contract, and portable fail-closed fallback.
Its macOS parser required returned attribute bitmaps, per-entry errors, names, object
types, modification times, flags, file logical and allocated sizes, and directory mount
status; device identity remained conditional on `--one-filesystem`.

## Results

The complete composition changed paired wall time by **+1.86%**, with a 95% interval
from −1.96% to +4.56%. That is neither an improvement nor evidence of a definite
regression, and it misses the 3% production threshold.

The internal mechanism did reduce user CPU 50.96%, peak RSS 39.70%, and minor faults
32.32% relative to H59. Aggregate CPU improved 2.94%, but system CPU—the dominant
cost—was unchanged. The kernel walk therefore remained the wall-time floor, while the
added record-parser and reduction-specific control flow did not convert narrower
retained state into a speedup.

## Verdict

**REJECTED** — H62 plus H63 does not improve the primary wall metric and would add a
second walker plus a second unsafe macOS record parser.
Both engine layers are reverted.

This closes a tempting branch of the search: after H59, eliminating more Rust-side
allocation and retained state can produce excellent CPU and memory counters without
materially accelerating a warm APFS traversal.
The next experiments should attack the syscall-dominated path or intentionally request a
smaller semantic result, rather than further duplicating the rich-summary reducer.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
