---
title: Content roll-up lookup and indexed type-rule tiers
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-064
  title: Content roll-up lookup and indexed type-rule tiers
  date: "2026-08-21"
  hypotheses:
    - H94
    - H95
  subject:
    tree_label: spike-15977
    tree_root_id: 15fd30c5887b80cca0244ab1911b73f4ecf9c9d04d3cca889d81ce519a81e83c
    tree_provenance: "python3 explorations/benchmarks/spikes/gen_tree.py <root> 17000"
    tree_reconstructible: true
    tree_engine_digest: 9c17100d19a64045a6ef02b7de83abd5f05c159203300d330171b13f215126af
    tree_entries: 17041
    tree_directories: 1045
    tree_files: 15977
    tree_symlinks: 19
    tree_apparent_bytes: 595728806
    tree_allocated_bytes: 26341376
    tree_max_depth: 16
    tree_mutated_during_run: false
    host_cpu: Linux
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 0
    host_system: Linux 6.18.44-fc-v21
    filesystem: ""
    os_cache: warm-steady
  method:
    trials: 24
    warmups: 3
    interleaved: true
    control: origin/main at 703ceac
    candidate: HashMap rollups plus LazyLock-indexed name and extension tiers
    control_binary:
      name: control
      sha256: 003d9e855ded3f313a1d3de828030c6d6dfc1d76e6f8b7491da67bd868622595
      size_bytes: 1954064
      args: []
    candidate_binary:
      name: candidate
      sha256: b9d9a0c1311695de34aca55d144f1f38761ede81235877adabe7fbb3a7ae4211
      size_bytes: 1949528
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-h94-h95-content.json
  results:
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 518917865.5
          candidate_median: 450013361.0
          change_pct: -13.401
          ci95_low_pct: -14.736
          ci95_high_pct: -10.921
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        component_ns:
          control_median: 427894756.5
          candidate_median: 359573731.5
          change_pct: -16.546
          ci95_low_pct: -17.639
          ci95_high_pct: -14.562
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        cpu_ns:
          control_median: 957148000.0
          candidate_median: 878589000.0
          change_pct: -8.588
          ci95_low_pct: -9.881
          ci95_high_pct: -6.777
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        user_cpu_ns:
          control_median: 564871500.0
          candidate_median: 492575000.0
          change_pct: -14.094
          ci95_low_pct: -17.145
          ci95_high_pct: -12.43
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        system_cpu_ns:
          control_median: 401854000.0
          candidate_median: 398546000.0
          change_pct: 1.437
          ci95_low_pct: -4.282
          ci95_high_pct: 6.647
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 24
        peak_rss_bytes:
          control_median: 34803712.0
          candidate_median: 34959360.0
          change_pct: 0.424
          ci95_low_pct: -0.142
          ci95_high_pct: 0.73
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 24
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: inconclusive
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 450421204.5
          candidate_median: 314671328.0
          change_pct: -30.307
          ci95_low_pct: -30.692
          ci95_high_pct: -29.607
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        component_ns:
          control_median: 404085055.5
          candidate_median: 267636858.0
          change_pct: -33.729
          ci95_low_pct: -34.277
          ci95_high_pct: -33.091
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        cpu_ns:
          control_median: 449644500.0
          candidate_median: 314117500.0
          change_pct: -30.178
          ci95_low_pct: -30.749
          ci95_high_pct: -29.551
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        user_cpu_ns:
          control_median: 419587500.0
          candidate_median: 283622500.0
          change_pct: -32.51
          ci95_low_pct: -34.106
          ci95_high_pct: -31.105
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        system_cpu_ns:
          control_median: 36072000.0
          candidate_median: 32070000.0
          change_pct: 0.229
          ci95_low_pct: -15.801
          ci95_high_pct: 21.23
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 24
        blocked_ns:
          control_median: 539080.5
          candidate_median: 535013.5
          change_pct: 2.17
          ci95_low_pct: -14.14
          ci95_high_pct: 8.664
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 24
        peak_rss_bytes:
          control_median: 44144640.0
          candidate_median: 44316672.0
          change_pct: 0.372
          ci95_low_pct: 0.241
          ci95_high_pct: 0.482
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 24
      qualification:
        campaign_stage: exploratory
        classification: superior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 78
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "Two LazyLock hash tables and one map key-type change; no new dependency, no unsafe, no threading, no new failure mode. The max_by_key last-wins tie-break is the one subtlety and is pinned by a test."
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -30.307
    reason: "Cumulative -30.31% [-30.69%, -29.61%] on content-cache-hit and -13.40% on content-basic, both intervals well below zero, RSS neutral, mechanism confirmed by caller-tree profile rather than inferred from a flat one. H95 cold-path transfer prediction dropped: -2.34% [-5.05%, -0.64%] at 40 pairs, below the bar. Scope, established afterwards by exp-065: the warm figure this verdict rests on transfers to a dense real tree at -25.78%; the -13.40% on content-basic does not, reading -2.38% there, because this subject is depth 16 and 22.6x sparse. Read the cold number as evidence about this tree, not about the content tier."
    commit: 9fb6a33
---
## What was measured

Two changes to the content tier, measured cumulatively against `origin/main` by the
real-tree harness, and individually against each other by
`benchmarks/spikes/paired_runner.py`.

- **H94** (`fdu-cq7t`): `ContentIndex::rollups` becomes a `HashMap`, and the ancestor
  walk stops allocating a `PathBuf` per ancestor per file on the common hit.
- **H95** (`fdu-9dcj`): the exact-name and extension tiers of the type cascade resolve
  through two `LazyLock` hash tables instead of a non-short-circuiting scan over all 65
  rules and 167 extension strings.

## Why this experiment exists at all: the flat profile was read wrong

`fdu-926e` recorded classification as “about 34% of a warm content open” and was the P0
target of the queue on that basis.
That number came from a flat callgrind profile, where `std::path::compare_components`
sums to about 36% of instructions.

The caller tree disagrees, and the caller tree is right:

| Caller edge into `compare_components` | Ir | calls |
| --- | --- | --- |
| `ContentIndex::merge_ancestors` | 1,256,661,348 (36.30%) | 2,028,997 |
| `apply_analysis` → `BTreeMap<PathBuf, FileAnalysis>::remove` | 383,869,618 (11.09%) | 529,036 |
| `classify::classify_path_with_prefix` | 51,198,615 (1.48%) | 2,266,470 |
| `classify::with_flags` | 53,016,939 (1.53%) | 510,591 |

Classification was **11.11% inclusive**, not 34%. The “~96 comparisons per file” in the
bead was right about the count and wrong about the owner: it is roughly 8 ancestors ×
log2(1,045 directories), not a scan of the rules table.
The harness oracle (`perf_probe::summarize_index`, 9.42%) is excluded from every figure
quoted here.

This is the second time this campaign has been sent at the wrong function by a flat
profile — the first was the `BTreeMap` in `load_content_cache`, measured at 0.9% and
worth −3.0%. Both times the flat view named a std library function and the caller tree
named the owner.

## Results

Real-tree harness, 24 interleaved trials per job, 15,977-file / 1,045-directory
generated tree, tree fingerprint bound to the run.

The subject is `python3 explorations/benchmarks/spikes/gen_tree.py <root> 17000` (seed
42 is hard-coded), and two of its properties decide how far these percentages travel —
see **What these numbers are evidence about** below, and exp-065, which measured it.
It is depth 16, and `gen_tree.py` writes anything over 256 bytes with `os.truncate`, so
it is 595.7 MB apparent against 26.3 MB allocated: 22.6× sparse.

| Job | Control | Candidate | Change | 95% interval |
| --- | --- | --- | --- | --- |
| `content-cache-hit` | 450.4 ms | 314.7 ms | **−30.31%** | [−30.69%, −29.61%] |
| `content-basic` | 518.9 ms | 450.0 ms | **−13.40%** | [−14.74%, −10.92%] |

### What these numbers are evidence about

**Re-measured on 2026-08-23 against `origin/main` at `ac4806a`, 44 commits later, on a
regenerated copy of this subject** (exp-065). Both figures reproduced: `content-basic`
−13.56% [−15.57%, −12.73%] against the −13.401% below, and `content-cache-hit` −32.61%
against −30.307%. Nothing here needs correcting.

What the re-measurement did establish is how far each number carries.
On a dense real tree — 10,703 files of Rust source, depth 10, no sparseness — the same
binaries measure:

| Job | This subject | Dense real tree |
| --- | --- | --- |
| `content-cache-hit` | −32.61% | **−25.78%** |
| `content-basic` | −13.56% | **−2.38%** |

The warm number transfers, and it is the one this experiment took its verdict on.
The cold number does not: on a dense tree it is under the 3% bar.
Both are correct measurements of one absolute saving against denominators that differ by
more than the saving does — this tree is deeper (more ancestors per file) and its files
are holes (near-zero read cost per file), so bookkeeping is most of its cold work and a
corner of a real tree’s. Read the −13.40% as evidence about this subject, not about the
content tier.

Attribution between the two arms, `content-cache-hit`, `paired_runner.py`:

| Arm | Base | Candidate | wall | 95% CI | pairs |
| --- | --- | --- | --- | --- | --- |
| H94 alone | 464.4 ms | 346.9 ms | −25.42% | [−26.51%, −24.46%] | 24 |
| H95 on top of H94 | 345.6 ms | 326.8 ms | −5.08% | [−6.39%, −3.60%] | 40 |

Instruction counts across the whole sequence: 3,462,200,305 → 2,266,646,925 →
2,106,908,485, i.e. −39.15% cumulative.
`merge_ancestors` inclusive fell 43.73% → 14.07% of profile (−78.9% absolute) and
`classify_path_with_prefix` 16.96% → 10.68% (−41.4% absolute), which is the mechanism
each hypothesis predicted rather than a win found somewhere else.

Peak RSS was neutral throughout (42.3 → 42.5 MB on cache-hit).

## What was dropped

**H95’s predicted transfer to the analysis path, as an H95 claim.** H95 argued the
cascade runs on the cold analysis path too, so `content-basic` should move similarly.
Measured against the post-H94 base it read −4.20% [−6.10%, −0.06%] at 24 pairs and
**−2.34% [−5.05%, −0.64%] at 40** — direction right, interval below zero, median below
the 3% bar once the estimate settled.
Not claimed for H95.

The −13.40% this experiment records on `content-basic` is therefore **H94’s**, not
H95’s: that job populates roll-ups for every one of the 15,977 files through the same
ancestor walk. Reading the cumulative number as vindication of H95’s transfer argument
would be exactly the error this record exists to prevent.

The 24-pair reading is worth keeping for its own sake: it moved 1.9 points on the same
host with nothing changed but sample count.

## What was deliberately not done

- `Index::apply_analysis` still re-runs `classify_path` on a path `analysis_candidates`
  just classified, so classification is computed twice per warm file.
  `AnalysisCandidate` is public and a caller can hand-build an inconsistent one, so that
  guard is a real contract check, not dead work.
  Making classification cheap paid for it without weakening it; deleting it would have
  been `fdu-926e`’s fix 2 and a contract change.
- `files: BTreeMap<PathBuf, FileAnalysis>` is untouched, and its `remove` was a further
  11.09% through `apply_analysis`.
- `with_flags` still walks path components for the vendored and documentation flags on
  every file — 4.42% of the pre-H94 profile.
- The shebang tier has the same non-short-circuiting shape as the two that were indexed,
  but sits on the content-prefix path, off this hot path.
  Left alone rather than indexed on the strength of a mechanism argument with no
  measurement behind it.
- The structural form of H94 — key roll-ups by `EntryId` and defer to one bottom-up
  pass, the shape that won −51.9% on snapshot load in `fdu-91ts` — remains open and is
  now the larger of the two remaining items on this path.

## Oracles

`engine_digest` and the content digest are unchanged across both arms, and the harness
bound the tree fingerprint to the run.
Beyond that, a
`--view tree,types,families,languages,extensions,summary --depth 12 --format json`
render over the same tree is byte-identical between arms at 227,198 bytes apart from two
timestamp fields — which verifies intermediate directory roll-ups, not only the root
total the probe digest covers.
Two tests were added: one pins the indexed tiers against the scan they replaced over
every key the rules table declares, including the `max_by_key` last-wins tie-break; the
other pins non-UTF-8 names to the unknown tier.

## Regime

Linux, virtualized, uncontrolled host, warm OS cache, exploratory stage.
Content tier only. Nothing here is evidence about the aggregate or index tiers, or about
macOS.
