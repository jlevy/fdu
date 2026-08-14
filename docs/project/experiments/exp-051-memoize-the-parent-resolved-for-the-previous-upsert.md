---
title: Memoize the parent resolved for the previous upsert
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-051
  title: Memoize the parent resolved for the previous upsert
  date: "2026-08-14"
  hypotheses:
    - S1
  subject:
    tree_label: meta450k
    tree_root_id: 49fd7bea88ee0b90566f01fb140e86707e2dd6979dd01af97f30cdb13c535aee
    tree_engine_digest: 5b894066affe6de8c7f0ba44d00eccd9bc3b7e5d0cf146786f3ae860f4b0c3bc
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
    trials: 20
    warmups: 3
    interleaved: true
    control: fdu at 855aa2e
    candidate: "same, with a one-slot parent memo in apply_validated"
    control_binary:
      name: control
      sha256: eb7045ba931f314cc5c4c0213b1046176d2b2aab3503420cbae497ac5059822c
      size_bytes: 1443648
      args: []
    candidate_binary:
      name: candidate
      sha256: 96fa343e4a6f15474d2cfc3f0dba21dc9eea7d026d84d8f3a2c5050a6e5cdeb3
      size_bytes: 1444752
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: /tmp/fdu-perf/results/run-s1-parent-memo.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2022071226.5
          candidate_median: 1852852604.5
          change_pct: -7.348
          ci95_low_pct: -10.424
          ci95_high_pct: -6.119
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        component_ns:
          control_median: 913449452.0
          candidate_median: 761980469.5
          change_pct: -17.642
          ci95_low_pct: -19.628
          ci95_high_pct: -12.655
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        cpu_ns:
          control_median: 3051397000.0
          candidate_median: 2846333500.0
          change_pct: -5.403
          ci95_low_pct: -7.956
          ci95_high_pct: -5.11
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        user_cpu_ns:
          control_median: 2107999000.0
          candidate_median: 1932982000.0
          change_pct: -8.826
          ci95_low_pct: -11.162
          ci95_high_pct: -7.427
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        system_cpu_ns:
          control_median: 951296000.0
          candidate_median: 947708000.0
          change_pct: 0.552
          ci95_low_pct: -4.783
          ci95_high_pct: 3.557
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        peak_rss_bytes:
          control_median: 287776768.0
          candidate_median: 281458688.0
          change_pct: -2.914
          ci95_low_pct: -4.139
          ci95_high_pct: -0.171
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1745487784.0
          candidate_median: 1741208508.0
          change_pct: 0.539
          ci95_low_pct: -1.095
          ci95_high_pct: 1.64
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        component_ns:
          control_median: 333875780.0
          candidate_median: 330295322.0
          change_pct: -1.278
          ci95_low_pct: -2.883
          ci95_high_pct: 1.349
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        cpu_ns:
          control_median: 2648961500.0
          candidate_median: 2640344000.0
          change_pct: 0.135
          ci95_low_pct: -0.718
          ci95_high_pct: 1.007
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        user_cpu_ns:
          control_median: 1686757500.0
          candidate_median: 1704267000.0
          change_pct: 1.969
          ci95_low_pct: -0.096
          ci95_high_pct: 3.559
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        system_cpu_ns:
          control_median: 955031500.0
          candidate_median: 952741500.0
          change_pct: -2.022
          ci95_low_pct: -3.371
          ci95_high_pct: 0.189
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        peak_rss_bytes:
          control_median: 199342080.0
          candidate_median: 199464960.0
          change_pct: 0.034
          ci95_low_pct: 0.021
          ci95_high_pct: 0.055
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
  reference_tools: []
  complexity:
    lines_changed: 78
    new_dependencies:
      - "none"
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "One 40-line struct and a single call-site split; the memo is cleared on removal and invalidation. The clear on the kind-change path was removed after tracing showed the memo holds the entry's parent rather than the entry, making it untestable defensive code."
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: -7.348
    reason: "cold-scan-index -7.35% [-10.42%, -6.12%] over 20 interleaved trials, clearing the 3% bar with the whole interval below zero; the index-build component itself fell 16.6%. warm-revalidate is unchanged (+0.54%, interval spans zero). normalize instructions fell 89%, confirming the memo hits."
    commit: null
---
## What was measured

`Index::apply_upsert` resolved every entry’s parent by splitting the path into a
component vector and descending from the root, one `BTreeMap` lookup per level — to
reach a directory the walker was standing in when it produced the record.
A callgrind profile of a single-threaded `scan-index` over a 17,100-entry tree, with the
probe’s oracle digest backed out, put the allocator at about 35% of engine work and
path-component iteration at about 13%, and the caller tree attributed **426,818**
component comparisons to `apply_validated` — roughly 25 per entry.

A walker reports a directory’s children consecutively, because that is the order one
`getdents64` batch hands them over.
Remembering the parent resolved for the previous upsert therefore answers almost every
entry with one path comparison.

## Why the memo is one slot rather than a map

A map would keep entries alive across structural changes and turn every miss into a
hash, where consecutive runs are what the walker actually produces.
One slot captures those runs and costs a path comparison when it misses.

## Result

The realtree harness, 20 trials per variant, interleaved:

| Job | Wall | Component | Verdict |
| --- | ---: | ---: | --- |
| `cold-scan-index` | 2022.1 → 1852.9 ms, **−7.35%** [−10.42%, −6.12%] | 913.4 → 762.0 ms | ACCEPT |
| `warm-revalidate` | 1745.5 → 1741.2 ms, +0.54% [−1.09%, +1.64%] | 333.9 → 330.3 ms | REJECT, no change |

The spike harness agreed independently on a 450k tree: −5.48% [−7.78%, −4.30%] over 40
cold pairs, with RSS 191.2 → 188.2 MB and CPU 5.36 → 5.17 s.

Instruction counts corroborate the mechanism: `normalize` fell from 3,150,123 Ir to
348,492, an 89% reduction, so the memo hits on the large majority of entries.
Total instructions fell 6.0%, or about 9.7% of engine work once the probe’s unchanged
oracle is backed out.

## Where the prediction was wrong

`fdu-ypk2` predicted at least 15% on the cold job, reasoning from the snapshot loader’s
−51.9% for what looked like the same defect.
The prediction was right about the part it described and wrong about which number it
would move: the index-build **component** fell 16.6%, from 913.4 ms to 762.0 ms, while
the **wall** fell 7.35%, because a cold scan also pays for a walk and its syscalls that
this does not touch.
Stating a component prediction against a wall-clock accept rule is the error worth not
repeating.

The remaining gap to the loader’s −51.9% is allocation.
That fix removed a `PathBuf` join, an `Observation` vector, a `normalize` vector and the
descent together; this removes only the last two, and the producer still allocates and
clones a `PathBuf` per entry — 34,256 `Op::clone` calls in the profile.

The bead also proposed a parent-relative observation form,
`Op::UpsertUnder { parent: EntryId, .. }`. That cannot work as stated: `EntryId`s are
allocated by the consumer, so a producer has no id to send.
Reaching the remaining allocation needs a batch-shaped observation — one directory path
with its children — which is filed separately rather than folded in here.

## Warm was not a regression

A first 25-pair warm run reported +3.42% [+1.61%, +9.08%], which reads as a clear
regression and is not one.
At 45 pairs the effect disappeared, running the matchup in both orderings disagreed on
its sign and magnitude (−0.03% and −0.98%, both under 1%), and the realtree harness
independently returned +0.54% with an interval spanning zero.

Warm revalidation walks all 421,690 files and does reach this code, so a regression was
mechanically plausible rather than obviously spurious — which is exactly why it needed
three measurements instead of an argument.
Recorded because the first number was believable and wrong.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
