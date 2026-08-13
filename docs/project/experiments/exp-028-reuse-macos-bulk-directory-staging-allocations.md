---
title: Reuse macOS bulk directory staging allocations
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-028
  title: Reuse macOS bulk directory staging allocations
  date: 2026-08-12
  hypotheses:
    - H54
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
    control: exp-026 bulk reader with one fresh entry vector per directory
    candidate: "one retained entry vector per bulk reader, drained after complete validation"
    control_binary:
      name: control
      sha256: 35198f0525f9501b71bd6764362f35723c925a3689b99c587bfbc457da896019
      size_bytes: 569104
      args: []
    candidate_binary:
      name: candidate
      sha256: 76c5a9e5fc4c49463a70489fa6faf5e81e47b16d9f4d8ead88cca7aa6cad4b5c
      size_bytes: 569120
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp028-reuse-bulk-staging-small-final.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 382236416.5
          candidate_median: 307767896.0
          change_pct: 0.207
          ci95_low_pct: -7.089
          ci95_high_pct: 3.88
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 261097020.5
          candidate_median: 192003083.5
          change_pct: -2.197
          ci95_low_pct: -12.096
          ci95_high_pct: 5.74
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1384266000.0
          candidate_median: 1183389000.0
          change_pct: -2.815
          ci95_low_pct: -20.026
          ci95_high_pct: 1.1
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 225763000.0
          candidate_median: 225608000.0
          change_pct: -0.761
          ci95_low_pct: -3.326
          ci95_high_pct: 4.163
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 1169937500.0
          candidate_median: 955984000.0
          change_pct: -3.638
          ci95_low_pct: -23.257
          ci95_high_pct: 2.288
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
          control_median: 35315712.0
          candidate_median: 34840576.0
          change_pct: -1.294
          ci95_low_pct: -2.424
          ci95_high_pct: 1.336
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
          control_median: 390263333.5
          candidate_median: 396029208.5
          change_pct: 1.325
          ci95_low_pct: 0.292
          ci95_high_pct: 3.119
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        component_ns:
          control_median: 160659416.5
          candidate_median: 163037187.0
          change_pct: 1.87
          ci95_low_pct: 1.12
          ci95_high_pct: 4.309
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        cpu_ns:
          control_median: 1739344000.0
          candidate_median: 1761523500.0
          change_pct: -0.062
          ci95_low_pct: -1.26
          ci95_high_pct: 2.832
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 235304500.0
          candidate_median: 229575500.0
          change_pct: -3.073
          ci95_low_pct: -7.877
          ci95_high_pct: 0.264
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 1502370000.0
          candidate_median: 1532167500.0
          change_pct: 0.647
          ci95_low_pct: -0.447
          ci95_high_pct: 3.506
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
          control_median: 34824192.0
          candidate_median: 35323904.0
          change_pct: 1.531
          ci95_low_pct: 0.547
          ci95_high_pct: 2.778
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 505724917.0
          candidate_median: 503263062.5
          change_pct: -0.848
          ci95_low_pct: -1.531
          ci95_high_pct: 0.549
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 305812375.5
          candidate_median: 303789646.0
          change_pct: -1.09
          ci95_low_pct: -2.643
          ci95_high_pct: 0.414
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 501752000.0
          candidate_median: 499681000.0
          change_pct: -0.794
          ci95_low_pct: -1.492
          ci95_high_pct: 0.433
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 223681500.0
          candidate_median: 223012500.0
          change_pct: -0.723
          ci95_low_pct: -1.276
          ci95_high_pct: 0.071
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 277554500.0
          candidate_median: 277024000.0
          change_pct: -0.623
          ci95_low_pct: -1.883
          ci95_high_pct: 0.478
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        blocked_ns:
          control_median: 3756229.0
          candidate_median: 3565979.0
          change_pct: -2.79
          ci95_low_pct: -15.009
          ci95_high_pct: 19.256
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 32849920.0
          candidate_median: 32489472.0
          change_pct: -0.297
          ci95_low_pct: -1.597
          ci95_high_pct: 0.405
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 20
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - each reader retains the largest directory entry-vector capacity it encounters until the worker or reconciliation ends
    notes: 14 insertions and 6 deletions; no dependency or unsafe change; 720k confirmation was gated on a promising 60k result and did not run
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 0.207
    reason: "Cold-index wall was +0.21%, producer regressed 1.32%, and warm wall was -0.85%; predicted user-CPU and fault reductions were absent while producer RSS and faults regressed"
    commit: null
---
# Reuse macOS bulk directory staging allocations

## Hypothesis

H54 targeted the allocation residue after exp-026. Each `macos_bulk::Reader::read`
created and dropped one `Vec<Entry>` per directory--7,350 times on the 60k subject and
88,201 times on the 720k subject.
Retaining that vector in the reader and draining it after a successful
complete-directory parse should reuse capacity, reduce allocator work and faults, and
benefit cold and warm consumers without weakening fallback.

## What was tried

The reader gained an entry vector beside its existing 64 KiB syscall buffer.
A read cleared and refilled the vector, returned a draining iterator only after the
entire directory validated, and cleared partial results before portable fallback.
No caller, syscall, parser, index, or observation behavior changed.
The candidate added fourteen lines and removed six, with no dependency or unsafe change.

The exact exp-026 binary and candidate ran twelve interleaved pairs after three warmups
for cold index, producer-only, and full warm revalidation on a freshly fingerprinted,
immutable 60,067-entry APFS subject.
The pre-registered plan required a 3% primary wall/component gain with lower user CPU or
faults before spending a long run at 720k.

## What the numbers said

Cold-index wall was +0.21% [-7.09%, +3.88%], its component -2.20%, and user CPU -0.76%;
all intervals included zero.
Producer wall regressed 1.32% [+0.29%, +3.12%] and its component 1.87%, while the
predicted user-CPU and fault improvements were unclear; RSS instead regressed 1.53% and
faults 1.18%. Warm wall was -0.85% [-1.53%, +0.55%], with component, user CPU, RSS, and
faults all unclear. Every sample passed the oracle and the tree remained unchanged.

A temporary host-load interval affected both indexed variants and widened that job’s
interval.
The quiet producer and warm jobs resolve the hypothesis independently: there is
no meaningful reduction in allocation or CPU. The system allocator already recycles
these small, short-lived vector buffers effectively; retaining one only changes where
capacity remains live.

## Verdict

**Rejected and reverted.** No measured path reached the 3% gate or showed the predicted
mechanism, and producer memory/fault counters moved in the wrong direction.
The 720k run was not triggered.
Future allocation work should target the per-entry owned name/path objects visible to
consumers, not the directory vector’s backing allocation.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
