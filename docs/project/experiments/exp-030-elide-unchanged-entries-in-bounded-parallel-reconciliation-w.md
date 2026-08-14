---
title: Elide unchanged entries in bounded parallel reconciliation waves
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-030
  title: Elide unchanged entries in bounded parallel reconciliation waves
  date: "2026-08-12"
  hypotheses:
    - H12
    - H9
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
    control: exp-026 serial bulk-backed full reconciliation
    candidate: four-worker bounded immutable-baseline waves with producer-side no-op elision
    control_binary:
      name: control
      sha256: 35198f0525f9501b71bd6764362f35723c925a3689b99c587bfbc457da896019
      size_bytes: 569104
      args: []
    candidate_binary:
      name: candidate
      sha256: 54db14278796b5ab1233ed71eefe07e2061c3913a957ac8e4f5fa79a8a4c2765
      size_bytes: 585680
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-exp030-bounded-parallel-reconcile-large-exact.json
  results:
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 14463441479.0
          candidate_median: 5708114437.5
          change_pct: -59.53
          ci95_low_pct: -62.799
          ci95_high_pct: -50.433
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        component_ns:
          control_median: 11864471687.5
          candidate_median: 3189117937.5
          change_pct: -72.547
          ci95_low_pct: -75.606
          ci95_high_pct: -66.648
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
        cpu_ns:
          control_median: 8001670500.0
          candidate_median: 9258738000.0
          change_pct: 17.301
          ci95_low_pct: 5.938
          ci95_high_pct: 30.616
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        user_cpu_ns:
          control_median: 2900203000.0
          candidate_median: 3072327500.0
          change_pct: 6.931
          ci95_low_pct: 1.954
          ci95_high_pct: 12.932
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        system_cpu_ns:
          control_median: 5094090000.0
          candidate_median: 6186410500.0
          change_pct: 23.682
          ci95_low_pct: 8.495
          ci95_high_pct: 40.335
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 12
        peak_rss_bytes:
          control_median: 358211584.0
          candidate_median: 354623488.0
          change_pct: -0.988
          ci95_low_pct: -1.145
          ci95_high_pct: -0.905
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 12
  reference_tools: []
  complexity:
    lines_changed: 419
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes:
      - a panicking reconciliation worker makes the sweep partial and reports the missing work
      - a wave exceeding the bounded deferred-change budget repeats the full tree through the serial reconciler
      - exclusive full reconciliation now delays effective deltas until the current bounded directory wave joins
    notes: "407 insertions and 12 deletions; no dependency or unsafe change; direct full-tree path only, while shared, scoped, and explicit one-worker reconciliation retain the serial reference"
  verdict:
    decision: accepted
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -59.53
    reason: "Warm-open wall improved 30.25% at 60k and 59.53% at 720k; reconciliation component improved 50.31% and 72.55%, exact oracles passed, and RSS stayed within the preregistered bound"
    commit: null
---
# Elide unchanged entries in bounded parallel reconciliation waves

## Hypothesis

H12 revisited exp-002 after the intervening reconciliation changes.
The rejected 2026 parallel sweep still sent every unchanged entry through one mutable
index consumer. H14 later made an exclusive no-op decidable from captured child state,
and H53 made a directory’s complete stat-tier metadata available in one audited macOS
bulk read. Workers should therefore compare filesystem entries with one immutable index
baseline, discard exact matches at the producer, and send only effective changes through
the delta contract after each bounded wave.

The pre-registered 60k gate required warm-open wall to improve at least 15% with a
confidence interval below zero, reconciliation component time to improve at least 25%,
exact oracle parity, and no more than 10% additional RSS. A 720k confirmation would run
only after that gate passed.

## What was tried

Exclusive full-tree reconciliation now takes bounded waves from the same region-aware
breadth-first frontier as the cold walker.
Scoped workers hold an immutable index view, read different directory claims, compare
complete state in place, and retain only upserts or removals that can change the index.
After the workers join, those operations are put in deterministic causal order and
applied in the configured batch size through ordinary observations before the next wave
begins. Delta delivery therefore remains progressive and no mutation bypasses
`Index::apply`.

The deferred change set is capped at the existing maximum observation size.
Overflow discards the unapplied wave and repeats through the incremental serial
reconciler, so a high-churn or adversarially wide tree costs time rather than unbounded
memory. Shared reconciliation retains its conditional serial path because readers and
other producers can change its ABA baseline between lock boundaries.
Explicit one-worker and scoped reconciliation also remain on the reference path.

An exploratory 60k curve found four workers faster than two, six, or eight.
Four beat six by 6.85% while six used 45.92% more total CPU; at 720k six was an unclear
4.60% faster with an interval crossing zero.
Automatic reconciliation therefore stops at four while explicit caller thread settings
remain honored. Tests compare parallel and serial mutation results on every platform and
force the deferred-budget fallback to prove that no partial wave is applied.

## What the numbers said

On the exact 60,067-entry APFS gate, warm-open wall fell 30.25% [-32.11%, -28.41%] and
the reconciliation component fell 50.31%. Total CPU rose 56.06% and system CPU 94.19%
because four metadata readers run concurrently, but peak RSS rose only 3.29%, within the
pre-registered bound.
Every sample passed the independent oracle and the tree fingerprint remained unchanged.

On the 720,805-entry cache-pressure confirmation, warm-open wall fell 59.53%
[-62.80%, -50.43%] and the reconciliation component fell 72.55%. Blocked time fell to
zero, total CPU rose 17.30%, and peak RSS improved 0.99%. A noisy middle interval raised
both variants and was retained; the paired confidence interval still stayed more than
50% below zero.

The post-change 60k profile shows the intended shift: kernel/syscall samples fell from
64.94% after H14 to 27.76%, while `open` and `getattrlistbulk` remain the largest named
costs at 16.47% and 15.76%. Scoped-thread startup and waiting are now visible residue,
which makes wave-size amortization a separate follow-up rather than a reason to reject
the structural win.

## Verdict

**Accepted.** H12 clears every pre-registered gate at 60k and scales to a much larger
win at 720k. It composes producer-side no-op elision with the current BFS scheduler and
bulk metadata backend without weakening full verification, delta-only mutation,
progressive delivery, shared arbitration, or bounded memory.
Warm open is now about 351 ms versus roughly 296 ms for a cold index on the 60k subject;
snapshot load, not full reconciliation, is most of the remaining gap.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
