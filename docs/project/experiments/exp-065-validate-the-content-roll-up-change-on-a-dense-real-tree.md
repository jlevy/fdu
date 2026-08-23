---
title: Validate the content roll-up change on a dense real tree
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-065
  title: Validate the content roll-up change on a dense real tree
  date: "2026-08-23"
  hypotheses:
    - H94
    - H95
  subject:
    tree_label: cargo-registry-src
    tree_root_id: 3b0c27281785ff1f1fefa8cead3d8882cc450153bcb41a286646cc3979594f68
    tree_engine_digest: 1e1c005ab9d6b9ff690931506bdf001d7fddd2dbfd2f6702cd14bcaf6cdd8e5f
    tree_provenance: "The cargo registry source cache, populated by cargo fetch over this workspace's lockfile. Shape depends on which crates a given lockfile pulls, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 13020
    tree_directories: 2317
    tree_files: 10703
    tree_symlinks: 0
    tree_apparent_bytes: 221367691
    tree_allocated_bytes: 248791040
    tree_max_depth: 10
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
    control: origin/main at ac4806a
    candidate: "HashMap rollups plus LazyLock-indexed name and extension tiers, as accepted in exp-064"
    control_binary:
      name: control
      sha256: 52f021957e6354d39188b554a7f41e7c0ff6bc64768d737bccb2f5fa7ff913c7
      size_bytes: 1958952
      args: []
    candidate_binary:
      name: candidate
      sha256: 6a7baab94d573e8800248f8e37022439a7bec8fb759b151f129bdbbdc94e1137
      size_bytes: 1954512
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-perf-results/run-pr38-revalidate.json
  results:
    - job: code-sloc
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1067472461.5
          candidate_median: 1056111634.5
          change_pct: -1.087
          ci95_low_pct: -1.97
          ci95_high_pct: 0.181
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 24
        component_ns:
          control_median: 996962275.0
          candidate_median: 985584823.5
          change_pct: -0.703
          ci95_low_pct: -1.342
          ci95_high_pct: -0.317
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 24
        cpu_ns:
          control_median: 3773412500.0
          candidate_median: 3755885500.0
          change_pct: -0.407
          ci95_low_pct: -0.92
          ci95_high_pct: -0.047
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 24
        user_cpu_ns:
          control_median: 3422762000.0
          candidate_median: 3386595000.0
          change_pct: -0.682
          ci95_low_pct: -1.888
          ci95_high_pct: 0.363
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 24
        system_cpu_ns:
          control_median: 357335000.0
          candidate_median: 362070500.0
          change_pct: 1.914
          ci95_low_pct: -5.107
          ci95_high_pct: 12.611
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 24
        peak_rss_bytes:
          control_median: 33490944.0
          candidate_median: 33490944.0
          change_pct: -0.14
          ci95_low_pct: -0.342
          ci95_high_pct: 0.329
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 24
      qualification:
        campaign_stage: exploratory
        classification: noninferior
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
    - job: code-sloc-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 261924383.5
          candidate_median: 191652093.5
          change_pct: -26.511
          ci95_low_pct: -28.208
          ci95_high_pct: -25.028
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        component_ns:
          control_median: 225003942.0
          candidate_median: 156242304.5
          change_pct: -29.997
          ci95_low_pct: -31.086
          ci95_high_pct: -28.885
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        cpu_ns:
          control_median: 261467500.0
          candidate_median: 191279500.0
          change_pct: -26.618
          ci95_low_pct: -28.221
          ci95_high_pct: -25.036
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        user_cpu_ns:
          control_median: 236012500.0
          candidate_median: 168405000.0
          change_pct: -28.36
          ci95_low_pct: -31.881
          ci95_high_pct: -24.865
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        system_cpu_ns:
          control_median: 27943000.0
          candidate_median: 27674000.0
          change_pct: -6.566
          ci95_low_pct: -32.847
          ci95_high_pct: 22.882
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 24
        blocked_ns:
          control_median: 443258.0
          candidate_median: 420117.5
          change_pct: -7.564
          ci95_low_pct: -20.433
          ci95_high_pct: 2.759
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 24
        peak_rss_bytes:
          control_median: 38813696.0
          candidate_median: 38760448.0
          change_pct: -0.164
          ci95_low_pct: -0.253
          ci95_high_pct: 0.18
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
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
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 510202669.0
          candidate_median: 499603866.0
          change_pct: -2.384
          ci95_low_pct: -3.581
          ci95_high_pct: -0.536
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 24
        component_ns:
          control_median: 441342639.0
          candidate_median: 432809982.5
          change_pct: -2.979
          ci95_low_pct: -4.001
          ci95_high_pct: -0.346
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 24
        cpu_ns:
          control_median: 1557838500.0
          candidate_median: 1531269500.0
          change_pct: -1.261
          ci95_low_pct: -2.221
          ci95_high_pct: -0.592
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 24
        user_cpu_ns:
          control_median: 1269631500.0
          candidate_median: 1232726500.0
          change_pct: -2.918
          ci95_low_pct: -4.383
          ci95_high_pct: -0.356
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 24
        system_cpu_ns:
          control_median: 287430500.0
          candidate_median: 299668000.0
          change_pct: 7.695
          ci95_low_pct: -2.469
          ci95_high_pct: 10.262
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 24
        peak_rss_bytes:
          control_median: 33126400.0
          candidate_median: 33110016.0
          change_pct: -0.161
          ci95_low_pct: -0.364
          ci95_high_pct: 0.025
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 24
      qualification:
        campaign_stage: exploratory
        classification: noninferior
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
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 270680931.5
          candidate_median: 201863371.5
          change_pct: -25.776
          ci95_low_pct: -26.741
          ci95_high_pct: -24.517
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        component_ns:
          control_median: 234339660.0
          candidate_median: 164673047.5
          change_pct: -29.854
          ci95_low_pct: -31.142
          ci95_high_pct: -28.625
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        cpu_ns:
          control_median: 270245500.0
          candidate_median: 201241500.0
          change_pct: -25.804
          ci95_low_pct: -26.771
          ci95_high_pct: -24.523
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        user_cpu_ns:
          control_median: 241636500.0
          candidate_median: 167337500.0
          change_pct: -31.367
          ci95_low_pct: -33.87
          ci95_high_pct: -27.854
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 24
        system_cpu_ns:
          control_median: 26027000.0
          candidate_median: 31890000.0
          change_pct: 26.392
          ci95_low_pct: -0.609
          ci95_high_pct: 51.097
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 24
        blocked_ns:
          control_median: 441464.0
          candidate_median: 450182.5
          change_pct: -1.303
          ci95_low_pct: -9.084
          ci95_high_pct: 13.294
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 24
        peak_rss_bytes:
          control_median: 38477824.0
          candidate_median: 38369280.0
          change_pct: -0.181
          ci95_low_pct: -0.309
          ci95_high_pct: -0.122
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: No production change; this re-measures the exp-064 candidate against current main on a second subject.
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -25.776
    reason: "The warm mechanism transfers to a dense real tree at -25.78% [-26.74%, -24.52%] on content-cache-hit and -26.51% on code-sloc-cache-hit; exp-064 also reproduced on its own regenerated subject at -13.56% against -13.401% recorded. The cold half does not transfer: content-basic reads -2.38% here against -13.56% there, because that subject is depth 16 and 22.6x sparse, so bookkeeping is most of its cold work."
    commit: b061f5b
---
## What was measured

Whether exp-064’s accepted content-tier change still holds 44 commits later, and whether
its two headline numbers mean the same thing on a tree that is not the one they were
measured on.

The change under test is unchanged from exp-064: `ContentIndex::rollups` as a `HashMap`
(H94) plus `LazyLock`-indexed name and extension tiers (H95). The control is
`origin/main` at `ac4806a`, which now carries the crate split and 44 commits of drift
that did not exist when exp-064 was recorded.

Two subjects, the same four jobs, 24 interleaved pairs each:

- **The original subject, regenerated.** `gen_tree.py <root> 17000` rebuilt exp-064’s
  tree exactly — 17,041 entries, 1,045 directories, 15,977 files, 19 symlinks, depth 16,
  595,728,806 bytes apparent and 26,341,376 allocated, every field matching the recorded
  subject.
- **A dense real tree**, this experiment’s recorded subject: 10,703 files of real Rust
  source, depth 10, 221 MB apparent against 249 MB allocated.

## The reproduction holds

| Job | exp-064 recorded | Same subject, today | Δ |
| --- | --- | --- | --- |
| `content-basic` | −13.401% | **−13.56%** [−15.57%, −12.73%] | 0.16 pt |
| `content-cache-hit` | −30.307% | **−32.61%** [−32.87%, −32.27%] | 2.30 pt |

exp-064 is sound. A fresh build of both arms, a fresh generation of the tree, and a
control 44 commits newer land within a fifth of a point on the job exp-064 was
questioned over. Nothing about its numbers needed correcting.

## The same change, on a dense tree

| Job | Generated, depth 16, sparse | Real source, depth 10, dense |
| --- | --- | --- |
| `content-cache-hit` | −32.61% [−32.87%, −32.27%] | **−25.78%** [−26.74%, −24.52%] |
| `code-sloc-cache-hit` | −34.38% [−35.10%, −33.39%] | **−26.51%** [−28.21%, −25.03%] |
| `content-basic` | −13.56% [−15.57%, −12.73%] | **−2.38%** [−3.58%, −0.54%] |
| `code-sloc` | −13.06% [−14.23%, −11.91%] | **−1.09%** [−1.97%, +0.18%] |

The warm cache-hit win transfers: a quarter off both warm content jobs on real source is
the mechanism doing on a real tree what it did on a generated one, and it is the job
exp-064 took its verdict on.

The cold analysis jobs do not transfer.
`content-basic` falls from −13.56% to −2.38%, under the 3% bar; `code-sloc` falls to
−1.09% with an interval spanning zero.
On this subject the cold half of exp-064’s headline would not have been accepted.

## Why the subject decides it

Two properties of the generated tree inflate the cold number, and the artifact recorded
both without anyone reading them as a qualification.

**Depth.** H94 removes a `PathBuf` allocation per ancestor per file.
The generated tree is depth 16; the real tree is depth 10. Warm per-file saving tracks
that ratio closely — 9.04 µs/file against 6.43 µs/file, a factor of 1.41 where depth
alone predicts 1.6.

**Sparseness.** `gen_tree.py` writes files above 256 bytes with `os.truncate`, so they
are holes.
The tree’s 595.7 MB apparent against 26.3 MB allocated — a 22.6× ratio, in the
recorded subject all along — means `content-basic` there reads almost nothing per file.
The real subject’s 221 MB apparent against 249 MB allocated is dense: every byte
analyzed is a byte read.

So the cold job’s denominator differs by far more than its numerator.
Per-file wall saving on `content-basic` is 4.31 µs on the generated tree and 0.99 µs on
the real one, against a per-file cost that is near-zero read plus bookkeeping in the
first case and real read, decode and analysis in the second.
The bookkeeping H94 deletes is most of the sparse tree’s cold work and a small corner of
the dense tree’s.

**The recorded CPU splits that 4.4× into two factors, and only one of them is the
mechanism getting smaller.** Per-file *user-CPU* saving is 4.53 µs on the generated tree
and 3.45 µs on the dense one — a ratio of 1.31, close to the 1.41 the warm jobs show and
consistent with depth 16 against depth 10. The work deleted therefore transfers almost
intact. What does not transfer is how much of it the user waits for: on the sparse tree
0.95 of each saved CPU microsecond becomes wall, and on the dense tree 0.29 does.
Those two factors multiply back to the observed gap, 1.31 × 3.32 = 4.36.

That second factor is overlap, not denominator.
A cold `content-basic` run on dense source has real read, decode and analysis to do on
its reader threads, so consumer-side bookkeeping removed from the critical path is
bookkeeping somebody else was already waiting through; on a tree of holes there is
nothing to hide behind and every saved cycle is a saved microsecond.
Both percentages are correct measurements of the same absolute mechanism, against
denominators that are not comparable *and* against critical paths that are not the same
shape.

The practical consequence is a second question to ask of any structural result, beside
“what was the denominator”: **was the saving on the critical path of the regime it will
ship into?** A change that deletes consumer CPU can be worth its full measured wall on a
tree where the consumer is the bottleneck and nearly none of it on a tree where the
kernel or the reader is — without the mechanism weakening at all.

The warm jobs are the ones that transfer because a cache hit does no reading on either
subject, so the denominators are alike and only depth separates them.

## What this changes

- exp-064’s numbers stand as recorded.
  Its verdict rests on `content-cache-hit`, which reproduces here and transfers to a
  dense tree.
- Its `content-basic` figure is qualified in place, not corrected: it is evidence about
  a deep, sparse, generated tree, and −2.38% is what the same change does to a dense
  one.
- `Subject` gained `tree_provenance` and `tree_reconstructible`, and the ledger now
  prints the apparent-to-allocated ratio when a tree is materially sparse.
  exp-064 held every fact needed to predict this result and stated none of them where a
  reader would look.

## Regime

Linux, virtualized, uncontrolled host, warm OS cache, exploratory stage.
Content tier only. Nothing here is evidence about the aggregate or index tiers, or about
macOS. The cold jobs start from an absent snapshot, not a dropped page cache: dropping
it needs root and a hypervisor’s cache sits underneath regardless.
