---
title: "Linux H172 exact transient tree tier cuts the default tree 13% on linux-v6.12"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-180
  title: "Linux H172 exact transient tree tier cuts the default tree 13% on linux-v6.12"
  date: "2026-09-29"
  hypotheses:
    - H172
    - H176
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
    control: "a1a4a568 probe: H171 + H175 (accepted engine)"
    candidate: "956659de probe: H172 transient tree tier with H176 and F6e"
    control_binary:
      name: control
      sha256: ef07e553fa04cc83ac07549ed575303b89ea99d9ed27b88e25e429be12527509
      size_bytes: 3818784
      args: []
    candidate_binary:
      name: h172
      sha256: 882a639f58a20c22081fca23fc02359ce21809c90d91396a8b350910de555b51
      size_bytes: 3846168
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-180/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 92036609.0
          candidate_median: 94040125.5
          control_p95_over_median: 1.151
          candidate_p95_over_median: 1.086
          change_pct: 3.495
          ci95_low_pct: -2.978
          ci95_high_pct: 8.122
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 88943448.0
          candidate_median: 90549333.5
          control_p95_over_median: 1.114
          candidate_p95_over_median: 1.085
          change_pct: 3.325
          ci95_low_pct: -2.537
          ci95_high_pct: 7.341
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 313273500.0
          candidate_median: 324668500.0
          control_p95_over_median: 1.14
          candidate_p95_over_median: 1.086
          change_pct: 2.76
          ci95_low_pct: -1.906
          ci95_high_pct: 5.481
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 132788000.0
          candidate_median: 145950000.0
          control_p95_over_median: 1.147
          candidate_p95_over_median: 1.072
          change_pct: 6.062
          ci95_low_pct: 3.083
          ci95_high_pct: 12.016
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        system_cpu_ns:
          control_median: 181894000.0
          candidate_median: 183482000.0
          control_p95_over_median: 1.242
          candidate_p95_over_median: 1.118
          change_pct: -2.911
          ci95_low_pct: -8.731
          ci95_high_pct: 7.557
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
          - "minor_faults straddles its +10% regression limit"
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
          minor_faults: inconclusive
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 91806074.5
          candidate_median: 80169147.5
          control_p95_over_median: 1.247
          candidate_p95_over_median: 1.143
          change_pct: -13.485
          ci95_low_pct: -18.847
          ci95_high_pct: -6.31
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 86784121.0
          candidate_median: 76913319.0
          control_p95_over_median: 1.252
          candidate_p95_over_median: 1.143
          change_pct: -11.668
          ci95_low_pct: -17.85
          ci95_high_pct: -5.252
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 300434000.0
          candidate_median: 279530000.0
          control_p95_over_median: 1.065
          candidate_p95_over_median: 1.038
          change_pct: -9.537
          ci95_low_pct: -10.264
          ci95_high_pct: -4.941
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 113147500.0
          candidate_median: 93367000.0
          control_p95_over_median: 1.15
          candidate_p95_over_median: 1.208
          change_pct: -18.065
          ci95_low_pct: -25.543
          ci95_high_pct: -9.798
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 193961500.0
          candidate_median: 181240500.0
          control_p95_over_median: 1.092
          candidate_p95_over_median: 1.134
          change_pct: -1.588
          ci95_low_pct: -10.408
          ci95_high_pct: 4.077
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 1555
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "most lines are tests: the transient-versus-indexed differential over every bound case, eligibility, and boundary units"
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -13.485
    reason: "quiet 20-pair linux-v6.12 default-tree -13.48% [-18.85%, -6.31%] and node-modules-dense -10.30% [-15.09%, -7.20%] (exp-181); summary and cold-scan-index placebos include zero; balanced-1m screen -3.20% wall, peak RSS -79%; answers identical"
    commit: 956659de
    kept: candidate
---
## What was predicted

H172, an exact transient tree tier, with H176 inside it and the fused post-walk pass
(F6e), as settled by the overnight plan’s review
([amendment 5](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md)).

When a one-shot tree request proves nothing reads the index, the builder keeps every
directory and only the K = ⌈100/min-share⌉ largest files.
It folds every other file into its directory’s totals and a per-directory `FoldedFiles`
tally, which the share omission reads.
Under the tier no extension is interned and no per-extension map is built (H176), and
detached directories are allocated as listed in full, so a clean walk skips the second
arena pass (F6e).

Pre-registered in the registry (`f55d501c`) before any timed sample:
- **Deciding:** `default-tree` wall −3% with the interval below zero on `linux-v6.12`
  and on `node-modules-dense` (co-primaries, both real).
- **Screening:** `linux-balanced-1m`.
- **Placebos:** `aggregate-summary` with and without controls (the summary route is
  untouched), and `cold-scan-index`.
- **Prediction:** −8% to −12% on `linux-v6.12` after H171.

## What was measured

Control: the accepted H171+H175 engine (`a1a4a568` probe; the merge `2379233a` has the
same crates). Candidate: `956659de`, which is the H172 branch merged with that engine
(`b5e8c060`) plus tests only.
All cells quiet, with no invalid samples.

| Subject, job | Pairs | Control | H172 | Change |
| --- | ---: | ---: | ---: | --- |
| `linux-v6.12`, `default-tree` | 20 | 91.8 ms | 80.2 ms | **−13.48% [−18.85%, −6.31%]** |
| `linux-v6.12`, `default-tree --no-controls` | 20 | 84.2 ms | 68.8 ms | −19.39% [−22.54%, −14.06%] |
| `linux-v6.12`, `aggregate-summary` (placebo) | 20 | 92.0 ms | 94.0 ms | +3.50% [−2.98%, +8.12%] |
| `linux-v6.12`, `aggregate-summary --no-controls` (placebo) | 20 | 74.2 ms | 73.1 ms | +1.92% [−3.35%, +5.07%] |
| `node-modules-dense`, `default-tree` | 20 | 85.3 ms | 75.4 ms | **−10.30% [−15.09%, −7.20%]** (exp-181) |
| `node-modules-dense`, `cold-scan-index` (placebo) | 20 | 184.8 ms | 182.1 ms | −1.90% [−5.18%, +2.25%] |
| `linux-balanced-1m`, `default-tree` (screen) | 12 | 1,344.8 ms | 1,302.4 ms | −3.20% [−4.74%, −2.40%] |

- **Balanced screen:** peak RSS fell 79% (306 MB to 64 MB), minor faults 80% and user
  CPU 10%.
- **Kernel tree:** the index holds 5,930 entries instead of 92,473, and bytes allocated
  fall from 100.8 MB to 20.7 MB (counters).
- **Placebos:** all four include zero.
- **Host drift:** the control’s absolute level was 126 ms on the kernel tree in the
  09:50 cell (exp-179) and 91.8 ms here.
  The paired changes are what this record claims.

**Answers.**
- The product command line matched the base byte for byte in 171 comparisons: text, JSON
  and JSONL, with `--view tree` and the tier-reaching sort, share, depth, breadth and
  size variants over the three subjects.
  A second 37-variant check matched in 111 comparisons, 27 of which reach the tier.
- `transient_tree_equals_the_indexed_tree_under_every_bound_case` compares folded and
  full reports across worker counts, orders, shares, metrics, sort keys, bounds, ties at
  K, zero-size files, ignored files at the boundary, refused and unreadable controls, a
  failed directory listing, FIFOs, sockets and hard links.
- The workspace passes 1,124 tests and the golden corpus passes 212. The
  permission-gated cases were also run as a non-root user.
- A Fable adversarial review found no counterexample.
  It checked that K is exact against the root total at every depth, and that ties and
  eligibility against every request field and command-line flag hold.
  It also confirmed that a folded index can never leave the one-shot route.

## Decision

Accepted on both real co-primaries: the default tree is 13.5% faster on the source tree
with ignore rules and 10.3% faster on the dense tree, peak memory falls 79% on a million
entries, and every placebo is at zero.
H176 and F6e are part of this change and are not separately claimed.
On the generated tree the wall effect is 3%, below the prediction: there the walk’s CPU
sets the time, which is the next item’s target.
No public API change, no new dependency, no `unsafe`. About 1,500 lines, most of them
tests.
