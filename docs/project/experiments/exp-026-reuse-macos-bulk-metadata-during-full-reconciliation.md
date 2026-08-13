---
title: Reuse macOS bulk metadata during full reconciliation
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-026
  title: Reuse macOS bulk metadata during full reconciliation
  date: 2026-08-12
  hypotheses:
    - H53
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
    control: exp-022/025 current code with portable full reconciliation
    candidate: "the existing macOS bulk metadata reader reused by direct, shared, and scoped reconciliation"
    control_binary:
      name: control
      sha256: 52e0b303402ac0eafa11b06013b731126d81bef482acc962cca3ad9fa2ebc879
      size_bytes: 552576
      args: []
    candidate_binary:
      name: candidate
      sha256: 35198f0525f9501b71bd6764362f35723c925a3689b99c587bfbc457da896019
      size_bytes: 569104
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp026-bulk-warm-reconcile-large-final.json
  results:
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 21161477146.0
          candidate_median: 14014334666.5
          change_pct: -34.389
          ci95_low_pct: -37.66
          ci95_high_pct: -29.498
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 18591269375.0
          candidate_median: 11505266333.0
          change_pct: -39.052
          ci95_low_pct: -41.612
          ci95_high_pct: -33.853
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 13628052500.0
          candidate_median: 7659548500.0
          change_pct: -44.064
          ci95_low_pct: -46.409
          ci95_high_pct: -39.944
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        user_cpu_ns:
          control_median: 3171037500.0
          candidate_median: 2844390500.0
          change_pct: -11.111
          ci95_low_pct: -13.816
          ci95_high_pct: -7.492
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        system_cpu_ns:
          control_median: 10421811500.0
          candidate_median: 4814143000.0
          change_pct: -53.974
          ci95_low_pct: -56.289
          ci95_high_pct: -50.384
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        blocked_ns:
          control_median: 7537567937.5
          candidate_median: 6195483791.5
          change_pct: -16.962
          ci95_low_pct: -21.332
          ci95_high_pct: -13.376
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        peak_rss_bytes:
          control_median: 357908480.0
          candidate_median: 358268928.0
          change_pct: 0.064
          ci95_low_pct: -0.034
          ci95_high_pct: 0.181
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 139
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - an unsupported or malformed bulk directory is discarded before application and reread through the portable reconciliation path
      - "warm reconciliation now shares H26 whole-directory staging, so an extremely wide directory delays its entries and temporarily uses memory proportional to that directory"
    notes: macOS-only reuse of the existing 64 KiB reader and audited FFI boundary; reconciliation remains serial and other platforms plus explicit one-worker configurations remain portable
  verdict:
    decision: accepted
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -34.389
    reason: "Warm-open wall improved 34.39% at 720k and 18.97% at 60k; large-tree total CPU fell 44.06% and system CPU 53.97%, RSS was neutral, and all oracle checks passed"
    commit: null
---
# Reuse macOS bulk metadata during full reconciliation

## Hypothesis

H53 followed the current warm profile rather than the earlier cold profile.
Full reconciliation spent 64.94% of samples in the kernel: one `fstatat` per entry was
29.25%, directory open 19.49%, and `getdirentries64` 6.76%. The bounds-audited H26
reader already returns exactly the name, kind, identity, fingerprint, and size contract
reconciliation needs.
Reusing it per directory should remove the per-entry metadata syscall while leaving
index arbitration, scoped reconciliation, and complete-directory portable fallback
unchanged.

## What was tried

On macOS, direct and shared reconciliation now create one reusable 64 KiB bulk reader.
For each directory, a complete successful `getattrlistbulk` result flows through the
existing expectation, unchanged-elision, conditional-upsert, descent, removal, and
batch-flush logic. Any unsupported filesystem, per-entry error, malformed record, mount,
or firmlink discards the uncommitted directory result and reopens it through the
portable path.
Other platforms and explicit one-worker configurations retain the portable
implementation.

The reconciliation loop remains serial.
No worker, lock, dependency, unsafe block, snapshot field, or delta operation was added.
A macOS test mutates an indexed tree with an addition, edit, and deletion, then proves
bulk and portable reconciliation have identical reports, entries, roll-ups, and
extension tallies. The existing direct, shared-handle, subtree, invalidation, depth,
filesystem-boundary, and failure tests also exercise the default bulk path on macOS.

## What the numbers said

On the immutable 720,805-entry cache-pressure subject, full warm-open wall fell 34.39%
[-37.66%, -29.50%] and the reconciliation component fell 39.05%. The candidate removed
work rather than hiding it: total CPU fell 44.06%, system CPU 53.97%, blocked time
16.96%, and involuntary context switches 72.65%. Peak RSS and minor faults were neutral.
Every sample passed the independent oracle and the tree fingerprint was unchanged.

The final run against a freshly fingerprinted immutable 60,067-entry subject improved
warm-open wall 18.97% [-19.62%, -18.37%] and the reconciliation component 27.85%. The
earlier same-binary exploratory run also showed total CPU down 19.48%, system CPU down
27.05%, and RSS unclear; it was not used for the claim because its older reference
fingerprint had timestamp drift, even though the tree remained unchanged during the run.

The post-change profile confirms the predicted transition.
`fstatat` and `getdirentries64` disappeared from warm reconciliation’s top list; batched
`getattrlistbulk` accounts for 16.61% and directory open is now the largest filesystem
residue at 30.27%.

## Verdict

**Accepted.** The change clears the gate at both scales, substantially reduces CPU,
preserves exact reconciliation and delta semantics, and reuses the platform boundary
already justified by exp-022. It also improves the sound full-sweep fallback that
FSEvents must retain and gives future journal-scoped reconciliation the same bulk
verification primitive.
Whole-directory staging remains H26’s known limitation; the large-tree RSS interval
shows no additional retained-memory cost in this path.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
