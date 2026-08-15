---
title: Share the index with the snapshot writer instead of deep-cloning it
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-059
  title: Share the index with the snapshot writer instead of deep-cloning it
  date: "2026-08-15"
  hypotheses:
    - H87
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
    control: "post-exp-058 head: spawn_save deep-clones the index before rendering"
    candidate: "open_with_pending_save returns Arc<Index>; the writer takes a reference"
    control_binary:
      name: control
      sha256: 92832cc119c173ed16a5849a44f9920cebbbd4ddab15e22cb91ebe6496349e8b
      size_bytes: 1881000
      args: []
    candidate_binary:
      name: candidate
      sha256: 1cfd0710715ae4bd30fc59ae3e7ad4dc11d14d157ae4e34649a94f8d862bd43f
      size_bytes: 1882480
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: /tmp/fdu-realtree/results/run-exp059-shared-save-index.json
  results:
    - job: cold-open-save
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2080074936.0
          candidate_median: 1821367744.5
          change_pct: -10.503
          ci95_low_pct: -21.958
          ci95_high_pct: -8.108
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 1078592720.5
          candidate_median: 858620470.0
          change_pct: -17.259
          ci95_low_pct: -35.822
          ci95_high_pct: -12.184
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 2995609000.0
          candidate_median: 2761455000.0
          change_pct: -6.529
          ci95_low_pct: -15.105
          ci95_high_pct: -5.519
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 1839485000.0
          candidate_median: 1717761000.0
          change_pct: -6.088
          ci95_low_pct: -8.341
          ci95_high_pct: -5.486
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 1150021500.0
          candidate_median: 1032045000.0
          change_pct: -6.45
          ci95_low_pct: -24.29
          ci95_high_pct: -2.527
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 516450304.0
          candidate_median: 331880448.0
          change_pct: -35.256
          ci95_low_pct: -38.156
          ci95_high_pct: -33.078
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1636321058.0
          candidate_median: 1658164404.0
          change_pct: 1.689
          ci95_low_pct: -0.857
          ci95_high_pct: 3.072
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        component_ns:
          control_median: 669291748.5
          candidate_median: 687232372.5
          change_pct: 2.509
          ci95_low_pct: -1.125
          ci95_high_pct: 7.778
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        cpu_ns:
          control_median: 2598540500.0
          candidate_median: 2567484500.0
          change_pct: -0.563
          ci95_low_pct: -1.715
          ci95_high_pct: 0.631
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        user_cpu_ns:
          control_median: 1650400000.0
          candidate_median: 1633024500.0
          change_pct: -1.662
          ci95_low_pct: -3.817
          ci95_high_pct: 2.195
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        system_cpu_ns:
          control_median: 948109500.0
          candidate_median: 952382000.0
          change_pct: 1.195
          ci95_low_pct: -2.847
          ci95_high_pct: 4.742
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
        peak_rss_bytes:
          control_median: 279060480.0
          candidate_median: 295071744.0
          change_pct: 3.987
          ci95_low_pct: -6.035
          ci95_high_pct: 10.085
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 45
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - "open_with_pending_save's return type is a breaking change for library consumers of that entry point; open() is unchanged"
    notes: ""
  verdict:
    decision: accepted
    primary_job: cold-open-save
    primary_metric: wall_ns
    change_pct: -10.503
    reason: "cold-open-save wall -10.50% [-21.96%, -8.11%], component -17.26%, peak RSS -35.26% [-38.16%, -33.08%] - the second copy of the index disappearing; cold-scan-index unmoved as the placebo at +1.69% [-0.86%, +3.07%]"
    commit: bd9779d
---
`spawn_save` deep-cloned the whole index — every boxed entry, both stored copies of
every name, and every `BTreeMap` — on the caller’s thread, before rendering could begin,
on every cache-writing run.
The clone bought the writer a view independent of the renderer; sharing one immutable
index buys the same independence for a refcount bump, because both are readers and the
index is read-only from that point.

`open_with_pending_save` now returns `Arc<Index>`. The blocking `open` keeps its
owned-`Index` signature exactly: it already joined the writer before returning, so the
writer’s reference is gone by then and `Arc::into_inner` is infallible.
The watch path in the CLI joins first for the same reason.
No fallback clone is written on either path - one would be untestable defensive code
that quietly reintroduces the copy this change exists to remove.

The peak-RSS result is the mechanism showing itself: -35.26% [-38.16%, -33.08%] is close
to the whole second copy of a 450k-entry index disappearing, which is exactly what a
deep clone was. Component -17.26% and user CPU -6.09% are the caller-thread work that no
longer happens before rendering starts.

`cold-scan-index` is the placebo here and behaved: +1.69% [-0.86%, +3.07%], interval
spanning zero, on a job that writes no cache and therefore never reached this code.

Cost to carry, stated because it is not zero: `open_with_pending_save` is public, so its
return type is a breaking change for a library consumer using that entry point rather
than `open`. At 0.0.1, with the CLI and execution planner as the only in-tree callers,
that is worth a third of the memory and a tenth of the wall on the default first run
against a tree.
