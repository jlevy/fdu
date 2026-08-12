---
title: Batch macOS scan metadata with getattrlistbulk
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-022
  title: Batch macOS scan metadata with getattrlistbulk
  date: 2026-08-12
  hypotheses:
    - H3
    - H26
  subject:
    tree_label: cache-pressure-12x
    tree_root_id: ffd40fd8482e8ed64bd19bcd1a724389532ca4889be43adf830122279ac63180
    tree_engine_digest: f2909250591b9b64d98956b0b2d8a9c3bd588b4c23f046a4660f3f174173dc23
    tree_entries: 720805
    tree_directories: 88201
    tree_files: 632340
    tree_symlinks: 264
    tree_apparent_bytes: 13021004064
    tree_allocated_bytes: 14760886272
    tree_max_depth: 20
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
    control: "portable read_dir plus one metadata syscall per entry, with adaptive workers"
    candidate: "macOS getattrlistbulk names and complete stat-tier metadata, with portable directory fallback"
    control_binary:
      name: control
      sha256: 9e583fc7877f9b7a64a81787a7d6b5a0407225936ffae26079b8efe2b2a99db7
      size_bytes: 552512
      args: []
    candidate_binary:
      name: candidate
      sha256: 52e0b303402ac0eafa11b06013b731126d81bef482acc962cca3ad9fa2ebc879
      size_bytes: 552576
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp022-getattrlistbulk-large-final.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 6478719645.5
          candidate_median: 4537002458.5
          change_pct: -30.132
          ci95_low_pct: -32.186
          ci95_high_pct: -25.106
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 4990039000.0
          candidate_median: 3077668729.0
          change_pct: -38.976
          ci95_low_pct: -44.88
          ci95_high_pct: -31.766
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 29818063000.0
          candidate_median: 15046679500.0
          change_pct: -43.564
          ci95_low_pct: -60.045
          ci95_high_pct: -34.03
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 3371125000.0
          candidate_median: 2688851500.0
          change_pct: -19.602
          ci95_low_pct: -21.062
          ci95_high_pct: -17.737
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 26468566500.0
          candidate_median: 12039086000.0
          change_pct: -46.617
          ci95_low_pct: -65.749
          ci95_high_pct: -36.627
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
          control_median: 330539008.0
          candidate_median: 332890112.0
          change_pct: 2.715
          ci95_low_pct: -1.416
          ci95_high_pct: 37.002
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
          control_median: 11288842458.5
          candidate_median: 6705811354.0
          change_pct: -41.604
          ci95_low_pct: -43.944
          ci95_high_pct: -36.827
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 4822370208.5
          candidate_median: 2629104729.5
          change_pct: -46.997
          ci95_low_pct: -52.016
          ci95_high_pct: -38.73
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 64393398000.0
          candidate_median: 22887055000.0
          change_pct: -59.031
          ci95_low_pct: -67.205
          ci95_high_pct: -52.781
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 4562182500.0
          candidate_median: 3006296000.0
          change_pct: -33.611
          ci95_low_pct: -34.821
          ci95_high_pct: -32.769
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 59716695000.0
          candidate_median: 19853548500.0
          change_pct: -61.401
          ci95_low_pct: -69.632
          ci95_high_pct: -54.221
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
          control_median: 332013568.0
          candidate_median: 327852032.0
          change_pct: -1.058
          ci95_low_pct: -2.496
          ci95_high_pct: -0.624
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 542
    new_dependencies:
      - "libc 0.2.189 (macOS-only direct dependency, already locked transitively)"
    new_unsafe_blocks: 1
    new_failure_modes:
      - a malformed or unsupported bulk response causes that directory to be discarded and reread through the portable backend
      - "one worker buffers a complete directory before publishing it, increasing peak memory and delaying progress within extremely wide directories"
    notes: "macOS-only accelerator; one bounds-audited FFI call, per-record returned-attribute validation, mount/firmlink-boundary fallback, 64 KiB buffer per worker, and unchanged portable implementation elsewhere"
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -30.132
    reason: "Current code improved 720k end-to-end wall 30.13% and producer wall 41.60%, while the separate 60k run improved them 5.22% and 9.25%; all oracle checks passed and CPU fell at both scales"
    commit: null
---
# Batch macOS scan metadata with getattrlistbulk

## Hypothesis

H3/H26 predicted the fresh post-adaptive profile's result: 67.20% of cold-index samples
were in the kernel. One directory open accounted for 29.74%, one `fstatat` per entry for
20.08%, and directory enumeration for another 9.29%; index code was only 1.35%. macOS
`getattrlistbulk` returns names, type, identity, both fingerprint times, and both size
measures for many entries in one call. Replacing `read_dir` plus per-entry metadata
calls on macOS should therefore reduce cold producer wall and system CPU enough to move
end-to-end wall at both the original and cache-pressure scales.

## What was tried

The concurrent macOS producer opens each claimed directory and fills a reusable 64 KiB
buffer with `getattrlistbulk`. A target-specific parser checks the kernel's per-entry
returned-attribute bitmap, record length, every fixed-width read, the relative name
offset and length, the terminal NUL, and path-component validity before constructing an
observation. It requests exactly fdu's existing metadata contract: name, device, vnode
type, mtime, ctime, file id, logical size, and allocated size.

The accelerator does not weaken the portable path. If the call fails, an entry reports
an error, required fields are absent, a record is malformed, or a containing directory
includes a mount, trigger, or firmlink boundary whose bulk metadata differs from
`stat`, the worker discards that directory's uncommitted bulk results and reopens it
through `read_dir`. Other platforms and explicit one-worker scans retain the portable
implementation. One macOS-only direct dependency exposes the system call: `libc`
0.2.189 was already locked transitively, more than fourteen days old, and covered by
the repository's provenance, license, and advisory checks. The single unsafe block is
the FFI call; parsing uses bounds-checked byte copies rather than pointer dereferences.

Unit coverage compares bulk and portable results exactly for regular files,
directories, symlinks, allocated bytes, fingerprints, device/inode identity, and a
decomposed Unicode name. A synthetic valid record pins the layout and signed `dev_t`
conversion, every possible truncation is rejected, and missing or unexpected attribute
sets, an escaping name offset, and the firmlink flag fail closed. The existing
serial-versus-parallel tests then exercise the bulk backend against the portable
reference over complete trees.

## What the numbers said

On the immutable 720,805-entry cache-pressure subject, end-to-end cold-index wall fell
30.13% [−32.19%, −25.11%] and its scan component fell 38.98%. Producer wall fell
41.60% [−43.94%, −36.83%] and its component fell 47.00%. This removed work instead of
merely overlapping it: end-to-end total CPU fell 43.56% and system CPU 46.62%; producer
total CPU fell 59.03% and system CPU 61.40%. Every measured sample passed the
independent oracle and the before/after tree fingerprints matched. One indexed ordinal
captured an abrupt host-load spike against the candidate; it remains in the run, and
the paired interval still clears the gate.

The separate current-code run on the immutable 60,067-entry subject also cleared the
gate: end-to-end wall fell 5.22% [−7.99%, −3.69%], producer wall 9.25%
[−11.83%, −7.71%], and total CPU 7.37% and 9.74%, respectively. The price is small but
visible at that scale: index peak RSS rose 1.90%, while the producer RSS interval
included zero. On the large producer job RSS instead fell 1.06%; the large index
interval was too wide to interpret under host load.

The post-change profile shows the intended phase change. Per-entry `fstatat` and
`getdirentries64` disappeared from the cold top list; opening directories is now 33.86%
of samples and `getattrlistbulk` itself 25.93%. That makes dirfd-relative open the next
measured cold-scan hypothesis rather than another index micro-optimization.

## Verdict

**Accepted.** The speedup is large at both tested scales, composes with adaptive worker
depth, and substantially reduces CPU as well as latency. That justifies the narrow
platform backend, direct binding, audited unsafe call, and sub-megabyte small-tree RSS
cost. The known limitation is whole-directory staging: an extremely wide directory
delays its observations and can temporarily use memory proportional to its entries.
Any future streaming rewrite must preserve the current all-or-nothing fallback, because
falling back after publishing part of a directory would duplicate observations.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
