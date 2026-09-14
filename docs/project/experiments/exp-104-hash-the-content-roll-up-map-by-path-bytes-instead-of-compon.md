---
title: Hash the content roll-up map by path bytes instead of components
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-104
  title: Hash the content roll-up map by path bytes instead of components
  date: "2026-09-14"
  hypotheses:
    - H103
  subject:
    tree_label: linux-kernel-7043
    tree_root_id: 951be6806409ec42b8266983c811d6b44400bcae8346d085530096985af6a2b1
    tree_engine_digest: 1a63921c7b9d151f6497f2c06786c2f79e07951b760212af8fe603d3f0e69bca
    tree_provenance: "git clone --depth 1 https://github.com/torvalds/linux at 704340f1cd0dcef829eb62f5b48ae95a2ce17bdf, with .git removed after checkout."
    tree_reconstructible: true
    tree_entries: 102318
    tree_directories: 6283
    tree_files: 95933
    tree_symlinks: 102
    tree_apparent_bytes: 1640614597
    tree_allocated_bytes: 1869725696
    tree_max_depth: 11
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.80GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16856092672
    host_system: Linux 6.18.44-fc-v24
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 40
    warmups: 3
    interleaved: true
    control: main at dda7e6af
    candidate: "ContentIndex::rollups keyed by byte-hashed PathKey under an in-crate FxHash-style hasher"
    control_binary:
      name: control
      sha256: 96cda6db3b05a8c3cba16ee8063e707acaa7788b131ceb7c2dc08066f24d3924
      size_bytes: 2787696
      args: []
    candidate_binary:
      name: candidate
      sha256: 146fde0ce0b53949f082d41b9fe4b44cf9ae86cee73105ed07b8bb2c131e1273
      size_bytes: 2788864
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-104-rollup-byte-hash.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1961291377.0
          candidate_median: 1973564439.5
          control_p95_over_median: 1.047
          candidate_p95_over_median: 1.048
          change_pct: 0.054
          ci95_low_pct: -0.819
          ci95_high_pct: 0.859
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 40
        component_ns:
          control_median: 1723090367.5
          candidate_median: 1727946871.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.056
          change_pct: 0.067
          ci95_low_pct: -0.772
          ci95_high_pct: 0.996
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 40
        cpu_ns:
          control_median: 1960351000.0
          candidate_median: 1972821500.0
          control_p95_over_median: 1.047
          candidate_p95_over_median: 1.048
          change_pct: 0.06
          ci95_low_pct: -0.814
          ci95_high_pct: 0.896
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 40
        user_cpu_ns:
          control_median: 1783546500.0
          candidate_median: 1801737500.0
          control_p95_over_median: 1.053
          candidate_p95_over_median: 1.064
          change_pct: 0.633
          ci95_low_pct: -0.175
          ci95_high_pct: 1.71
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 40
        system_cpu_ns:
          control_median: 177949000.0
          candidate_median: 168712000.0
          control_p95_over_median: 1.192
          candidate_p95_over_median: 1.209
          change_pct: -2.317
          ci95_low_pct: -9.273
          ci95_high_pct: 2.677
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 40
        blocked_ns:
          control_median: 764539.0
          candidate_median: 748130.5
          control_p95_over_median: 1.797
          candidate_p95_over_median: 1.548
          change_pct: -0.954
          ci95_low_pct: -6.923
          ci95_high_pct: 5.833
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 40
        peak_rss_bytes:
          control_median: 220491776.0
          candidate_median: 220264448.0
          control_p95_over_median: 1.001
          candidate_p95_over_median: 1.0
          change_pct: -0.121
          ci95_low_pct: -0.135
          ci95_high_pct: -0.093
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 40
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
  reference_tools: []
  complexity:
    lines_changed: 66
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "A hand-written FxHash-style Hasher and a second Borrow contract on PathKey, for no measured wall change. Reverted."
  verdict:
    decision: rejected
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: 0.054
    reason: "Mechanism confirmed, effect absent: instructions -1.69% on the decisive subject and -3.18% on a small dense one, but wall +0.05% [-0.82%, +0.86%] over 40 pairs. The warm content open is not instruction-bound."
    commit: dda7e6af5b7bd4a088a816f1c449cf56c22a8a62
---
## What was tried

`ContentIndex::rollups` was a `HashMap<PathBuf, ContentRollUp>` under the default
`RandomState`. Two costs follow from that pair of choices, and H103 removed both in one
arm because [H102’s registry entry](../guides/performance-loop.md) scoped them as one
increment:

- `Path`’s own `Hash` walks `Components`, so it re-parses the path and makes one
  `Hasher::write` call per component rather than one over the whole key.
- SipHash-1-3’s collision resistance buys nothing for a map keyed by paths from the
  user’s own tree that is only ever point-queried, and its per-call setup is pure cost.

The candidate keys the map by the byte-ordered `PathKey` the `files` map already uses
(exp-069/H102), adds `impl Hash for PathKey` hashing `self.bytes()`, and hashes with an
in-crate FxHash-style mixer.
No new dependency and no `unsafe`; the change is 66 lines inside `content_index.rs` and
touches no public signature.

## What happened

The mechanism does exactly what it claims, and it does not matter.

| Subject | Entries | Instructions (control → candidate) | Change |
| --- | --- | --- | --- |
| cargo registry checkout | 3,077 | 188,692,572 → 182,691,359 | **−3.18%** |
| linux kernel checkout | 102,318 | 12,931,638,512 → 12,712,583,220 | **−1.69%** |

Wall time on the decisive subject, 40 interleaved pairs:

- `content-cache-hit` wall **+0.05%**, 95% interval **[−0.82%, +0.86%]** — REJECT.
- component **+0.28%** (1,723.1 ms → 1,727.9 ms), peak RSS flat.
- Content digest and engine digest byte-identical on every trial; 95,933 cache hits
  both.

The interval is tight and centred on zero.
This is a settled null, not an underpowered reading — the distinction H95 paid for and
this record should not have to relearn.

## Why it fails, and what that transfers to

Three things are worth carrying forward.

**The warm content open is no longer instruction-bound.** Removing 219 million
instructions from a 12.9 billion instruction run moved wall time by nothing measurable,
and CPU time did not move either.
The instructions H103 removed are high-IPC (SipHash rounds and component parsing are
tight, well-predicted loops); what remains is memory-stalled.
Any future hypothesis on this tier whose mechanism is “fewer instructions” should expect
the same answer, and should be screened against this result rather than against the 3%
bar in the abstract.
The tier’s remaining cost is layout and allocation — which is what H78/H83 and the
structural form in `fdu-jxhk` already say.

One correctness item came out of reading this map as well: `rollup()` looks up the
caller’s raw path in a map whose keys are all normalized, which is unreachable on unix
and a silently missing roll-up on Windows.
The rejected candidate fixed it incidentally and took the fix with it when it was
reverted; filed as `fdu-cfpa`.

**The saving is subject-shaped, and shrinks with scale.** −3.18% on a 3k-entry tree
became −1.69% on a 102k-entry one, because the snapshot parse grows faster than the
roll-up hashing does.
A screening subject would have overstated this by roughly 2×. That is the same lesson
exp-065 recorded from the other side, and it is the reason the loop requires 50,000
entries before a subject may decide.

**The 8% estimate in H102’s registry entry was measured on the wrong denominator.** That
entry said “Next increment: `Path::hash` and SipHash on the roll-up map (8%)”. A
caller-tree profile of a warm content open puts `<Path as Hash>::hash` at 3.98% of the
profile and 5.2% of the engine, with the surrounding hashbrown probing bringing
`merge_ancestors` to 9.66% of the profile.
The 8% was the whole of `merge_ancestors`’ map work, not the part a hashing change can
take — and it was a share of a profile that still included the probe’s own oracle digest
at 21.06%. Corrected below.

## Harness cost in this profile

`perf_probe::summarize_index` — the oracle digest, not the engine — is **21.06%** of the
`content-cache-hit` profile on the registry subject (`Sha256::compress` 13.78% plus
`finalize` 6.97%). `fdu_core::open_for_report` is 76.33%. Every engine percentage in
this document is stated against the profile total and then restated against that 76.33%
where the distinction matters; none of them are quoted raw off a flat profile.

## An unrelated observation the profile surfaced

On this subject `Index::install_controls` → `reclassify_controlled_subtrees` is **19.43%
of the profile / 25.5% of the engine** on a `--cache only` warm open, almost all of it
`ControlMatcher::is_ignored` → `std::path::compare_components` (10.87% of the profile
from 47,690 calls). It is not redundant work — `install_controls` already short-circuits
when neither control table governs anything — but it is the largest single engine item
on this tier after the two loaders, and it is `Path` component comparison again, which
is the cost exp-069 already removed once elsewhere.
Filed as `fdu-hzyb` rather than folded in here, because it is control state rather than
content and it is correctness-sensitive.
