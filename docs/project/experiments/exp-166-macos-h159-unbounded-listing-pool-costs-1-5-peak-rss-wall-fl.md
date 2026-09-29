---
title: "macOS H159 unbounded listing pool costs 1-5% peak RSS, wall flat"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-166
  title: "macOS H159 unbounded listing pool costs 1-5% peak RSS, wall flat"
  date: "2026-09-28"
  hypotheses:
    - H159
  subject:
    tree_label: system-private-frameworks
    tree_root_id: b718281f3051a0ed5b4fc59d83614845f67e17999095cf2d837a0c551e24869c
    tree_engine_digest: 0c863b0ab28dc47e3db5a0298fe3239a51959056ec5b97c519e49ad1bfd965bf
    tree_provenance: "The sealed macOS system volume's private frameworks, read-only and identical on every Mac running the same OS build (Darwin 25.5.0 here). Reconstructible by installing that build."
    tree_reconstructible: true
    tree_entries: 158705
    tree_directories: 55256
    tree_files: 96542
    tree_symlinks: 6907
    tree_apparent_bytes: 5752378316
    tree_allocated_bytes: 3910119424
    tree_max_depth: 14
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
    control: "56c506e1 probe (stack-141 top, engine a5c0ab46)"
    candidate: "H159 first build 666b51f0: drained detached listings returned to their walker, up to 16 spare listings of up to 256 children per worker"
    control_binary:
      name: control
      sha256: 73cd1f226e1f129bc327ef5e1e20dcf6116d3d538fc85472801ec56a8ce1bd5a
      size_bytes: 3148912
      args: []
    candidate_binary:
      name: candidate
      sha256: 85b9d465d98727ffb5a9d69ff69b13a6c94f864cc49c8ed1a7d70642d5ba335e
      size_bytes: 3181952
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-166/run.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3131850396.0
          candidate_median: 3091984292.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.077
          change_pct: 2.399
          ci95_low_pct: -4.426
          ci95_high_pct: 8.793
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 2826956292.0
          candidate_median: 2782667437.5
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.087
          change_pct: 3.389
          ci95_low_pct: -4.958
          ci95_high_pct: 9.546
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 23771520500.0
          candidate_median: 23935383500.0
          control_p95_over_median: 1.097
          candidate_p95_over_median: 1.147
          change_pct: 7.189
          ci95_low_pct: -8.297
          ci95_high_pct: 16.095
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 638685500.0
          candidate_median: 643855000.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.014
          change_pct: 1.262
          ci95_low_pct: -0.4
          ci95_high_pct: 3.498
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 23128733000.0
          candidate_median: 23292854000.0
          control_p95_over_median: 1.1
          candidate_p95_over_median: 1.151
          change_pct: 7.344
          ci95_low_pct: -8.503
          ci95_high_pct: 16.49
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 72556544.0
          candidate_median: 73883648.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.013
          change_pct: 1.54
          ci95_low_pct: 0.515
          ci95_high_pct: 2.994
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2862517979.0
          candidate_median: 2867216729.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.059
          change_pct: 1.318
          ci95_low_pct: -5.097
          ci95_high_pct: 5.197
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 2856782125.0
          candidate_median: 2861072916.5
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.059
          change_pct: 1.3
          ci95_low_pct: -5.079
          ci95_high_pct: 5.189
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 24070691000.0
          candidate_median: 23883453000.0
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.133
          change_pct: 4.333
          ci95_low_pct: -8.315
          ci95_high_pct: 8.258
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 349051500.0
          candidate_median: 350744500.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.032
          change_pct: 2.816
          ci95_low_pct: -0.609
          ci95_high_pct: 3.254
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 23715541500.0
          candidate_median: 23531793500.0
          control_p95_over_median: 1.08
          candidate_p95_over_median: 1.135
          change_pct: 4.358
          ci95_low_pct: -8.447
          ci95_high_pct: 8.346
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 73457664.0
          candidate_median: 74522624.0
          control_p95_over_median: 1.014
          candidate_p95_over_median: 1.014
          change_pct: 1.426
          ci95_low_pct: 1.124
          ci95_high_pct: 1.79
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
  reference_tools:
    - name: dust
      wall_ns_median: 2497256187.5
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 253
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "about 105 of the 253 diff lines are tests; no dependency, no unsafe, no platform gate"
  verdict:
    decision: superseded
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 1.318
    reason: "uncontrolled macOS frameworks default-tree wall +1.32% [-5.10%, +5.20%] with peak RSS +1.43% [+1.12%, +1.79%], and rustup default-tree peak RSS +5.04% [+2.89%, +11.52%] past the pre-registered 5% margin; superseded by the bounded pool of exp-167"
    commit: 666b51f0
    kept: control
---
## What was predicted

H159: on Linux the index tier’s remaining gap to pdu and diskus is glibc arena
contention, not allocation volume.
The [2026-09-27 comparison](../reports/report-2026-09-27-fdu-linux-tool-comparison.md)
closed the whole gap by running the unchanged binary under `LD_PRELOAD` mimalloc,
jemalloc, or tcmalloc (1.39 → 1.11–1.13 s), and `glibc.malloc.arena_max=1` made it 3.3
s. Its context-switch profile put the index consumer freeing chunks that walker threads
had allocated: each directory’s child list and path.

This first build returns each drained listing to the worker that allocated it, as H147’s
`ScannerBatch::recycle` does for the transient tier
([exp-151](exp-151-linux-transient-batch-recycle-clears-3-after-h85-misses-20.md)).
`DetachedIndexBuilder::push_directory` drains a listing in place, and the consumer sends
the emptied list back through a per-worker channel carried with each published chunk.
At its next publish the worker takes back what came back, and it kept up to 16 spare
listings (4 × `DIR_CLAIM`) whose child buffer held at most 256 children.

Named before measuring, in `fdu-578e`: the deciding cell is Linux, with `default-tree`
wall as the primary job and peak RSS non-inferior (upper bound at most +5%). No Linux
host was reachable, so this was the macOS non-regression screen of the same
unconditional code; a median more than 3% worse would be a regression.

## What was measured

Release probes of `56c506e1` (control) and `666b51f0` (candidate), 12 interleaved pairs
of `default-tree` and `cold-scan-index`, on two nominated subjects on internal APFS.

The quiet cell was attempted first on `system-private-frameworks`. It started at 22.1%
CPU busy, but other workloads raised the host to 71% and 27 of its 60 samples were
invalidated, so it is not recorded.
The uncontrolled cell recorded here had no invalid samples, no fingerprint drift, and a
before-sample CPU busy median of 17.6% (maximum 44.9%) at a one-minute load average of
8.9–12.9 on ten cores.

- `default-tree`: wall +1.32% [−5.10%, +5.20%]; peak RSS **+1.43% [+1.12%, +1.79%]**
  (70.1 → 71.1 MiB).
- `cold-scan-index`: wall +2.40% [−4.43%, +8.79%]; peak RSS **+1.54% [+0.52%, +2.99%]**.

`rustup-toolchains` (uncontrolled, quiet not attempted on this build; raw run in
[evidence/exp-166/rustup-run.json](evidence/exp-166/rustup-run.json)):

- `default-tree`: wall −0.80% [−8.10%, +6.91%]; peak RSS **+5.04% [+2.89%, +11.52%]**
  (25.1 → 27.4 MiB).
- `cold-scan-index`: wall −2.66% [−4.46%, +0.97%]; peak RSS +3.29% [−0.90%, +6.68%].

One counted, untimed `scan-index` per arm on `system-private-frameworks`
([control](evidence/exp-166/control-scan-index-counters.txt),
[candidate](evidence/exp-166/candidate-scan-index-counters.txt)) confirmed the detached
route (158,704 builder entries) and the reuse: allocations 1,654,218 → 1,580,451, bytes
allocated 309.1 → 290.1 MB.

## Decision

Superseded by
[exp-167](exp-167-macos-h159-bounded-listing-recycle-is-rss-and-wall-neutral-l.md).
Wall was flat on both subjects, but peak RSS rose by a constant 1–2 MiB, which on the
77,159-entry subject breaches the 5% non-inferiority margin `fdu-578e` pre-registered.
A retained spare listing is memory the consumer can no longer reuse for the index, and
this build could pin up to 16 listings of up to 20 KiB each per worker.
The worker takes returned listings back once per chunk, so one chunk’s worth is all it
can reuse; `b1f57ecd` bounds the pool to that and to 64 children per listing.
Everything past either bound is still freed on the worker that allocated it, so the
cross-thread frees H159 removes stay removed.
This build is not in the product.
