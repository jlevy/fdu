---
title: "H86 Linux evidence stage: relative gates pass, floor gates fail"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-103
  title: "H86 Linux evidence stage: relative gates pass, floor gates fail"
  date: "2026-09-02"
  hypotheses:
    - H86
  subject:
    tree_label: linux-450k
    tree_root_id: 65b45ec723560c09be8165ad08a7b7c33cc048c2713b4b8c9b2a906973b9286d
    tree_engine_digest: e6da049852a2172f6b73202db295564076a1dc2868438444d3322f322b414a95
    tree_provenance: "Generated balanced recipe, 450,001 entries, manifest 65aa72b53d5fdae1665d66451b0071b1605f015e8657da69697a25d928dbed6d, semantic digest 0c5230889cbe6ee25ceb6e64560cb012bccd03126565fd8f8d313e7013715e3d, engine digest e6da049852a2172f6b73202db295564076a1dc2868438444d3322f322b414a95"
    tree_reconstructible: true
    tree_entries: 450001
    tree_directories: 56251
    tree_files: 393750
    tree_symlinks: 0
    tree_apparent_bytes: 358665192
    tree_allocated_bytes: 1344430080
    tree_max_depth: 7
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.80GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16856133632
    host_system: Linux 6.18.44-fc-v22
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: c6380f7 immediate immutable control
    candidate: 5d7b86f H86 consumer representation (codex/streaming-performance-parity)
    control_binary:
      name: control
      sha256: 6f541256f006029a127b9f7dfcc95294464f7db79da92ca3f510be5900894758
      size_bytes: 2594584
      args: []
    candidate_binary:
      name: candidate
      sha256: d6fbcec2827f5ea277ec769e71c03f3386e85cb9e5f59676247cd9575c7dd534
      size_bytes: 2710552
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /home/user/perf/results/run-h86-linux-immediate.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1905623645.0
          candidate_median: 1537022159.5
          control_p95_over_median: 1.134
          candidate_p95_over_median: 1.109
          change_pct: -18.165
          ci95_low_pct: -24.254
          ci95_high_pct: -13.719
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 854276236.5
          candidate_median: 614503395.5
          control_p95_over_median: 1.305
          candidate_p95_over_median: 1.073
          change_pct: -25.838
          ci95_low_pct: -34.415
          ci95_high_pct: -19.059
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 3470719000.0
          candidate_median: 2844123000.0
          control_p95_over_median: 1.071
          candidate_p95_over_median: 1.096
          change_pct: -17.971
          ci95_low_pct: -21.474
          ci95_high_pct: -13.139
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 1948230000.0
          candidate_median: 1564570500.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.047
          change_pct: -20.496
          ci95_low_pct: -25.032
          ci95_high_pct: -16.158
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1519554500.0
          candidate_median: 1317559500.0
          control_p95_over_median: 1.118
          candidate_p95_over_median: 1.124
          change_pct: -12.201
          ci95_low_pct: -21.926
          ci95_high_pct: -3.168
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 318380032.0
          candidate_median: 161003520.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.08
          change_pct: -49.163
          ci95_low_pct: -52.606
          ci95_high_pct: -46.158
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1189662207.5
          candidate_median: 821671459.5
          control_p95_over_median: 1.185
          candidate_p95_over_median: 1.089
          change_pct: -31.704
          ci95_low_pct: -34.313
          ci95_high_pct: -29.153
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 1081785914.5
          candidate_median: 803657035.0
          control_p95_over_median: 1.193
          candidate_p95_over_median: 1.088
          change_pct: -26.685
          ci95_low_pct: -29.923
          ci95_high_pct: -24.005
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 2769353500.0
          candidate_median: 2119954500.0
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.137
          change_pct: -19.946
          ci95_low_pct: -23.879
          ci95_high_pct: -17.303
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 1230775500.0
          candidate_median: 803619500.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.031
          change_pct: -36.439
          ci95_low_pct: -39.325
          ci95_high_pct: -28.87
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 1514813500.0
          candidate_median: 1347282500.0
          control_p95_over_median: 1.194
          candidate_p95_over_median: 1.124
          change_pct: -7.961
          ci95_low_pct: -12.691
          ci95_high_pct: -5.166
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 328824832.0
          candidate_median: 210688000.0
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.052
          change_pct: -35.048
          ci95_low_pct: -37.681
          ci95_high_pct: -33.123
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
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
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 29410223182.0
          candidate_median: 26082340091.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.053
          change_pct: -10.728
          ci95_low_pct: -13.967
          ci95_high_pct: -8.239
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 21665766319.5
          candidate_median: 20507452819.0
          control_p95_over_median: 1.047
          candidate_p95_over_median: 1.055
          change_pct: -5.465
          ci95_low_pct: -7.888
          ci95_high_pct: -2.387
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 32085253500.0
          candidate_median: 29809111000.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.052
          change_pct: -7.195
          ci95_low_pct: -10.028
          ci95_high_pct: -3.771
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 22163574500.0
          candidate_median: 21005891000.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.041
          change_pct: -5.318
          ci95_low_pct: -8.517
          ci95_high_pct: -2.065
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 10043974500.0
          candidate_median: 8765304000.0
          control_p95_over_median: 1.086
          candidate_p95_over_median: 1.085
          change_pct: -11.158
          ci95_low_pct: -14.88
          ci95_high_pct: -9.119
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 1264730112.0
          candidate_median: 1084055552.0
          control_p95_over_median: 1.0
          candidate_p95_over_median: 1.002
          change_pct: -14.283
          ci95_low_pct: -14.294
          ci95_high_pct: -14.26
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
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
    new_failure_modes:
      - "absolute floor ratio, not paired regression"
    notes: No code change proposed or made; this is an evidence stage against an existing candidate.
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -31.704
    reason: "The pre-registered Linux floor gates fail on the index tier: default-tree wall is 2.60x the parfloor syscall floor against a 1.4x gate and its peak RSS 6.59x arena_spike against a 3x gate (cold-scan-index 4.86x and 5.03x). Both floor cells are stable (max/min 1.204 and 1.391), so the ratios reject rather than abstain, even though the relative gates pass: default-tree wall -31.70% [-34.31%, -29.15%], cold-scan-index -18.16% [-24.25%, -13.72%], paired peak RSS -35.05% and -49.16%."
    commit: null
---
# H86 Linux Evidence Stage: Relative Gates Pass, Floor Gates Fail

## Stage and Subject

H86 is pre-registered as one decision with two evidence stages, and this is the second:
the original Linux floor claims, which a Darwin acceptance does not replace
([campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md#h86-preregistration-one-decision-two-evidence-stages)).
It ran on a 4-core KVM Xeon against the 450,001-entry generated `balanced` corpus
subject (56,251 directories, 393,750 files), because the delegate’s macOS host could not
supply it while blocked at 127 MiB free.
The two floor cells it is judged against, with their preparation and raw samples, are in
[the Linux floor-cell note](../research/research-2026-09-02-linux-floor-cell-for-h86.md).

## What Was Measured, and What Ships

Candidate `5d7b86f` ran against the immediate immutable control `c6380f7`. Both commits
come from #52’s branch as it stood before a restack, and neither is an ancestor of that
branch now; the restacked equivalents are `f972250` and `a74ac2a`. The tags
`perf/h86-linux-candidate` and `perf/h86-linux-control` keep the measured source
reachable. `verdict.commit` is null because this experiment neither landed nor reverted
code.

The measured binaries are not the ones #52 ships:

- **Features:** the binaries record no feature set, and the release-probe recipe at
  `5d7b86f` built with `--no-default-features` alone, which compiles `gitignore` out.
  Since `1a39be9`, performance builds enable it.
- **Probe scope:** at `5d7b86f` the `default-tree` probe requested control discovery.
  Since `64c6e61` it matches the non-watch CLI’s controls-off scope.
- **Engine:** `ad52469` adds the point-lookup mutation preflight on top.

None of these is a plausible route to the gates.
Passing would take `default-tree` finishing in 443 ms against the 822 ms measured, and
peak RSS at or below 91.5 MiB against the 200.9 MiB and 153.5 MiB the two jobs measured.
Compiling `gitignore` in adds control reading rather than removing work; with it
compiled out, the probe’s control request read no control files; and the preflight
change targets public mutation, which exp-102 measured on `delta-apply` jobs.
The absolute figures below still describe a binary no branch ships, so the quiet-host
stage has to measure the final one.

## The Relative Gates Pass

Twelve paired interleaved trials per job, with zero invalid samples.
Engine digests were identical across all three binaries at worker counts one through
four before any timing, and the post-run tree digest is unchanged.

| Job | Paired wall change | 95% interval | Paired peak RSS change |
| --- | --- | --- | --- |
| `default-tree` | -31.70% | [-34.31%, -29.15%] | -35.05% |
| `cold-scan-index` | -18.16% | [-24.25%, -13.72%] | -49.16% |
| `opened-discovery` | -10.73% | [-13.97%, -8.24%] | -14.28% |

`opened-discovery` only had to stay noninferior within +3%. Every candidate wall
`p95/median` is at or below 1.109, inside the 1.5 limit; across every recorded metric
the largest is 1.137, `default-tree` CPU.

## The Floor Gates Fail

`parfloor stat` at four workers gives a parallel syscall floor of 316.4 ms, and
`arena_spike` under its pre-registered low-churn warm-steady cell gives 362.8 ms and
30.5 MiB. The campaign-2 plan and the floor report measure the index tier with
`default-tree`, so it carries the verdict:

| Job | Variant | Wall | × syscall floor | Peak RSS | × spike RSS |
| --- | --- | --- | --- | --- | --- |
| `default-tree` | control | 1,189.7 ms | 3.76 | 313.6 MiB | 10.28 |
| `default-tree` | candidate | 821.7 ms | **2.60** | 200.9 MiB | **6.59** |
| `cold-scan-index` | control | 1,905.6 ms | 6.02 | 303.6 MiB | 9.96 |
| `cold-scan-index` | candidate | 1,537.0 ms | 4.86 | 153.5 MiB | 5.03 |

The gates are 1.4× on index wall and 3× on peak RSS, and the candidate fails both on
both jobs. `cold-scan-index` supports the verdict rather than headlining it: its wall
also times the probe’s post-scan summary of the index, outside the measured component
(614.5 ms of 1,537.0 ms), so its 4.86× overstates the index tier’s distance from the
gate.

## Why the Rejection Stands

The plan voids the floor and RSS ratios only when the prepared `arena_spike` cell has
`max/min` above 2.0. It measured 1.204, and `parfloor` 1.391, so the ratios can reject.

The host regime cannot carry the rejection on its own.
This ran on a shared cloud KVM with the agent process resident, and the experiment
schema has no host-pressure field, so the `uncontrolled` regime, and the floor cells
running on the same host in the same session as the fdu arms, are the operator’s account
rather than recorded values.
[The performance loop](../guides/performance-loop.md) limits an uncontrolled run to
exploration and discovery.
The rejection rests on its margins instead:

- **Worst floor sample:** against the slowest retained `parfloor` sample, 424.4 ms,
  `default-tree` is still 1.94× the floor.
  Against the largest `arena_spike` RSS sample, 30.6 MiB, its peak RSS is still 6.57×,
  and `cold-scan-index`’s 5.02×.
- **Candidate spread:** to reach the wall gate against that slowest floor sample, a
  `default-tree` trial would have to run 27.7% under the candidate’s median.
  To reach the RSS gate against the largest spike sample, a `cold-scan-index` trial
  would need 40% less peak RSS than its median, and a `default-tree` trial 54% less.
  The candidate’s lower tail was not recorded (see below), but its `default-tree` wall
  `p95/median` is 1.089.

It is not a quiet-host verdict and does not substitute for one, and per
[the platform tuning guide](../guides/platform-tuning.md) it makes no bare-metal claim.

## Records That Did Not Survive

The pre-registration requires all raw samples, `p95/median`, and `max/min` for every
arm. For the fdu arms only derived figures exist:

- **Raw samples:** `run_artifact` names the run JSON on the measurement VM, which no
  longer exists, and the file was never committed.
  Every per-trial wall, CPU, and RSS sample for both arms of all three jobs went with
  it, so the paired intervals above cannot be recomputed.
- **Derived figures:** what `make perf-record` derived from that file survives here:
  per-arm medians, `p95/median`, paired changes, and intervals.
- **`max/min`:** the experiment schema has no such field, so the candidate’s
  pre-registered `max/min` at or below 2.0 is unverified.
  It cannot change the decision, because failing it would only add a second failure to a
  stage that already fails.
  Recording the field is tracked as `fdu-c4jr`.

The floor cells’ raw samples are intact in the floor-cell note.

## Deviations from the Pre-Registration

Three, each detailed in the floor-cell note:

- The aggregate gate, at most 1.25× the floor on the nominated real subjects, was not
  evaluated.
- The subject is a new 450,001-entry corpus tree, not the pre-registered 450,463-entry
  primary subject. Both are generated trees, which the campaign-2 plan’s corpus rule
  found understate fdu’s distance from the floor.
- fdu’s worker count was neither pinned nor recorded.
  The automatic policy starts four workers on this host, matching the floor tools, only
  because the host has fewer cores than fdu’s six-worker cap.

## What the Floor Cells Say about the Residual

`parfloor` at 316 ms and `arena_spike` at 363 ms differ by about 15% in wall time, so on
this tree retaining an index-shaped result adds little over raw parallel enumeration.
The candidate’s `default-tree` is 822 ms: 2.60× the floor in total, 1.60× above it.
That points the residual at the consumer, and
[the earlier syscall census](../research/research-2026-08-13-linux-first-measurements.md),
in which fdu issued the same syscall counts as `dut` and `diskus`, supports that
direction. This cell cannot locate the residual, though: it recorded no CPU for either
floor tool, and the candidate’s `default-tree` spends about 64% of its CPU time in the
kernel.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
