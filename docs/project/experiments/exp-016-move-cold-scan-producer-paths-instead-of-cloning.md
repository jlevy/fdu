---
title: Move cold-scan producer paths instead of cloning
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-016
  title: Move cold-scan producer paths instead of cloning
  date: "2026-08-12"
  hypotheses:
    - H51
  subject:
    tree_label: metabrowser
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
    control: current region-scheduled breadth-first producer
    candidate: move non-directory relative paths into observation ops
    control_binary:
      name: control
      sha256: be3349ee5238da00b5bce9ff7f72e68fd3fc0a9f96eae16c969c520f0e90977f
      size_bytes: 535968
      args: []
    candidate_binary:
      name: candidate
      sha256: 2a3329f3c4436109bbc34d2ec4bdb667bd3fcb94639fb51a4ad1fa2db9d01b1d
      size_bytes: 552480
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp016-producer-path-ownership-small.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 336046979.5
          candidate_median: 339900520.5
          change_pct: -0.437
          ci95_low_pct: -5.305
          ci95_high_pct: 1.516
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 216950854.5
          candidate_median: 221742729.0
          change_pct: -0.867
          ci95_low_pct: -7.759
          ci95_high_pct: 2.296
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 1198457500.0
          candidate_median: 1217453000.0
          change_pct: 0.917
          ci95_low_pct: -3.812
          ci95_high_pct: 5.838
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 251034500.0
          candidate_median: 250972500.0
          change_pct: -1.161
          ci95_low_pct: -2.734
          ci95_high_pct: 2.268
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 947337000.0
          candidate_median: 963889000.0
          change_pct: 1.175
          ci95_low_pct: -5.184
          ci95_high_pct: 7.953
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 34177024.0
          candidate_median: 35463168.0
          change_pct: 3.876
          ci95_low_pct: 1.503
          ci95_high_pct: 4.777
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 539727291.5
          candidate_median: 543484729.0
          change_pct: 1.364
          ci95_low_pct: -2.389
          ci95_high_pct: 6.966
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 220519479.0
          candidate_median: 227601521.0
          change_pct: 1.771
          ci95_low_pct: -5.736
          ci95_high_pct: 8.007
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 2035048000.0
          candidate_median: 2060449000.0
          change_pct: 2.269
          ci95_low_pct: -3.558
          ci95_high_pct: 5.457
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 329644000.0
          candidate_median: 327719500.0
          change_pct: -0.401
          ci95_low_pct: -2.109
          ci95_high_pct: 2.664
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 1711063500.0
          candidate_median: 1728383500.0
          change_pct: 2.386
          ci95_low_pct: -4.533
          ci95_high_pct: 6.778
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 34250752.0
          candidate_median: 35782656.0
          change_pct: 3.993
          ci95_low_pct: 3.294
          ci95_high_pct: 6.2
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 5
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: Reverted; transferring ownership changes which allocation remains live in each batch but did not reduce the measured work.
  verdict:
    decision: rejected
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -0.437
    reason: "Wall and CPU were unchanged while peak RSS and minor faults regressed about 4%, so the ownership rewrite is not worth carrying"
    commit: null
---
# Move cold-scan producer paths instead of cloning

## Hypothesis

H51 separated two similarly described allocations.
Experiment 003 removed copies made inside index arbitration and concluded that the
remaining allocator cost was in the producer.
The portable producer still built a relative `PathBuf`, cloned it into an upsert, and
retained the original only when the entry was a directory.
Moving the path for the mostly-file case should have removed one allocation per file,
reducing producer user CPU and minor faults before affecting wall time.

## What was tried

Both the serial reference walker and the parallel worker moved each relative path into
its `Op::Upsert`. They cloned a second path only for directories that had to remain in
the traversal frontier.
No observation, batching, ordering, or index behavior changed.

## What the numbers said

Twelve interleaved pairs on the immutable 60,067-entry tree found no speed or CPU
signal. Cold-index wall was −0.44% [−5.30%, +1.52%], producer wall was +1.36%
[−2.39%, +6.97%], and both user-CPU intervals crossed zero.

The memory counters moved in the wrong direction.
Peak RSS regressed 3.88% [+1.50%, +4.78%] for cold-index and 3.99% [+3.29%, +6.20%] for
the producer; minor faults regressed 3.05% and 3.80%, respectively.
The result exposes why allocation counts alone were misleading: in the control, the
allocator can recycle the original short-lived path buffer while the clone remains in
the batch. Moving the original changes which allocation remains live, but does not make
the batch smaller.

## Verdict

**Rejected.** Wall and CPU were unchanged while both memory signals regressed.
The production change was reverted.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
