---
title: "macOS H159 bounded listing recycle is RSS and wall neutral, Linux pending"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-167
  title: "macOS H159 bounded listing recycle is RSS and wall neutral, Linux pending"
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
    candidate: "H159 b1f57ecd: drained detached listings returned to their walker, reuse bounded to DIR_CLAIM spare listings of up to 64 children per worker"
    control_binary:
      name: control
      sha256: 73cd1f226e1f129bc327ef5e1e20dcf6116d3d538fc85472801ec56a8ce1bd5a
      size_bytes: 3148912
      args: []
    candidate_binary:
      name: candidate
      sha256: b8fa4099ccb5f28bbd13deebf50b3528c4c4490953d88b784db69bfaae7c8440
      size_bytes: 3181952
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-167/run.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3256793979.0
          candidate_median: 3292559437.5
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.015
          change_pct: 1.057
          ci95_low_pct: -3.295
          ci95_high_pct: 5.08
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 2951474479.0
          candidate_median: 2990283958.5
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.017
          change_pct: 1.471
          ci95_low_pct: -3.597
          ci95_high_pct: 5.804
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 25874467000.0
          candidate_median: 26084002500.0
          control_p95_over_median: 1.053
          candidate_p95_over_median: 1.071
          change_pct: 2.059
          ci95_low_pct: -4.879
          ci95_high_pct: 7.349
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 645234000.0
          candidate_median: 651413500.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.017
          change_pct: 1.619
          ci95_low_pct: -0.632
          ci95_high_pct: 2.258
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 25229233000.0
          candidate_median: 25432046500.0
          control_p95_over_median: 1.054
          candidate_p95_over_median: 1.072
          change_pct: 2.083
          ci95_low_pct: -4.974
          ci95_high_pct: 7.486
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 72900608.0
          candidate_median: 72327168.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.012
          change_pct: -1.196
          ci95_low_pct: -1.55
          ci95_high_pct: -0.303
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
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
          control_median: 2855291437.5
          candidate_median: 2842405771.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.043
          change_pct: -1.41
          ci95_low_pct: -5.319
          ci95_high_pct: 4.409
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 2849651291.0
          candidate_median: 2836481416.5
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.043
          change_pct: -1.411
          ci95_low_pct: -5.325
          ci95_high_pct: 4.422
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 24173938000.0
          candidate_median: 23983323000.0
          control_p95_over_median: 1.103
          candidate_p95_over_median: 1.083
          change_pct: -0.399
          ci95_low_pct: -8.458
          ci95_high_pct: 5.761
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 346431000.0
          candidate_median: 351759500.0
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.036
          change_pct: 1.8
          ci95_low_pct: -0.862
          ci95_high_pct: 4.011
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 23825965500.0
          candidate_median: 23635366500.0
          control_p95_over_median: 1.104
          candidate_p95_over_median: 1.084
          change_pct: -0.402
          ci95_low_pct: -8.618
          ci95_high_pct: 5.817
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 72884224.0
          candidate_median: 72736768.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.017
          change_pct: -0.056
          ci95_low_pct: -1.334
          ci95_high_pct: 0.178
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
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
      wall_ns_median: 2654230375.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 256
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "about 105 of the 256 diff lines are tests; no dependency, no unsafe, no platform gate"
  verdict:
    decision: in-progress
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -1.41
    reason: "uncontrolled macOS frameworks default-tree -1.41% [-5.32%, +4.41%] and cold-scan-index +1.06% [-3.29%, +5.08%] with peak RSS -0.06% and -1.20%: no macOS regression and the exp-166 RSS cost is gone; the deciding Linux cell pre-registered in fdu-578e has not run"
    commit: b1f57ecd
    kept: candidate
---
## What was predicted

H159: on Linux the index tier’s remaining gap to pdu and diskus is glibc arena
contention, not allocation volume.
The [2026-09-27 comparison](../reports/report-2026-09-27-fdu-linux-tool-comparison.md)
closed the whole gap by running the unchanged binary under `LD_PRELOAD` mimalloc,
jemalloc, or tcmalloc (1.39 → 1.11–1.13 s), and `glibc.malloc.arena_max=1` made it 3.3
s. Its context-switch profile put the index consumer freeing chunks that walker threads
had allocated: each directory’s child list and path.

The change returns each drained listing to the worker that allocated it, as H147’s
`ScannerBatch::recycle` does for the transient tier
([exp-151](exp-151-linux-transient-batch-recycle-clears-3-after-h85-misses-20.md)).
`DetachedIndexBuilder::push_directory` drains a listing in place rather than consuming
it, and the consumer sends the emptied list back through a per-worker channel carried
with each published chunk.
At its next publish the worker takes back what came back, clears anything left in a
listing the consumer did not apply, reuses up to one chunk’s worth (`DIR_CLAIM`) of
listings whose child buffer holds at most 64 children plus one emptied publish list, and
frees the rest on its own thread.
A reused listing’s path is cleared and refilled with the bytes `to_path_buf` would have
copied. Ordering and every listing’s facts are unchanged.
The pool bounds come from
[exp-166](exp-166-macos-h159-unbounded-listing-pool-costs-1-5-peak-rss-wall-fl.md),
where a looser pool cost 1–2 MiB of peak RSS.

Named before measuring, in `fdu-578e`: the deciding cell is Linux.
`default-tree` wall is the primary job, down at least 3% with the interval below zero on
reconstructible `linux-v6.12`, with `linux-balanced-1m` screening; `cold-scan-index` is
also expected to move; the product `fdu-default-tree` contract is paired in the tool
harness; and peak RSS must be non-inferior (upper bound at most +5%). The claimed budget
is a part of the roughly 19% of the 1M indexed-tree wall the allocator screen attributes
to glibc, which is the consumer-side frees of listing buffers (two to three per
directory) and whatever walker-side arena waiting they cause.
The Linux 450k index tier stood at 1.78× the parallel floor against its 1.4× closure
threshold ([exp-141](exp-141-h111-linux-floor-and-rss-gates-fail-on-current-engine.md)).

No Linux host was reachable, so this record is the macOS non-regression screen that the
same unconditional code needs.
On Darwin the system allocator does not lock by owning arena, so no gain was expected;
the question was whether the channel and reuse cost anything.
A median more than 3% worse on either job would be a regression.

## What was measured

Release probes of `56c506e1` (control) and `b1f57ecd` (candidate), 12 interleaved pairs
of `default-tree` and `cold-scan-index`, on internal APFS.

Quiet cells were attempted first on both subjects and did not hold.
The `system-private-frameworks` quiet cell started, but unrelated workloads crossed the
25% busy bar around enough samples that neither job kept 12 valid pairs.
The `rustup-toolchains` quiet cell lost most of its `default-tree` samples the same way.
Neither is recorded.
The uncontrolled `system-private-frameworks` cell recorded here had no invalid samples,
no fingerprint drift, and a before-sample CPU busy median of 17.8% (maximum 40.3%) at a
one-minute load average of 10–12 on ten cores.
It is exploration, not a held-out claim.

- `default-tree`: wall −1.41% [−5.32%, +4.41%]; user CPU +1.80% [−0.86%, +4.01%]; peak
  RSS −0.06% [−1.33%, +0.18%] (69.5 → 69.4 MiB).
- `cold-scan-index`: wall +1.06% [−3.29%, +5.08%]; user CPU +1.62% [−0.63%, +2.26%];
  peak RSS −1.20% [−1.55%, −0.30%].

The uncontrolled `rustup-toolchains` cell, with 3,420 directories among 77,159 entries,
gives H159 little to act on (raw run in
[evidence/exp-167/rustup-run.json](evidence/exp-167/rustup-run.json)):

- `default-tree`: wall −3.21% [−5.27%, −0.69%]; peak RSS +2.77% [−0.96%, +4.89%] (25.8 →
  26.2 MiB).
- `cold-scan-index`: wall +0.07% [−7.73%, +4.78%]; peak RSS +1.28% [−1.66%, +4.30%].

That `default-tree` interval is below zero, but it is an uncontrolled cell on the
subject with the fewest directories, on an allocator the mechanism does not target.
It is not claimed as a macOS win.

One counted, untimed `scan-index` per arm on `system-private-frameworks`
([control](evidence/exp-167/control-scan-index-counters.txt),
[candidate](evidence/exp-167/candidate-scan-index-counters.txt)) confirmed the detached
route (158,704 builder entries) and the reuse: allocations 1,654,218 → 1,582,362
(−4.3%), bytes allocated 309.1 → 290.0 MB (−6.2%). exp-166’s looser pool removed about
the same number (1,580,451), so the bounds gave up almost no reuse.

## Decision

In progress: no macOS regression, Linux pending.
No median on either subject is worse than +1.06%, and the peak RSS cost exp-166 found is
gone.
The wall intervals’ upper bounds (+4.41% and +5.08% on the directory-heavy subject)
are past the 3% margin, so non-inferiority at that margin is not established on this
host, only the absence of a regression.
The candidate stays in the stack pending the Linux cell, which decides H159.

Linux protocol, from `fdu-578e`:

1. Build release `perf_probe` and release `fdu` for `56c506e1` (control) and this commit
   (candidate) with one toolchain.
   Record the build argv and copy each binary outside every measured tree.
2. Deciding subject: reconstructible `linux-v6.12`. Screening subject:
   `linux-balanced-1m`, from
   `python -m benchmarks.generate create --recipe balanced --entries 1000000` with seed
   `fdu-balanced-v1`.
3. Probe:
   `make perf-compare JOBS="default-tree cold-scan-index" TRIALS=12 PERF_HOST_REGIME=quiet`
   on each subject. The primary is `default-tree` wall on `linux-v6.12`, under the accept
   rule: median at least 3% faster, interval entirely below zero, zero invalid samples,
   no drift, and peak RSS non-inferior.
4. Product: `make perf-compare-tools` with `PERF_TOOL_CONTRACT=fdu-default-tree`, the
   control CLI as anchor and the candidate CLI as a `fdu-default-tree` tool, 12 adjacent
   pairs, 3 warm-ups, quiet, on both subjects.
   On `linux-balanced-1m`, add the `fdu` indexed-tree contract, on which the pdu and
   diskus gap was measured.
   These are reported beside the probe verdict and do not replace it.
5. Record the Linux cell as a new experiment, then update H159’s registry row and
   `fdu-578e`. H157’s product-job rerun (`fdu-o6um`) takes the kept arm as its control.
