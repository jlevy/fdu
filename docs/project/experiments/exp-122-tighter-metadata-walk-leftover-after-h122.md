---
title: Tighter metadata walk leftover after H122
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-122
  title: Tighter metadata walk leftover after H122
  date: "2026-09-19"
  hypotheses:
    - H122
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
    control: same probe at 8dd95be8
    candidate: same probe plus dir_enumeration_calls counter
    control_binary:
      name: control
      sha256: 3b6d17a92ce9936c5c64daa410ea8efc21a4c35f85fe7a57362de45f311b43e8
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: 3b6d17a92ce9936c5c64daa410ea8efc21a4c35f85fe7a57362de45f311b43e8
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-122-tighter-walk-profile.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2408155333.5
          candidate_median: 2467714145.5
          control_p95_over_median: 1.389
          candidate_p95_over_median: 1.289
          change_pct: -1.945
          ci95_low_pct: -16.095
          ci95_high_pct: 7.629
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 2400987979.0
          candidate_median: 2461132792.0
          control_p95_over_median: 1.386
          candidate_p95_over_median: 1.288
          change_pct: -1.916
          ci95_low_pct: -16.291
          ci95_high_pct: 7.629
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 16126747500.0
          candidate_median: 14835172500.0
          control_p95_over_median: 1.163
          candidate_p95_over_median: 1.292
          change_pct: -7.5
          ci95_low_pct: -28.836
          ci95_high_pct: 12.7
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 336388000.0
          candidate_median: 328185000.0
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.036
          change_pct: -2.092
          ci95_low_pct: -5.889
          ci95_high_pct: 0.426
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 15797956500.0
          candidate_median: 14506987500.0
          control_p95_over_median: 1.165
          candidate_p95_over_median: 1.298
          change_pct: -7.759
          ci95_low_pct: -29.305
          ci95_high_pct: 13.115
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 88858624.0
          candidate_median: 88842240.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.011
          change_pct: 0.027
          ci95_low_pct: -1.268
          ci95_high_pct: 0.794
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
      wall_ns_median: 2114015042.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 40
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "dir_enumeration_calls kept, off by default; counted only on a successful bulk read"
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -1.945
    reason: tighter leftover is 1.40 getattrlistbulk calls per directory; no userspace cut at 3 percent; H125 not minted
    commit: 8dd95be8
---
## What was predicted

H122 already named the leftover: deciding-scale `default-tree` is a cold walk, and the
named cost inside that walk is directory `__open` plus `getattrlistbulk`, not consume or
finish. A Darwin userspace cut of at least 3% would need a named mechanism inside the
leftover that is not `searchfs` and not a walker rewrite.
If none exists, the next useful cell is a tighter walk profile — counters plus a longer
sample — that tells a later Linux/H111 or `openat` campaign what to cut.

Named before measuring:

- Do not mint H125 as a cut unless a userspace mechanism can be named at ≥3% of wall.
- Add one off-by-default counter: `dir_enumeration_calls`, the macOS bulk backend’s
  `getattrlistbulk` syscalls including the empty terminator, counted only on a complete
  successful directory read (the same rule as `dir_opens`).
- The portable `read_dir` path leaves that counter at 0. Linux multiplicity stays a
  `getdents64` strace fact (the playbook’s 2.00 per directory).
- Attachment: 12-pair same-binary `default-tree` wall, `FDU_COUNTERS` unset.
- Attribution: counters-on `scan-index` / `default-tree`, and a 20 s sample (counters
  and oracle off).
- Quiet first. If the start gate fails, label uncontrolled.
  Do not lower the 25% bar.
- H113: one quiet start only.
  If it refuses, skip.
  Do not run uncontrolled.
- H107: hunt only. Run only on a tree whose ignored share is the walk.
  Do not mutate trading.
  Do not retry metabrowser.

Subject: nominated `system-private-frameworks` (sealed, reconstructible).
Experiment id exp-122. exp-113 remains reserved.

## What was measured

H113 quiet start (this session) refused at CPU busy **43.79% > 25.0%**. No pair.
The file-count shortcut was not compiled.
exp-113 unused.

H107 hunt (this host, no writes into trading):

| Tree | `.gitignore` files | Entries (find cap / probe) | Same `dir_opens` on/off? |
| --- | ---: | --- | --- |
| rustup-toolchains | 0 | nominated 77,132 | n/a (no controls) |
| system-private-frameworks | 0 | nominated 158,705 | n/a (0 control reads) |
| cargo-registry-src | 161 | ~22k, screening only | not deciding-scale |
| metabrowser-clone | 52 | nominated 145,931 | exp-106 refute |
| tbd checkout | 145 | 34,008 / opens 4,876 | yes, 4,876 = 4,876 |
| urollup checkout | 80 | 97,155 / opens 8,599 | yes, 8,599 = 8,599 |
| squares / convex-backend | 141 / 116 | 109k / 8k | no on-disk `node_modules` |

`should_descend` does not consult ignore state.
The scan still enumerates ignored subtrees so every row can show an ignored share.
A large ignored `node_modules` cannot make ignore “the walk” on this engine: it adds
matching and control reads on the same opens.
Constructing a wrapper `.gitignore` would not skip descent either.
No ignore-is-the-walk subject.
H107 not started.

H123 product follow-through: none.
`query::report` / `Index.report()` is already the opened and refresh path.
No smallest fdu-core change.
CLI invents nothing.

Same release probe both variants (`3b6d17a9…`, 2,536,992 bytes).
Job: harness `default-tree` after one snapshot seed per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.

Quiet was attempted.
The start gate refused at 30.3% CPU busy.
The pair ran as **uncontrolled**. Initial busy 49.15%; final 100.0%. Thermal `normal`.
The 25% bar was not lowered.
No RAM disk. Tree fingerprint matched the nominated digest (`0c863b0a…`) and did not
move.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 2,408.2 ms | 2,401.0 ms | 84.7 MiB |
| candidate | 2,467.7 ms | 2,461.1 ms | 84.7 MiB |

Self-comparison wall −1.95% [−16.09%, +7.63%]. Dust reference wall 2,114.0 ms.
0 invalid samples.

Counters-on attribution (same binary, not the verdict):

| Run | Component | Walk | Finish | Opens | Enum calls | Enum / open |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `scan-index` first | 2,000.8 ms | 1,991.4 ms | 8.7 ms | 55,256 | 77,509 | 1.403 |
| `scan-index` second | 1,643.1 ms | 1,635.8 ms | 6.6 ms | 55,256 | 77,509 | 1.403 |
| `default-tree` first (wrote snapshot) | 1,648.7 ms | 1,521.1 ms | 17.1 ms | 55,256 | 77,509 | 1.403 |
| `default-tree` second (byte-identical) | 1,536.6 ms | 1,480.3 ms | 13.7 ms | 55,256 | 77,509 | 1.403 |

Walk / component on the second `default-tree` is 96.33%, same shape as exp-118. Finish
stays 0.3–1.0%. 0 control reads.
158,704 detached entries.

A 20-second `/usr/bin/sample` on the profiling build (`--repeat 15`, counters and oracle
off, 165,807 stacks):

| Layer / symbol | Share |
| --- | ---: |
| kernel/syscall | 85.03% |
| `__open` | 50.36% |
| `getattrlistbulk` | 18.95% |
| lock waits (`__psynch_mutexwait` + `semaphore_wait_trap` + drop + cv) | 18.00% |
| `fdu::scan` | 3.02% |
| allocator | 3.00% |
| `walk_detached_worker` | 0.94% |
| `macos_bulk::Reader::read` | 0.61% |
| `run_concurrent_walk` | 0.35% |
| `fdu::index` | 0.41% |
| path | 0.10% |

Profile artifact: `/tmp/fdu-realtree/results/profile-exp-122-tighter-walk.json`.

## What the determination said

No userspace mechanism is ≥3% of wall.
`fdu::scan` as a whole is 3.02%; its largest symbol is 0.94%. Allocator as a whole is
3.00%; that is not a new named cut (H114 already failed).
H125 was not minted.

The leftover a later campaign can cut is still kernel:

- one `File::open` per directory (55,256) — the person-gated `openat` leftover
- 1.403 `getattrlistbulk` calls per directory including EOF (77,509 / 55,256)

Linux H111 should compare those counts to `getdents64` (playbook: 2.00 per directory)
plus per-entry `statx`, not hunt a Darwin userspace trim.
H55 already rejected a larger bulk buffer.

## Judgment

The tighter profile is the record, not a patch.
`dir_enumeration_calls` is kept (off by default, counted only on a successful bulk
read). Do not invent a Darwin consume, Path, or allocator cut from the 3% module totals.
Do not load a snapshot on `fdu PATH`. Do not start H111 on this host.
Do not start H107 without an ignore-is-the-walk subject; this hunt closed that for the
nominated set and for the large local checkouts that were legal to inspect.

No README 200K / 4M change.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
