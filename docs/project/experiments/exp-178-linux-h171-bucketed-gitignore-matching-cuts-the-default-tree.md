---
title: "Linux H171 bucketed .gitignore matching cuts the default tree 30% on linux-v6.12"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-178
  title: "Linux H171 bucketed .gitignore matching cuts the default tree 30% on linux-v6.12"
  date: "2026-09-29"
  hypotheses:
    - H171
  subject:
    tree_label: linux-v6.12
    tree_root_id: 4ffe9d749fe638cd6c746668f7ae66d88e50225fa6903eabd8c4b9405b1aa245
    tree_engine_digest: 55a09ea65f48d38d88d7e2eacde95fa81d0a9970baabfeeba6f9e9418f478704
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293209
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v49
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: e5a71c8a probe (0.2.1 engine)
    candidate: "7c69e88a probe: H171 bucketed matching"
    control_binary:
      name: control
      sha256: 3ed8fdbfa1af82713b2b828ec0398cfaed3f3eabd14f55b6b738914849bd0d25
      size_bytes: 3758784
      args: []
    candidate_binary:
      name: h171
      sha256: 24482043e6d2a6c057ff479a613347c963d0f9d9627b55a57d12f2849fb1e523
      size_bytes: 3815504
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-178/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 157395671.5
          candidate_median: 115529640.0
          control_p95_over_median: 1.142
          candidate_p95_over_median: 1.093
          change_pct: -25.451
          ci95_low_pct: -28.146
          ci95_high_pct: -21.743
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 153653946.0
          candidate_median: 112500329.0
          control_p95_over_median: 1.14
          candidate_p95_over_median: 1.09
          change_pct: -25.702
          ci95_low_pct: -28.631
          ci95_high_pct: -22.096
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 518353000.0
          candidate_median: 425727000.0
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.081
          change_pct: -16.947
          ci95_low_pct: -19.258
          ci95_high_pct: -14.827
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 221120000.0
          candidate_median: 130379000.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.179
          change_pct: -41.394
          ci95_low_pct: -44.662
          ci95_high_pct: -39.576
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 298308000.0
          candidate_median: 292838500.0
          control_p95_over_median: 1.082
          candidate_p95_over_median: 1.154
          change_pct: 1.705
          ci95_low_pct: -3.499
          ci95_high_pct: 5.137
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "voluntary_context_switches straddles its +50% regression limit"
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
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 188434075.5
          candidate_median: 130170716.5
          control_p95_over_median: 1.177
          candidate_p95_over_median: 1.064
          change_pct: -29.621
          ci95_low_pct: -33.577
          ci95_high_pct: -26.11
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 181757747.5
          candidate_median: 124920369.5
          control_p95_over_median: 1.194
          candidate_p95_over_median: 1.07
          change_pct: -30.776
          ci95_low_pct: -33.659
          ci95_high_pct: -26.99
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 509594000.0
          candidate_median: 423526500.0
          control_p95_over_median: 1.076
          candidate_p95_over_median: 1.047
          change_pct: -17.002
          ci95_low_pct: -18.693
          ci95_high_pct: -15.187
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 202723000.0
          candidate_median: 119730500.0
          control_p95_over_median: 1.121
          candidate_p95_over_median: 1.247
          change_pct: -41.741
          ci95_low_pct: -47.598
          ci95_high_pct: -36.02
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 315414000.0
          candidate_median: 307009500.0
          control_p95_over_median: 1.103
          candidate_p95_over_median: 1.059
          change_pct: -2.593
          ci95_low_pct: -7.619
          ci95_high_pct: 7.1
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 34209792.0
          candidate_median: 32401408.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.039
          change_pct: -6.052
          ci95_low_pct: -9.575
          ci95_high_pct: -4.429
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
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
    lines_changed: 1481
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "about 1,350 of the lines are tests: a property test against the linear matcher kept under cfg(test), a t3070-wildmatch table, FNV collision and memory-charge tests"
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -29.621
    reason: "quiet 20-pair linux-v6.12 default-tree -29.62% [-33.58%, -26.11%], aggregate-summary -25.45% [-28.15%, -21.74%]; placebos (--no-controls both arms; balanced-1m) include zero; glob evaluations 110 -> 0.0019 per entry; answers identical to the base and to git check-ignore"
    commit: 7c69e88a
    kept: candidate
---
## What was predicted

H171, revised by the overnight plan
([Q2 and amendment 3](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md)):
each `.gitignore` source is compiled at parse into literal-name and ends-with maps, an
anchored group per segment count, and a residual list behind literal prefix, suffix,
length and substring pre-checks, answering with the highest matching index.
It was pre-registered in the registry row before any timed sample (`f55d501c`):
- **Deciding:** `default-tree` and `aggregate-summary`, controls on, `linux-v6.12`; wall
  −3% with the interval below zero.
- **Prediction:** the controls-on tree within 1.15 times its own `--no-controls` arm
  (the unrevised prototype would reach only about 1.45 times).
- **Placebos:** both arms `--no-controls` on `linux-v6.12`; `default-tree` on
  `linux-balanced-1m`, which has no `.gitignore`.
- **Answers:** identical.

## What was measured

One quiet five-arm cell on `linux-v6.12`, 3 warmups and 20 pairs, no invalid samples:
the base `e5a71c8a`, H171 (`7c69e88a`), H171 with H175 (`a1a4a568`, exp-179), and the
base and the H175 build with `--no-controls`.

| Job | Base | H171 | Change | Base `--no-controls` |
| --- | ---: | ---: | --- | ---: |
| `default-tree` | 188.4 ms | 130.2 ms | **−29.62% [−33.58%, −26.11%]** | 114.8 ms |
| `aggregate-summary` | 157.4 ms | 115.5 ms | **−25.45% [−28.15%, −21.74%]** | 105.3 ms |

- Placebo, both arms `--no-controls` (base against the H171+H175 build): `default-tree`
  +0.02% [−0.90%, +2.49%], `aggregate-summary` −0.23% [−6.99%, +0.90%]; both include
  zero.
- Placebo on `linux-balanced-1m` (12 pairs, quiet, the H171+H175 build against the
  base): `default-tree` −0.62% [−2.59%, +1.89%], includes zero.
- Prediction: with H175 on top, the controls-on tree is 126.0 ms against its own
  `--no-controls` arm’s 113.3 ms, 1.11 times; the prediction holds.
- Counters (`FDU_COUNTERS=1`, default tree, `linux-v6.12`): 172 full glob evaluations
  for 92,473 entries, 0.0019 per entry, against about 110 per entry for the linear
  matcher; 3.5 map probes per entry.
- Absolute levels drifted during the session: the base’s `--no-controls` arm read 89.9
  ms at 08:20 (exp-175) and 114.8 ms here, and the balanced base 1,359 ms against 1,748
  ms. The paired changes are what this record claims.

**Answers.**
- The product command line’s text, JSON and JSONL output over all three subjects was
  byte-identical to the base: 54 comparisons, run-scoped timestamps masked.
- fdu’s ignored set agreed with `git check-ignore --no-index` on every path of
  `linux-v6.12` (92,473 paths, 40 ignored).
  It also agreed on a built-kernel overlay: 324,015 paths, 224,824 ignored, with the
  tree’s real `.gitignore` files and synthetic build outputs.
- A property test compares the new matcher with the linear one, kept under `cfg(test)`:
  6,000 random rule files and 144,000 queries.
  A table adapted from git’s `t3070-wildmatch` adds 91 name groups and 33 path rows.
- A Fable adversarial review found no counterexample.
  It ported both matchers to Python, fuzzed 760,000 queries, and checked the four
  classification lemmas over 436,620 unrestricted globs.

**Memory charge.** The matcher stores indices, not key bytes, and each rule’s segments
are exact-size slices, so `content_cost` is unchanged.
A new test fits the tightest rule shapes under it with 0–1 byte of slack per line.

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-placebo-balanced.json.gz`, the `linux-balanced-1m` placebo screen.

## Decision

Accepted. The default tree on the real source tree is 29.6% faster and the default
summary 25.5% faster, with both placebos at zero and answers identical against the base
and against git. `IGNORE_RULES_VERSION`, the snapshot format and the public API are
unchanged; there is no new dependency and no `unsafe`. About 1,480 lines, about 1,350 of
them tests.
