---
title: Installed CLI metadata one-shot stays cold scan on frameworks
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-107
  title: Installed CLI metadata one-shot stays cold scan on frameworks
  date: "2026-09-19"
  hypotheses:
    - H108
  subject:
    tree_label: system-private-frameworks
    tree_root_id: b718281f3051a0ed5b4fc59d83614845f67e17999095cf2d837a0c551e24869c
    tree_engine_digest: 0c863b0ab28dc47e3db5a0298fe3239a51959056ec5b97c519e49ad1bfd965bf
    tree_provenance: "The sealed macOS system volume's private frameworks, read-only and identical on every Mac running the same OS build (Darwin 25.5.0 here). Reconstructible by installing that build."
    tree_reconstructible: true
    tree_entries: 158705
    tree_directories: 55256
    tree_files: 96542
    tree_symlinks: 6907
    tree_apparent_bytes: 5752378316
    tree_allocated_bytes: 3910119424
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: Apple M1 Pro
    host_arch: arm64
    host_cores: 10
    host_performance_cores: 8
    host_efficiency_cores: 2
    host_memory_bytes: 34359738368
    host_system: Darwin 25.5.0
    filesystem: apfs
    host_virtualization: bare-metal
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 1
    interleaved: false
    control: first isolated-cache fdu PATH after OS warmup
    candidate: second fdu PATH sharing that cache
    control_binary:
      name: first
      sha256: d13f941d0ea27e385efd58d8e5582787c8e1e662c1b7728495f497cfb0db45e4
      size_bytes: 3382768
      args: []
    candidate_binary:
      name: second
      sha256: d13f941d0ea27e385efd58d8e5582787c8e1e662c1b7728495f497cfb0db45e4
      size_bytes: 3382768
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-107-h108-cli-default-tree.json
  results:
    - job: cli-default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2100000000.0
          candidate_median: 2060000000.0
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.083
          change_pct: -0.625
          ci95_low_pct: -4.968
          ci95_high_pct: 4.045
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 94117888.0
          candidate_median: 93609984.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.015
          change_pct: -0.751
          ci95_low_pct: -1.463
          ci95_high_pct: 1.461
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - cpu_ns is missing a paired percent interval
          - system_cpu_ns is missing a paired percent interval
          - minor_faults is missing a paired percent interval
          - voluntary_context_switches is missing a paired percent interval
          - involuntary_context_switches is missing a paired percent interval
          - major_faults is missing a paired absolute interval
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: inconclusive
          involuntary_context_switches: inconclusive
          major_faults: inconclusive
          minor_faults: inconclusive
          peak_rss_bytes: within-limit
          system_cpu_ns: inconclusive
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: cli-default-tree
    primary_metric: wall_ns
    change_pct: -0.625
    reason: "H108 confirmed: all 12 second runs stayed cold scan; wall -0.63% [-4.97%, +4.05%], median inside 3%. No engine change."
    commit: bd03cd6c
---
## What was predicted

H108: a metadata-only one-shot under `--cache auto` does not load a snapshot, because a
metadata snapshot cannot cheapen a full walk.
After warmup on an immutable deciding tree, the second installed-CLI `fdu PATH` stays
`cold scan` and wall stays within 3% of the first.
Content `--analyze` on a separate subtree still shows `cached` on the second run.

Accept rule, named before the run: every timed second-run footer is `cold scan`; the
paired median wall change is inside 3%; the tree fingerprint is unchanged; a content
`--analyze=code` pair still reports `cached` on the second run.
Falsified if any second metadata run is `cached` / warm revalidation, or the median wall
moves by at least 3%.

This is a determination of shipped behavior, not a code change.
Complexity is zero.

The 2026-09-18 CLI QA medium tree still exists and is deciding-scale, but it is a dirty
live checkout and that QA saw it mutate mid-run.
It was skipped. Subject: nominated `system-private-frameworks` (sealed, read-only,
reconstructible).

## What was measured

This branch’s release CLI, copied out of the tree: `fdu 0.1.0-dev+gbd03cd6cc`, sha256
`d13f941d0ea27e385efd58d8e5582787c8e1e662c1b7728495f497cfb0db45e4`. Not the
PATH-installed `fdu`. Isolated `XDG_CACHE_HOME` per pair.
Analyze-off default tree view.
`/usr/bin/time -l` wall and peak RSS. One OS-cache warmup, then 12 sequential
first/second pairs. `FDU_COUNTERS` unset on the claim-grade wall.

Quiet was attempted.
The start gate passed (CPU busy 20.44%). The cell did not hold: final CPU busy 26.08%.
Labeled **uncontrolled**. The 25% bar was not lowered.
No RAM disk. Tree fingerprint matched the nominated digest and did not move.

A third instrumented pair (`FDU_COUNTERS=1`) ran after the claim-grade wall and is
attribution only. Content `--analyze=code` used `trading/docs` (667 files) only for the
sidecar half of the prediction.

## What happened

All 12 second metadata runs, the warmup, and both instrumented runs printed `cold scan`.
Walked tally was 96,542 files / 5.3 GiB on every metadata run; ignore rules 0 files.
Each pair left a snapshot under the isolated cache and did not load it.

| Arm | Wall median | Peak RSS | files/s (median wall) |
| --- | ---: | ---: | ---: |
| first | 2.100 s | 89.7 MiB | 46.0k |
| second | 2.060 s | 89.3 MiB | 46.9k |

Wall −0.625% [−4.968%, +4.045%]. RSS −0.751% [−1.463%, +1.461%]. Median inside 3%. The
interval includes zero and crosses 3% because `/usr/bin/time` is 10 ms and the host
drifted; the footer is the categorical signal.

Content `--analyze=code` on `trading/docs`: first 667 fresh, `cold scan`, 64–76 ms;
second 667 cached / 0 B read, `warm revalidation`, 6.8–8.1 ms.

Instrumented metadata pair (not the verdict): detached walk 1.292 s of 1.34 s wall
(~96%), finish 4.9 ms, 158,705 stats, 55,256 directory opens, 0 file opens, 0 control
reads, 0 same-parent path comparisons, 193,870 syscalls.
The second instrumented run repeated those logical counts and stayed `cold scan`.

## Judgment

H108 holds on an immutable deciding tree.
A metadata one-shot that stays `cold scan` is the shipped plan, not an H9 regression:
`ReportPlan::read_snapshot` is false for this request, and the counters show the second
run repeats the walk.

No engine patch. Loading the snapshot would add reconciliation to a walk that already
stats every entry (the comment on `ReportPlan` measured 4.8 s warm vs 3.6 s cold on 494k
entries).
Skipping the write would save the finish (about 5 ms here) and break cache-only
/ watch warmup from a one-shot snapshot.
The walk is 96% of instrumented wall; that leftover is H86/H111 on Linux, not a Darwin
rewrite.

H109 is not this profile: path comparisons were 0 on metadata one-shot and on the
667-file content pair (no control files).
Next measurement is a deciding-scale `content-cache-hit` profile on a controls-bearing
tree. Do not re-run H107 on a tree whose ignored share is not the walk.
