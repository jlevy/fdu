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
    reason: "Mechanism confirmed, effect absent: instructions -1.69% on the decisive subject and -3.18% on a small dense one, but wall +0.05% [-0.82%, +0.86%] over 40 pairs. On this virtualized Linux host the warm content open is not instruction-bound."
    commit: null
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

Keying by bytes also made the candidate route `rollup()` through `normalized()`, as
`file()` already does.
A byte-keyed map needs that on Windows, where a slash and a backslash both separate
components but are different bytes.
The shipped `HashMap<PathBuf, _>` does not: `Path`’s `Hash` skips separator bytes and
its `Eq` compares components, so either spelling finds the same roll-up.
Any later byte-keyed `rollups` has to keep the normalization.

## What happened

The mechanism does exactly what it claims, and it does not matter.

| Subject | Entries | Instructions (control → candidate) | Change |
| --- | --- | --- | --- |
| cargo registry checkout | 3,077 | 188,692,572 → 182,691,359 | **−3.18%** |
| linux kernel checkout | 102,318 | 12,931,638,512 → 12,712,583,220 | **−1.69%** |

Paired changes (median of the per-pair differences) on the decisive subject, 40
interleaved pairs:

- `content-cache-hit` wall **+0.05%**, 95% interval **[−0.82%, +0.86%]** — REJECT.
- component **+0.07%**, 95% interval **[−0.77%, +1.00%]**; peak RSS flat.
- Content digest and engine digest byte-identical on every trial; 95,933 cache hits
  both.

The interval is tight and centred on zero.
This is a settled null, not an underpowered reading — the distinction H95 paid for and
this record should not have to relearn.

## Why it fails, and what that transfers to

Three things are worth carrying forward.

**On this host, the warm content open is not instruction-bound.** Removing 219 million
instructions from a 12.9 billion instruction run moved wall time by nothing measurable,
and CPU time did not move either.
Both intervals, wall [−0.82%, +0.86%] and CPU [−0.81%, +0.90%], exclude the −1.69% a
saving proportional to instructions would have produced.
The regime is one virtualized 4-core Xeon under Linux, warm-steady on ext4; Apple
Silicon and bare metal are unmeasured.
A plausible explanation, not a measured one, since no cycle or IPC counts were recorded:
the instructions H103 removed are high-IPC (SipHash rounds and component parsing are
tight, well-predicted loops), and what remains is memory-stalled.
A later hypothesis on this tier whose mechanism is “fewer instructions” should therefore
be screened against this result rather than against the 3% bar in the abstract.
That points the tier’s remaining cost at layout and allocation, which is what H78/H83
and the structural form in `fdu-jxhk` already say.

**The saving’s share is subject-shaped.** −3.18% on a 3,077-entry tree became −1.69% on
a 102,318-entry one.
Per entry, the saving was nearly the same on both subjects, 1,950 and 2,141
instructions; what differed was everything else the control run cost per entry, about
59,400 instructions on the registry subject and 124,200 on the kernel checkout (61,323
and 126,386 in total, less the saving).
The trees differ in shape as well as size, a registry of many small crates against one C
source tree, and only the registry subject was profiled, so what makes up that doubling
is not established. A snapshot parse that grows faster than the roll-up hashing is one
plausible reading. Either way, a screening subject would have overstated this saving by
roughly 2×. That is the same lesson exp-065 recorded from the other side, and it is the
reason the loop requires 50,000 entries before a subject may decide.

**The 8% in H102’s registry entry covered two maps, and H103 changed one.** exp-069
named `Path::hash` and SipHash as the next 8% of its warm profile across both the
roll-up `HashMap` and the candidate map the sidecar loader builds (`content_cache.rs`,
one insert and one remove per file).
H102’s registry row carried the figure forward as the roll-up map alone.
H103 removed the roll-up half and measured no wall effect; the loader half is untested.
At two full-path hashes per file, against one hash per ancestor per file for the
roll-ups, it is expected to be smaller, which is a prediction and not a result.
This experiment’s caller-tree profile, on the registry subject, puts
`<Path as Hash>::hash` at 3.98% of the profile and 5.2% of the engine, and
`merge_ancestors` with its hashbrown probing at 9.66% of the profile.
exp-069’s 8% came from a different subject and profile, so the two shares are not
directly comparable.
H102’s registry row now says this.

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
